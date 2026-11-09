use serde_json::{Value, json};
use std::process::Command;
use toolcapsule::{
    configuration_diff::encode,
    configuration_review::{MAX_HTML_BYTES, render},
};
fn sources() -> (Vec<u8>, Vec<u8>) {
    let a = json!({"schema_version":1,"kind":"configuration","valid":true,"checks":{"limits":{"status":"valid"},"policy":{"status":"valid","denies_all_tools":true},"trust":{"status":"valid","denies_all_signers":true}},"budget_fit":{"status":"fits","exceeded_fields":[]},"package_admission":"not_checked"});
    let mut b = a.clone();
    b["budget_fit"] = json!({"status":"exceeds","exceeded_fields":["fuel"]});
    (
        serde_json::to_vec(&a).unwrap(),
        serde_json::to_vec(&b).unwrap(),
    )
}
#[test]
fn bounded_semantic_html_and_hostile_input() {
    let (a, b) = sources();
    let raw = encode(&a, &b).unwrap();
    let html = render(&raw, None).unwrap();
    assert!(html.len() <= MAX_HTML_BYTES);
    assert!(html.contains("Before and after configuration summaries"));
    assert!(html.contains("Denies all signers"));
    assert!(html.contains("Changed"));
    assert!(html.contains("default-src 'none'"));
    assert!(html.contains("style-src 'sha256-"));
    assert!(!html.contains("<script"));
    assert!(!html.contains("http://"));
    assert!(
        render(&raw, Some((&a, &b)))
            .unwrap()
            .contains("Matched explicit")
    );
    assert!(render(&raw, Some((&b, &a))).is_err());
    let mut bad: Value = serde_json::from_slice(&raw).unwrap();
    bad["before"]["checks"]["limits"]["status"] = json!("<img src=x onerror=alert(1)>");
    assert!(render(&serde_json::to_vec(&bad).unwrap(), None).is_err());
    assert!(render(&vec![b' '; 16385], None).is_err());
    let same = render(&encode(&a, &a).unwrap(), None).unwrap();
    assert!(same.contains("No summary changes."));
}
#[test]
fn cli_checks_sources_before_exclusive_output() {
    let dir = tempfile::tempdir().unwrap();
    let (a, b) = sources();
    let before = dir.path().join("before.json");
    let after = dir.path().join("after.json");
    let report = dir.path().join("difference.json");
    let out = dir.path().join("review.html");
    std::fs::write(&before, &a).unwrap();
    std::fs::write(&after, &b).unwrap();
    let raw = encode(&a, &b).unwrap();
    std::fs::write(&report, &raw).unwrap();
    let run = |pair: bool| {
        let mut c = Command::new(env!("CARGO_BIN_EXE_toolcapsule"));
        c.arg("configuration-diff-html")
            .arg(&report)
            .arg("--out")
            .arg(&out);
        if pair {
            c.arg("--before").arg(&before).arg("--after").arg(&after);
        }
        c.output().unwrap()
    };
    let partial = Command::new(env!("CARGO_BIN_EXE_toolcapsule"))
        .arg("configuration-diff-html")
        .arg(&report)
        .arg("--out")
        .arg(&out)
        .arg("--before")
        .arg(&before)
        .output()
        .unwrap();
    assert!(!partial.status.success());
    assert!(!out.exists());
    assert!(partial.stdout.is_empty());
    std::fs::write(&report, encode(&b, &a).unwrap()).unwrap();
    assert!(!run(true).status.success());
    assert!(!out.exists());
    std::fs::write(&report, &raw).unwrap();
    assert!(run(true).status.success());
    let saved = std::fs::read(&out).unwrap();
    assert!(!run(false).status.success());
    assert_eq!(std::fs::read(&out).unwrap(), saved);
    assert_eq!(std::fs::read(&before).unwrap(), a);
    assert_eq!(std::fs::read(&after).unwrap(), b);
    assert_eq!(std::fs::read(&report).unwrap(), raw);
}
