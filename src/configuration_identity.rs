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
