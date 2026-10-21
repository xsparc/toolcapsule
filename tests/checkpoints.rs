use toolcapsule::{
    audit::{AuditCheckpoint, AuditEntry, Receipt},
    runtime::Outcome,
    storage::*,
};

fn log(seeds: &[u64]) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut previous = String::new();
    for (sequence, seed) in seeds.iter().enumerate() {
        let receipt = Receipt {
            schema_version: 1,
            profile: "synthetic".into(),
            tool: "fixture".into(),
            signer: "test".into(),
            manifest_sha256: "a".repeat(64),
            component_sha256: "b".repeat(64),
            input_sha256: "c".repeat(64),
            stdout_sha256: "d".repeat(64),
            stderr_sha256: "e".repeat(64),
            seed: *seed,
            outcome: Outcome::Success,
            fuel_consumed: 10,
        };
        let entry = AuditEntry::new(sequence as u64, previous, receipt).unwrap();
        previous = entry.hash.clone();
        bytes.extend(serde_json::to_vec(&entry).unwrap());
        bytes.push(b'\n');
    }
    bytes
}

#[test]
fn anchor_accepts_exact_and_append_only_prefixes() {
    let original = log(&[1, 2]);
    let anchor = checkpoint_log(&original).unwrap();
    assert_eq!(anchor.entries, 2);
    assert_eq!(
        verify_log_against(&original, &anchor, true).unwrap(),
        verify_log(&original).unwrap()
    );
    let extended = log(&[1, 2, 3]);
    assert_eq!(verify_log_against(&extended, &anchor, false).unwrap().0, 3);
    assert!(verify_log_against(&extended, &anchor, true).is_err());
}

#[test]
fn retained_anchor_detects_valid_rewrites_and_truncation() {
    let anchor = checkpoint_log(&log(&[1, 2])).unwrap();
    for altered in [log(&[9, 2]), log(&[1, 9, 3]), log(&[1]), log(&[])] {
        assert!(verify_log(&altered).is_ok());
        assert!(verify_log_against(&altered, &anchor, false).is_err());
    }
    let mut corrupt_tail = log(&[1, 2]);
    corrupt_tail.extend(b"{}\n");
    assert!(verify_log_against(&corrupt_tail, &anchor, false).is_err());
    let mut partial = log(&[1, 2, 3]);
    partial.pop();
    assert!(verify_log_against(&partial, &anchor, false).is_err());
}

#[test]
fn empty_anchor_has_explicit_semantics_and_invalid_anchors_fail() {
    let empty = checkpoint_log(b"").unwrap();
    assert!(verify_log_against(b"", &empty, true).is_ok());
    assert!(verify_log_against(&log(&[1]), &empty, false).is_ok());
    assert!(verify_log_against(&log(&[1]), &empty, true).is_err());
    for anchor in [
        AuditCheckpoint {
            schema_version: 2,
            ..empty.clone()
        },
        AuditCheckpoint {
            entries: 1,
            ..empty.clone()
        },
        AuditCheckpoint {
            checkpoint: "a".repeat(64),
            ..empty.clone()
        },
        AuditCheckpoint {
            entries: 1,
            checkpoint: "A".repeat(64),
            ..empty.clone()
        },
        AuditCheckpoint {
            entries: 1,
            checkpoint: "g".repeat(64),
            ..empty.clone()
        },
    ] {
        assert!(verify_log_against(&log(&[1]), &anchor, false).is_err());
    }
    for raw in [
        r#"{"schema_version":true,"entries":0,"checkpoint":""}"#,
        r#"{"schema_version":1,"entries":-1,"checkpoint":""}"#,
        r#"{"schema_version":1,"entries":0,"checkpoint":"","extra":true}"#,
    ] {
        assert!(serde_json::from_str::<AuditCheckpoint>(raw).is_err());
    }
    assert!(checkpoint_log(&vec![b'\n'; MAX_AUDIT_BYTES + 1]).is_err());
}
