//! Versioned normalized identity; not a signature or original-source identity.
use crate::{Error, configuration_diff::read_difference};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

/// Compact serde_json encoding of the validated closed envelope. Object keys are
/// recursively sorted by the default serde_json map; arrays retain their order.
/// The domain prefix fixes this encoding as v1, without claiming RFC 8785 JCS.
pub fn identity(raw: &[u8]) -> Result<Value, Error> {
    let checked = read_difference(raw)?;
    let bytes = serde_json::to_vec(&checked).map_err(|_| Error::Format)?;
    let mut hash = Sha256::new();
    hash.update(b"toolcapsule:configuration-difference:v1\n");
    hash.update(&bytes);
    Ok(
        json!({"schema_version":1,"kind":"configuration_difference_identity",
        "encoding":"toolcapsule_difference_v1","difference_sha256":hex::encode(hash.finalize()),
        "interpretation":"normalized_summary_identity_not_signature_or_source_identity"}),
    )
}

pub fn verify_expected(raw: &[u8], expected: &str) -> Result<(), Error> {
    if expected.len() != 64
        || !expected
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
    {
        return Err(Error::Format);
    }
    if identity(raw)?["difference_sha256"].as_str() != Some(expected) {
        return Err(Error::Format);
    }
    Ok(())
}

pub const MAX_IDENTITY_BYTES: usize = 4096;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct IdentityRecord {
    schema_version: u8,
    kind: String,
    encoding: String,
    difference_sha256: String,
    interpretation: String,
}

pub fn read_identity(raw: &[u8]) -> Result<IdentityRecord, Error> {
    if raw.len() > MAX_IDENTITY_BYTES {
        return Err(Error::InputLimit);
    }
    let record: IdentityRecord = serde_json::from_slice(raw).map_err(|_| Error::Format)?;
    if record.schema_version != 1
        || record.kind != "configuration_difference_identity"
        || record.encoding != "toolcapsule_difference_v1"
        || record.interpretation != "normalized_summary_identity_not_signature_or_source_identity"
        || record.difference_sha256.len() != 64
        || !record
            .difference_sha256
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
    {
        return Err(Error::Format);
    }
    Ok(record)
}

pub fn verify_record(raw: &[u8], record: &[u8]) -> Result<(), Error> {
    verify_expected(raw, &read_identity(record)?.difference_sha256)
}

pub fn schema() -> Value {
    serde_json::from_str(include_str!(
        "../schemas/configuration-difference-identity.schema.json"
    ))
    .expect("checked-in identity schema must be valid JSON")
}
