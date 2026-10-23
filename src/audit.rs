//! Digest-only evidence. A trusted checkpoint is needed to detect whole-log rewrites.
use crate::{
    Error,
    package::digest,
    runtime::{Outcome, PROFILE, PreparedTool, RunResult},
};
use serde::{Deserialize, Serialize};

/// An unsigned chain anchor. Its integrity must be protected outside the log.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AuditCheckpoint {
    pub schema_version: u32,
    pub entries: u64,
    pub checkpoint: String,
}

impl AuditCheckpoint {
    pub fn validate(&self) -> Result<(), Error> {
        let valid_hash = if self.entries == 0 {
            self.checkpoint.is_empty()
        } else {
            self.checkpoint.len() == 64
                && self
                    .checkpoint
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        };
        if self.schema_version != 1 || !valid_hash {
            return Err(Error::Audit);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Receipt {
    pub schema_version: u32,
    pub profile: String,
    pub tool: String,
    pub signer: String,
    pub manifest_sha256: String,
    pub component_sha256: String,
    pub input_sha256: String,
    pub stdout_sha256: String,
    pub stderr_sha256: String,
    pub seed: u64,
    pub outcome: Outcome,
    pub fuel_consumed: u64,
}

impl Receipt {
    pub fn new(tool: &PreparedTool, input: &[u8], seed: u64, result: &RunResult) -> Self {
        Self {
            schema_version: 1,
            profile: PROFILE.into(),
            tool: tool.manifest().name.clone(),
            signer: tool.signer().into(),
            manifest_sha256: tool.manifest_sha256().into(),
            component_sha256: tool.manifest().component_sha256.clone(),
            input_sha256: digest(input),
            stdout_sha256: digest(&result.stdout),
            stderr_sha256: digest(&result.stderr),
            seed,
            outcome: result.outcome.clone(),
            fuel_consumed: result.fuel_consumed,
        }
    }

    pub async fn replay(&self, tool: &PreparedTool, input: &[u8]) -> Result<RunResult, Error> {
        if self.schema_version != 1
            || self.profile != PROFILE
            || self.outcome != Outcome::Success
            || self.tool != tool.manifest().name
            || self.signer != tool.signer()
            || self.manifest_sha256 != tool.manifest_sha256()
            || self.component_sha256 != tool.manifest().component_sha256
            || self.input_sha256 != digest(input)
        {
            return Err(Error::Replay);
        }
        let result = tool.run(input, self.seed).await?;
        if Self::new(tool, input, self.seed, &result) != *self {
            return Err(Error::Replay);
        }
        Ok(result)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditEntry {
    pub sequence: u64,
    pub previous: String,
    pub receipt: Receipt,
    pub hash: String,
}

impl AuditEntry {
    pub fn new(sequence: u64, previous: String, receipt: Receipt) -> Result<Self, Error> {
        let hash = Self::hash(sequence, &previous, &receipt)?;
        Ok(Self {
            sequence,
            previous,
            receipt,
            hash,
        })
    }
    fn hash(sequence: u64, previous: &str, receipt: &Receipt) -> Result<String, Error> {
        let bytes = serde_json::to_vec(&("toolcapsule-audit-v1", sequence, previous, receipt))
            .map_err(|_| Error::Audit)?;
        Ok(digest(&bytes))
    }
    pub fn verify(&self, expected_sequence: u64, expected_previous: &str) -> Result<(), Error> {
        if self.receipt.schema_version != 1
            || self.sequence != expected_sequence
            || self.previous != expected_previous
            || self.hash != Self::hash(self.sequence, &self.previous, &self.receipt)?
        {
            return Err(Error::Audit);
        }
        Ok(())
    }
}
