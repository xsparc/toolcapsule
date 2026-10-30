use ed25519_dalek::SigningKey;
use toolcapsule::trust_check::{MAX_TRUST_BYTES, check_trust};

#[test]
fn accepts_empty_and_valid_public_keys_without_returning_them() {
    assert_eq!(check_trust(b"{}").unwrap(), 0);
    let key = hex::encode(SigningKey::from_bytes(&[7; 32]).verifying_key().to_bytes());
    assert_eq!(
        check_trust(format!(r#"{{"one":"{key}","two":"{key}"}}"#).as_bytes()).unwrap(),
        2
    );
}

#[test]
fn rejects_duplicate_decoded_ids_bad_keys_types_trailing_data_and_size() {
    let key = hex::encode(SigningKey::from_bytes(&[7; 32]).verifying_key().to_bytes());
    for bad in [
        format!(r#"{{"one":"{key}","\u006fne":"{key}"}}"#),
        format!(r#"{{"":"{key}"}}"#),
        format!(r#"{{"{}":"{key}"}}"#, "x".repeat(65)),
        r#"{"one":"xyz"}"#.into(),
        r#"{"one":false}"#.into(),
        format!(r#"{{"one":"{}"}}"#, "00".repeat(32)),
        format!(r#"{{"one":"{}"}}"#, "11".repeat(64)),
        "[]".into(),
        "{} {}".into(),
        " ".repeat(MAX_TRUST_BYTES + 1),
    ] {
        assert!(check_trust(bad.as_bytes()).is_err());
    }
}
