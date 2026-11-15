//! Directional differences between validated, deliberately lossy summaries.
use crate::{Error, configuration::read_result};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub const MAX_DIFF_BYTES: usize = 16 * 1024;
pub const FIELDS: [&str; 11] = [
    "/valid",
    "/checks/limits/status",
    "/checks/limits/error",
    "/checks/policy/status",
    "/checks/policy/error",
    "/checks/policy/denies_all_tools",
    "/checks/trust/status",
    "/checks/trust/error",
    "/checks/trust/denies_all_signers",
    "/budget_fit/status",
    "/budget_fit/exceeded_fields",
];

pub fn compare(before: &[u8], after: &[u8]) -> Result<Value, Error> {
    let before = serde_json::to_value(read_result(before)?).map_err(|_| Error::Format)?;
    let after = serde_json::to_value(read_result(after)?).map_err(|_| Error::Format)?;
    let changed: Vec<_> = FIELDS
        .into_iter()
        .filter(|field| before.pointer(field) != after.pointer(field))
        .collect();
    Ok(json!({"schema_version":1,"kind":"configuration_difference",
        "direction":"before_to_after","identical_summaries":changed.is_empty(),
        "before":before,"after":after,"changed_fields":changed,
        "interpretation":"summary_changes_not_source_identity_or_authority"}))
}

pub fn encode(before: &[u8], after: &[u8]) -> Result<Vec<u8>, Error> {
    let mut bytes =
        serde_json::to_vec_pretty(&compare(before, after)?).map_err(|_| Error::Format)?;
    bytes.push(b'\n');
    if bytes.len() > MAX_DIFF_BYTES {
        return Err(Error::InputLimit);
    }
    Ok(bytes)
}

pub fn schema() -> Value {
    serde_json::from_str(include_str!(
        "../schemas/configuration-difference.schema.json"
    ))
    .expect("checked-in difference schema must be valid JSON")
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawDifference {
    schema_version: u8,
    kind: String,
    direction: String,
    identical_summaries: bool,
    before: crate::configuration::RawResult,
    after: crate::configuration::RawResult,
    changed_fields: Vec<String>,
    interpretation: String,
}

/// Validate the complete retained envelope, including duplicate keys and derived fields.
pub fn read_difference(raw: &[u8]) -> Result<Value, Error> {
    if raw.len() > MAX_DIFF_BYTES {
        return Err(Error::InputLimit);
    }
    let retained: RawDifference = serde_json::from_slice(raw).map_err(|_| Error::Format)?;
    let before = serde_json::to_vec(&retained.before).map_err(|_| Error::Format)?;
    let after = serde_json::to_vec(&retained.after).map_err(|_| Error::Format)?;
    let expected = compare(&before, &after)?;
    if serde_json::to_value(retained).map_err(|_| Error::Format)? != expected {
        return Err(Error::Format);
    }
    Ok(expected)
}

/// Explicit summaries establish local consistency, never original-source identity.
pub fn verify_difference(raw: &[u8], before: &[u8], after: &[u8]) -> Result<(), Error> {
    if read_difference(raw)? != compare(before, after)? {
        return Err(Error::Format);
    }
    Ok(())
}
