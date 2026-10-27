use base64::{Engine as _, engine::general_purpose::STANDARD};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

const DOMAIN: &[u8] = b"toolcapsule-manifest-v1\0";
pub const MAX_MANIFEST_BYTES: usize = 16 * 1024;
pub const MAX_ENVELOPE_BYTES: usize = 24 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid package format")]
    Format,
    #[error("untrusted signer or invalid signature")]
    Signature,
    #[error("component digest mismatch")]
    Digest,
    #[error("capability or budget denied by host policy")]
    Policy,
    #[error("input exceeds its budget")]
    InputLimit,
    #[error("component compilation or interface validation failed")]
    Component,
    #[error("runtime unavailable")]
    Runtime,
    #[error("execution pool is at capacity")]
    Busy,
    #[error("replay differs from the recorded execution")]
    Replay,
    #[error("invalid or inconsistent audit chain")]
    Audit,
    #[error("local I/O failed")]
    Io,
}

pub fn digest(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Limits {
    pub fuel: u64,
    pub memory_bytes: usize,
    pub timeout_ms: u64,
    pub input_bytes: usize,
    pub output_bytes: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            fuel: 10_000_000,
            memory_bytes: 64 * 1024 * 1024,
            timeout_ms: 1000,
            input_bytes: 64 * 1024,
            output_bytes: 64 * 1024,
        }
    }
}

impl Limits {
    /// Validate budget shape without granting authority from any host policy.
    pub fn validate(&self) -> Result<(), Error> {
        if self.admitted_by(self) {
            Ok(())
        } else {
            Err(Error::Policy)
        }
    }

    fn admitted_by(&self, max: &Self) -> bool {
        self.fuel > 0
            && self.fuel <= max.fuel
            && self.memory_bytes >= 65536
            && self.memory_bytes <= max.memory_bytes
            && self.timeout_ms >= 10
            && self.timeout_ms <= max.timeout_ms
            && self.input_bytes > 0
            && self.input_bytes <= max.input_bytes
            && self.output_bytes > 0
            && self.output_bytes <= max.output_bytes
    }
}

/// The MVP profile has no host I/O capabilities. Unsupported grants fail closed.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Capabilities {
    pub filesystem: Vec<String>,
    pub network: Vec<String>,
    pub secrets: Vec<String>,
    pub subprocess: bool,
    pub clocks: String,
    pub randomness: String,
}

impl Default for Capabilities {
    fn default() -> Self {
        Self {
            filesystem: vec![],
            network: vec![],
            secrets: vec![],
            subprocess: false,
            clocks: "fixed".into(),
            randomness: "seeded".into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema_version: u32,
    pub name: String,
    pub version: String,
    pub description: String,
    pub abi: String,
    pub component_sha256: String,
    pub capabilities: Capabilities,
    pub limits: Limits,
}

impl Manifest {
    pub fn validate(&self) -> Result<(), Error> {
        if self.schema_version != 1
            || !valid_tool_name(&self.name)
            || self.version.is_empty()
            || self.version.len() > 64
            || self.description.len() > 512
            || self.abi != "wasi:cli/command@0.2"
            || self.component_sha256.len() != 64
            || !self
                .component_sha256
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(Error::Format);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignedManifest {
    pub key_id: String,
    /// Base64 preserves the exact signed JSON bytes, avoiding canonicalization ambiguity.
    pub manifest: String,
    pub signature: String,
}

impl SignedManifest {
    pub fn sign(manifest: &Manifest, key_id: &str, key: &SigningKey) -> Result<Self, Error> {
        manifest.validate()?;
        if key_id.is_empty() || key_id.len() > 64 {
            return Err(Error::Format);
        }
        let bytes = serde_json::to_vec(manifest).map_err(|_| Error::Format)?;
        if bytes.len() > MAX_MANIFEST_BYTES {
            return Err(Error::Format);
        }
        let signature = key.sign(&[DOMAIN, bytes.as_slice()].concat());
        Ok(Self {
            key_id: key_id.into(),
            manifest: STANDARD.encode(bytes),
            signature: hex::encode(signature.to_bytes()),
        })
    }
}

/// Populated by the host operator, never by a tool package.
pub type TrustStore = BTreeMap<String, String>;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub max_limits: Limits,
    pub max_component_bytes: usize,
    pub allowed_tools: Vec<String>,
}

fn valid_tool_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

impl Policy {
    /// Check configuration authoring errors without admitting any package.
    /// An empty allowlist is valid and denies every tool.
    pub fn validate(&self) -> Result<(), Error> {
        self.max_limits.validate()?;
        if self.max_component_bytes == 0
            || self.allowed_tools.iter().any(|name| !valid_tool_name(name))
            || self
                .allowed_tools
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != self.allowed_tools.len()
        {
            return Err(Error::Policy);
        }
        Ok(())
    }
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            max_limits: Limits::default(),
            max_component_bytes: 16 * 1024 * 1024,
            allowed_tools: vec![],
        }
    }
}

/// Fields are private so callers cannot construct or mutate a verified package.
pub struct VerifiedPackage {
    manifest: Manifest,
    manifest_sha256: String,
    signer: String,
    component: Vec<u8>,
}

impl VerifiedPackage {
    pub fn verify(
        envelope: &[u8],
        component: Vec<u8>,
        trust: &TrustStore,
        policy: &Policy,
    ) -> Result<Self, Error> {
        if envelope.len() > MAX_ENVELOPE_BYTES || component.len() > policy.max_component_bytes {
            return Err(Error::Policy);
        }
        let signed: SignedManifest = serde_json::from_slice(envelope).map_err(|_| Error::Format)?;
        if signed.key_id.is_empty() || signed.key_id.len() > 64 {
            return Err(Error::Format);
        }
        let key_hex = trust.get(&signed.key_id).ok_or(Error::Signature)?;
        let key_bytes: [u8; 32] = hex::decode(key_hex)
            .map_err(|_| Error::Signature)?
            .try_into()
            .map_err(|_| Error::Signature)?;
        let key = VerifyingKey::from_bytes(&key_bytes).map_err(|_| Error::Signature)?;
        let signature =
            Signature::from_slice(&hex::decode(&signed.signature).map_err(|_| Error::Signature)?)
                .map_err(|_| Error::Signature)?;
        let bytes = STANDARD
            .decode(&signed.manifest)
            .map_err(|_| Error::Format)?;
        if bytes.len() > MAX_MANIFEST_BYTES {
            return Err(Error::Format);
        }
        key.verify_strict(&[DOMAIN, bytes.as_slice()].concat(), &signature)
            .map_err(|_| Error::Signature)?;
        let manifest: Manifest = serde_json::from_slice(&bytes).map_err(|_| Error::Format)?;
        manifest.validate()?;
        if manifest.capabilities != Capabilities::default()
            || !manifest.limits.admitted_by(&policy.max_limits)
            || !policy.allowed_tools.contains(&manifest.name)
        {
            return Err(Error::Policy);
        }
        if digest(&component) != manifest.component_sha256 {
            return Err(Error::Digest);
        }
        Ok(Self {
            manifest,
            manifest_sha256: digest(&bytes),
            signer: signed.key_id,
            component,
        })
    }

    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }
    pub fn manifest_sha256(&self) -> &str {
        &self.manifest_sha256
    }
    pub fn signer(&self) -> &str {
        &self.signer
    }
    pub fn component(&self) -> &[u8] {
        &self.component
    }
}
