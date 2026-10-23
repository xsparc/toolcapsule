//! Optional checkpoint authentication. Signatures do not establish freshness.
use crate::{Error, TrustStore, audit::AuditCheckpoint};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};

const DOMAIN: &[u8] = b"toolcapsule-audit-checkpoint-v1\0";
pub const MAX_SIGNED_CHECKPOINT_BYTES: usize = 8192;
pub const MAX_CHECKPOINT_BYTES: usize = 4096;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignedCheckpoint {
    pub schema_version: u32,
    pub key_id: String,
    /// Base64 retains the exact authenticated JSON bytes.
    pub checkpoint: String,
    pub signature: String,
}

impl SignedCheckpoint {
    pub fn sign(
        checkpoint: &AuditCheckpoint,
        key_id: &str,
        key: &SigningKey,
    ) -> Result<Self, Error> {
        checkpoint.validate()?;
        if key_id.is_empty() || key_id.len() > 64 {
            return Err(Error::Format);
        }
        let bytes = serde_json::to_vec(checkpoint).map_err(|_| Error::Format)?;
        let signature = key.sign(&[DOMAIN, bytes.as_slice()].concat());
        Ok(Self {
            schema_version: 1,
            key_id: key_id.into(),
            checkpoint: STANDARD.encode(bytes),
            signature: hex::encode(signature.to_bytes()),
        })
    }

    /// Verify against the caller-selected checkpoint trust store on every call.
    pub fn verify(envelope: &[u8], trust: &TrustStore) -> Result<AuditCheckpoint, Error> {
        if envelope.len() > MAX_SIGNED_CHECKPOINT_BYTES {
            return Err(Error::InputLimit);
        }
        let signed: Self = serde_json::from_slice(envelope).map_err(|_| Error::Format)?;
        if signed.schema_version != 1 || signed.key_id.is_empty() || signed.key_id.len() > 64 {
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
            .decode(&signed.checkpoint)
            .map_err(|_| Error::Format)?;
        if bytes.len() > MAX_CHECKPOINT_BYTES {
            return Err(Error::InputLimit);
        }
        key.verify_strict(&[DOMAIN, bytes.as_slice()].concat(), &signature)
            .map_err(|_| Error::Signature)?;
        let checkpoint: AuditCheckpoint =
            serde_json::from_slice(&bytes).map_err(|_| Error::Format)?;
        checkpoint.validate()?;
        Ok(checkpoint)
    }
}
