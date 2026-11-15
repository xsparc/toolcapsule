//! Portable, strictly checked summaries and their derived difference.
use crate::{
    Error,
    configuration::{RawResult, read_result},
    configuration_diff::{RawDifference, compare, read_difference},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
pub const MAX_BUNDLE_BYTES: usize = 32 * 1024;
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RawBundle {
    schema_version: u8,
    kind: String,
    before: RawResult,
    after: RawResult,
    difference: RawDifference,
    interpretation: String,
}
pub fn create(before: &[u8], after: &[u8]) -> Result<Value, Error> {
    let a = serde_json::to_value(read_result(before)?).map_err(|_| Error::Format)?;
    let b = serde_json::to_value(read_result(after)?).map_err(|_| Error::Format)?;
    let difference = compare(before, after)?;
    Ok(
        json!({"schema_version":1,"kind":"configuration_summary_bundle","before":a,"after":b,
        "difference":difference,"interpretation":"summary_consistency_not_source_identity_or_authority"}),
    )
}
pub fn encode(before: &[u8], after: &[u8]) -> Result<Vec<u8>, Error> {
    let mut bytes =
        serde_json::to_vec_pretty(&create(before, after)?).map_err(|_| Error::Format)?;
    bytes.push(b'\n');
    if bytes.len() > MAX_BUNDLE_BYTES {
        return Err(Error::InputLimit);
    }
    Ok(bytes)
}
pub fn read_bundle(raw: &[u8]) -> Result<Value, Error> {
    if raw.len() > MAX_BUNDLE_BYTES {
        return Err(Error::InputLimit);
    }
    let retained: RawBundle = serde_json::from_slice(raw).map_err(|_| Error::Format)?;
    let before = serde_json::to_vec(&retained.before).map_err(|_| Error::Format)?;
    let after = serde_json::to_vec(&retained.after).map_err(|_| Error::Format)?;
    let diff = serde_json::to_vec(&retained.difference).map_err(|_| Error::Format)?;
    read_difference(&diff)?;
    let expected = create(&before, &after)?;
    if serde_json::to_value(retained).map_err(|_| Error::Format)? != expected {
        return Err(Error::Format);
    }
    Ok(expected)
}
pub fn verify_sources(raw: &[u8], before: &[u8], after: &[u8]) -> Result<(), Error> {
    if read_bundle(raw)? != create(before, after)? {
        return Err(Error::Format);
    }
    Ok(())
}
pub fn schema() -> Value {
    serde_json::from_str(include_str!(
        "../schemas/configuration-summary-bundle.schema.json"
    ))
    .expect("checked bundle schema")
}
