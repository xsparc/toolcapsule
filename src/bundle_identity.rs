//! Versioned normalized identity; not a signature or original-source identity.
use crate::{Error, configuration_bundle::read_bundle};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

/// Compact serde_json encoding of the validated closed envelope. Object keys are
/// recursively sorted by the default serde_json map; arrays retain their order.
/// The domain prefix fixes this encoding as v1, without claiming RFC 8785 JCS.
pub fn identity(raw: &[u8]) -> Result<Value, Error> {
    let checked = read_bundle(raw)?;
    let bytes = serde_json::to_vec(&checked).map_err(|_| Error::Format)?;
    let mut hash = Sha256::new();
    hash.update(b"toolcapsule:configuration-bundle:v1\n");
    hash.update(&bytes);
    Ok(
        json!({"schema_version":1,"kind":"configuration_bundle_identity",
        "encoding":"toolcapsule_bundle_v1","bundle_sha256":hex::encode(hash.finalize()),
        "bytes":bytes.len(),"interpretation":"normalized_summary_identity_not_signature_or_source_identity"}),
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
    if identity(raw)?["bundle_sha256"].as_str() != Some(expected) {
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
    bundle_sha256: String,
    interpretation: String,
    bytes: usize,
}

pub fn read_identity(raw: &[u8]) -> Result<IdentityRecord, Error> {
    if raw.len() > MAX_IDENTITY_BYTES {
        return Err(Error::InputLimit);
    }
    let record: IdentityRecord = serde_json::from_slice(raw).map_err(|_| Error::Format)?;
    if record.bytes == 0
        || record.bytes > crate::configuration_bundle::MAX_BUNDLE_BYTES
        || record.schema_version != 1
        || record.kind != "configuration_bundle_identity"
        || record.encoding != "toolcapsule_bundle_v1"
        || record.interpretation != "normalized_summary_identity_not_signature_or_source_identity"
        || record.bundle_sha256.len() != 64
        || !record
            .bundle_sha256
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
    {
        return Err(Error::Format);
    }
    Ok(record)
}

pub fn verify_record(raw: &[u8], record: &[u8]) -> Result<(), Error> {
    let retained = read_identity(record)?;
    let actual = identity(raw)?;
    if actual["bytes"].as_u64() != Some(retained.bytes as u64) {
        return Err(Error::Format);
    }
    verify_expected(raw, &retained.bundle_sha256)
}

pub fn schema() -> Value {
    serde_json::from_str(include_str!(
        "../schemas/configuration-bundle-identity.schema.json"
    ))
    .expect("checked-in identity schema must be valid JSON")
}
