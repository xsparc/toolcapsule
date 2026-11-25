//! Parse a bounded verification response without executing the recorded checks.
use crate::Error;
use serde::Deserialize;
use serde_json::Value;

/// Maximum bytes accepted for a single response, including whitespace.
pub const MAX_RESPONSE_BYTES: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationKind {
    ConfigurationBundle,
    ConfigurationBundleExport,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireResponse {
    schema_version: u8,
    verified: bool,
    interpretation: String,
    kind: VerificationKind,
    source_checked: bool,
    identity_checked: bool,
}

/// These flags describe the supplied record's claims, not trusted evidence.
#[derive(Debug, PartialEq, Eq)]
pub struct VerificationResponse {
    pub kind: VerificationKind,
    pub source_checked: bool,
    pub identity_checked: bool,
}

/// Require a JSON object with known, unique fields and supported claims.
///
/// This does not check a bundle, export, signature, or independently held source.
pub fn read_response(raw: &[u8]) -> Result<VerificationResponse, Error> {
    if raw.len() > MAX_RESPONSE_BYTES {
        return Err(Error::InputLimit);
    }
    // Serde also accepts positional arrays for derived structs. Require the
    // schema's object form, then decode the original bytes to reject duplicates.
    if raw
        .iter()
        .copied()
        .find(|b| !matches!(b, b' ' | b'\t' | b'\n' | b'\r'))
        != Some(b'{')
    {
        return Err(Error::Format);
    }
    let response: WireResponse = serde_json::from_slice(raw).map_err(|_| Error::Format)?;
    if response.schema_version != 1
        || !response.verified
        || response.interpretation != "summary_consistency_not_source_identity_or_authority"
    {
        return Err(Error::Format);
    }
    Ok(VerificationResponse {
        kind: response.kind,
        source_checked: response.source_checked,
        identity_checked: response.identity_checked,
    })
}

pub fn schema() -> Value {
    serde_json::from_str(include_str!(
        "../schemas/configuration-bundle-verification.schema.json"
    ))
    .expect("checked-in verification response schema must be valid JSON")
}
