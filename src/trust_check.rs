//! Read-only trust-file authoring checks; never changes runtime trust.
use crate::Error;
use ed25519_dalek::VerifyingKey;
use serde::Deserializer;
use serde::de::{self, MapAccess, Visitor};
use std::{collections::BTreeSet, fmt};

pub const MAX_TRUST_BYTES: usize = 64 * 1024;

/// Count valid entries without returning public keys or identifiers.
/// Empty maps are valid deny-all stores. Duplicate decoded IDs are rejected.
pub fn check_trust(bytes: &[u8]) -> Result<usize, Error> {
    if bytes.len() > MAX_TRUST_BYTES {
        return Err(Error::InputLimit);
    }
    struct TrustVisitor;
    impl<'de> Visitor<'de> for TrustVisitor {
        type Value = usize;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("a trust-store object")
        }
        fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<usize, M::Error> {
            let mut ids = BTreeSet::new();
            while let Some((id, encoded)) = map.next_entry::<String, String>()? {
                if id.is_empty() || id.len() > 64 || !ids.insert(id) {
                    return Err(de::Error::custom("invalid trust entry"));
                }
                let raw: [u8; 32] = hex::decode(encoded)
                    .map_err(|_| de::Error::custom("invalid public key"))?
                    .try_into()
                    .map_err(|_| de::Error::custom("invalid public key"))?;
                let key = VerifyingKey::from_bytes(&raw)
                    .map_err(|_| de::Error::custom("invalid public key"))?;
                if key.is_weak() {
                    return Err(de::Error::custom("weak public key"));
                }
            }
            Ok(ids.len())
        }
    }
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let count = deserializer
        .deserialize_map(TrustVisitor)
        .map_err(|_| Error::Format)?;
    deserializer.end().map_err(|_| Error::Format)?;
    Ok(count)
}
