use serde_json::json;
use std::process::Command;
use toolcapsule::configuration_bundle::{encode, render, verify_export};
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
fn exact_bundle_exports_and_source_modes() {
    let (a, b) = pair();
    let raw = encode(&a, &b).unwrap();
    for format in ["html", "csv"] {
        let export = render(&raw, format, Some((&a, &b))).unwrap();
        verify_export(&raw, &export, format, Some((&a, &b))).unwrap();
        let mut edited = export.clone();
        edited.push(b' ');
        assert!(verify_export(&raw, &edited, format, Some((&a, &b))).is_err());
        assert!(verify_export(&raw, &vec![0; 65537], format, None).is_err());
        assert!(render(&raw, format, Some((&b, &a))).is_err());
        let replacement = encode(&a, &a).unwrap();
        let replaced = render(&replacement, format, None).unwrap();
        verify_export(&replacement, &replaced, format, None).unwrap();
        assert!(verify_export(&replacement, &replaced, format, Some((&a, &b))).is_err());
        if format == "html" {
            let text = String::from_utf8(export.clone()).unwrap();
            assert!(text.contains("Matched explicit before/after summaries"));
            assert!(verify_export(&raw, &export, format, None).is_err());
            let tampered = text.replace("Content-Security-Policy", "Removed-Policy");
            assert!(verify_export(&raw, tampered.as_bytes(), format, Some((&a, &b))).is_err());
        }
    }
    assert!(render(&raw, "unknown", None).is_err());
    let mut hostile: serde_json::Value = serde_json::from_slice(&raw).unwrap();
    hostile["before"]["checks"]["limits"]["status"] = json!("<script>alert(1)</script>");
    assert!(render(&serde_json::to_vec(&hostile).unwrap(), "html", None).is_err());
    let mut failed: serde_json::Value = serde_json::from_slice(&a).unwrap();
    failed["valid"] = json!(false);
    failed["checks"]["limits"] = json!({"status":"invalid","error":"invalid_format"});
    failed["budget_fit"] = json!({"status":"not_checked","exceeded_fields":[]});
    let failed = encode(&serde_json::to_vec(&failed).unwrap(), &b).unwrap();
    for format in ["html", "csv"] {
        render(&failed, format, None).unwrap();
    }
}

#[test]
fn cli_bundle_export_is_exclusive_and_verification_read_only() {
    let d = tempfile::tempdir().unwrap();
    let (a, b) = pair();
    let bundle = encode(&a, &b).unwrap();
    for (name, raw) in [("a", &a), ("b", &b), ("bundle", &bundle)] {
        std::fs::write(d.path().join(name), raw).unwrap();
    }
    let run = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_toolcapsule"))
            .current_dir(d.path())
            .args(args)
            .output()
            .unwrap()
    };
    for format in ["html", "csv"] {
        let args = [
            "configuration-bundle-render",
            "bundle",
            "--format",
            format,
            "--out",
            format,
            "--before",
            "a",
            "--after",
            "b",
        ];
        assert!(run(&args).status.success());
        let saved = std::fs::read(d.path().join(format)).unwrap();
        let collision = run(&args);
        assert_eq!(collision.status.code(), Some(2));
        let verify = [
            "configuration-bundle-verify-export",
            "bundle",
            format,
            "--format",
            format,
            "--before",
            "a",
            "--after",
            "b",
        ];
        assert!(run(&verify).status.success());
        assert_eq!(std::fs::read(d.path().join(format)).unwrap(), saved);
        std::fs::write(d.path().join("edited"), [saved.as_slice(), b"\n"].concat()).unwrap();
        assert_eq!(
            run(&[
                "configuration-bundle-verify-export",
                "bundle",
                "edited",
                "--format",
                format,
                "--before",
                "a",
                "--after",
                "b"
            ])
            .status
            .code(),
            Some(2)
        );
        let bad = run(&[
            "configuration-bundle-render",
            "bundle",
            "--out",
            "blocked",
            "--before",
            "b",
            "--after",
            "a",
        ]);
        assert_eq!(bad.status.code(), Some(2));
        assert!(!d.path().join("blocked").exists());
        assert!(!String::from_utf8_lossy(&bad.stderr).contains(&d.path().display().to_string()));
    }
    assert_eq!(
        run(&[
            "configuration-bundle-render",
            "bundle",
            "--out",
            "partial",
            "--before",
            "a"
        ])
        .status
        .code(),
        Some(2)
    );
    assert!(!d.path().join("partial").exists());
    for (name, raw) in [("a", a), ("b", b), ("bundle", bundle)] {
        assert_eq!(std::fs::read(d.path().join(name)).unwrap(), raw);
    }
}
