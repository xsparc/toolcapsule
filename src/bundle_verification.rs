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
    require_object(raw)?;
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

fn require_object(raw: &[u8]) -> Result<(), Error> {
    if raw
        .iter()
        .copied()
        .find(|b| !matches!(b, b' ' | b'\t' | b'\n' | b'\r'))
        != Some(b'{')
    {
        return Err(Error::Format);
    }
    Ok(())
}

pub const MAX_BATCH_RESPONSES: usize = 32;
pub const BATCH_INTERPRETATION: &str = "verification_response_validity_counts_only";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BatchReceipt {
    schema_version: u8,
    format_valid: u8,
    invalid_format: u8,
    oversized: u8,
    unreadable: u8,
    total: u8,
    interpretation: String,
}

/// Check bounded format and arithmetic only, never the underlying evidence.
pub fn check_batch_receipt(raw: &[u8]) -> Result<(), Error> {
    if raw.len() > MAX_RESPONSE_BYTES {
        return Err(Error::InputLimit);
    }
    require_object(raw)?;
    let receipt: BatchReceipt = serde_json::from_slice(raw).map_err(|_| Error::Format)?;
    let total = usize::from(receipt.total);
    let counts = [
        receipt.format_valid,
        receipt.invalid_format,
        receipt.oversized,
        receipt.unreadable,
    ];
    if receipt.schema_version != 1
        || receipt.interpretation != BATCH_INTERPRETATION
        || !(1..=MAX_BATCH_RESPONSES).contains(&total)
        || counts.into_iter().map(usize::from).sum::<usize>() != total
    {
        return Err(Error::Format);
    }
    Ok(())
}

pub fn batch_schema() -> Value {
    serde_json::from_str(include_str!(
        "../schemas/configuration-bundle-verification-batch.schema.json"
    ))
    .expect("checked-in batch receipt schema must be valid JSON")
}
