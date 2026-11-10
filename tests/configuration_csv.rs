use serde_json::json;
use std::process::Command;
use toolcapsule::{
    configuration_csv::{MAX_CSV_BYTES, render, verify},
    configuration_diff::encode,
};
fn pair() -> (Vec<u8>, Vec<u8>) {
    let a = json!({"schema_version":1,"kind":"configuration","valid":true,"checks":{"limits":{"status":"valid"},"policy":{"status":"valid","denies_all_tools":true},"trust":{"status":"valid","denies_all_signers":true}},"budget_fit":{"status":"fits","exceeded_fields":[]},"package_admission":"not_checked"});
    let mut b = a.clone();
    b["budget_fit"] = json!({"status":"exceeds","exceeded_fields":["fuel","memory_bytes"]});
    (
        serde_json::to_vec(&a).unwrap(),
        serde_json::to_vec(&b).unwrap(),
    )
}
#[test]
fn canonical_rows_quoting_and_source_gate() {
    let (a, b) = pair();
    let raw = encode(&a, &b).unwrap();
    let csv = render(&raw, None).unwrap();
    let text = std::str::from_utf8(&csv).unwrap();
    assert!(text.starts_with("\"schema_version\",\"direction\",\"field\",\"before\",\"after\",\"change\",\"interpretation\"\r\n"));
    assert_eq!(text.split("\r\n").count(), 13);
    assert!(text.ends_with("\r\n"));
    assert!(text.contains("\"text:[\"\"fuel\"\",\"\"memory_bytes\"\"]\""));
    assert!(text.contains("\"text:not_reported\""));
    verify(&raw, &csv, Some((&a, &b))).unwrap();
    assert!(verify(&raw, &csv, Some((&b, &a))).is_err());
    let replaced = encode(&b, &a).unwrap();
    let replaced_csv = render(&replaced, None).unwrap();
    verify(&replaced, &replaced_csv, None).unwrap();
    assert!(verify(&replaced, &replaced_csv, Some((&a, &b))).is_err());
}
#[test]
fn exact_bytes_reject_repairs_and_bad_evidence() {
    let (a, b) = pair();
    let raw = encode(&a, &b).unwrap();
    let csv = render(&raw, None).unwrap();
    let text = String::from_utf8(csv.clone()).unwrap();
    let mut rows: Vec<_> = text.split("\r\n").collect();
    rows.swap(1, 2);
    for bad in [
        text.replace("\r\n", "\n").into_bytes(),
        text.replace("text:", "").into_bytes(),
        text.replace("unchanged", "changed").into_bytes(),
        rows.join("\r\n").into_bytes(),
        [csv.clone(), b"\r\n".to_vec()].concat(),
        vec![0xff],
        vec![b' '; MAX_CSV_BYTES + 1],
    ] {
        assert!(verify(&raw, &bad, None).is_err());
    }
    assert!(render(b"{}", None).is_err());
    assert!(render(&vec![b' '; 16385], None).is_err());
}
#[test]
fn cli_exclusive_export_and_readonly_verification() {
    let dir = tempfile::tempdir().unwrap();
    let (a, b) = pair();
    let report = dir.path().join("diff.json");
    let csv = dir.path().join("review.csv");
    let before = dir.path().join("before.json");
    let after = dir.path().join("after.json");
    let raw = encode(&a, &b).unwrap();
    std::fs::write(&report, &raw).unwrap();
    std::fs::write(&before, &a).unwrap();
    std::fs::write(&after, &b).unwrap();
    let run = |verify: bool| {
        let mut c = Command::new(env!("CARGO_BIN_EXE_toolcapsule"));
        c.arg(if verify {
            "configuration-diff-csv-verify"
        } else {
            "configuration-diff-csv"
        })
        .arg(&report);
        if !verify {
            c.arg("--out");
        }
        c.arg(&csv)
            .arg("--before")
            .arg(&before)
            .arg("--after")
            .arg(&after);
        c.output().unwrap()
    };
    std::fs::write(&report, encode(&b, &a).unwrap()).unwrap();
    assert!(!run(false).status.success());
    assert!(!csv.exists());
    std::fs::write(&report, &raw).unwrap();
    assert!(run(false).status.success());
    let saved = std::fs::read(&csv).unwrap();
    assert!(!run(false).status.success());
    assert!(run(true).status.success());
    let partial = Command::new(env!("CARGO_BIN_EXE_toolcapsule"))
        .arg("configuration-diff-csv-verify")
        .arg(&report)
        .arg(&csv)
        .arg("--before")
        .arg(&before)
        .output()
        .unwrap();
    assert!(!partial.status.success());
    assert!(partial.stdout.is_empty());
    std::fs::write(&csv, b"private-marker").unwrap();
    let failure = run(true);
    assert!(!failure.status.success());
    assert!(failure.stdout.is_empty());
    assert!(!String::from_utf8_lossy(&failure.stderr).contains("private-marker"));
    assert_eq!(std::fs::read(&csv).unwrap(), b"private-marker");
    assert_eq!(saved, render(&raw, None).unwrap());
    assert_eq!(std::fs::read(&before).unwrap(), a);
    assert_eq!(std::fs::read(&after).unwrap(), b);
    assert_eq!(std::fs::read(&report).unwrap(), raw);
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 4);
}
