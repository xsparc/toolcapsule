use serde_json::json;
use toolcapsule::{
    bundle_identity::{identity, read_identity, schema, verify_expected, verify_record},
    configuration_bundle::encode,
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
fn normalized_closed_bundle_identity() {
    let (a, b) = pair();
    let raw = encode(&a, &b).unwrap();
    let value = identity(&raw).unwrap();
    assert_eq!(
        value["bundle_sha256"],
        "bc159345e23e65a16d370806e0c25c047736645b4b9e276d61a61a4317764b49"
    );
    assert_eq!(value["bytes"], 1624);
    let record = serde_json::to_vec(&value).unwrap();
    read_identity(&record).unwrap();
    verify_record(&raw, &record).unwrap();
    let digest = value["bundle_sha256"].as_str().unwrap();
    verify_expected(&raw, digest).unwrap();
    let bundle: serde_json::Value = serde_json::from_slice(&raw).unwrap();
    assert_eq!(
        identity(&serde_json::to_vec(&bundle).unwrap()).unwrap(),
        value
    );
    assert_ne!(identity(&encode(&b, &a).unwrap()).unwrap(), value);
    assert!(verify_record(&encode(&a, &a).unwrap(), &record).is_err());
    for (key, v) in [
        ("schema_version", json!(true)),
        ("schema_version", json!(1.0)),
        ("schema_version", json!(2)),
        ("bytes", json!(0)),
        ("bytes", json!(32769)),
        ("bytes", json!(true)),
        ("bytes", json!(1.0)),
        ("extra", json!(1)),
        ("encoding", json!("jcs")),
        ("bundle_sha256", json!("A".repeat(64))),
    ] {
        let mut bad = value.clone();
        bad[key] = v;
        assert!(read_identity(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
    for key in value.as_object().unwrap().keys() {
        let mut bad = value.clone();
        bad.as_object_mut().unwrap().remove(key);
        assert!(read_identity(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
    let mut bad = value.clone();
    bad["bytes"] = json!(1);
    assert!(verify_record(&raw, &serde_json::to_vec(&bad).unwrap()).is_err());
    let duplicate = String::from_utf8(record.clone())
        .unwrap()
        .replacen("{", "{\"bytes\":1,", 1);
    assert!(read_identity(duplicate.as_bytes()).is_err());
    assert!(read_identity(&vec![b' '; 4097]).is_err());
    for bad in ["", "0", &"g".repeat(64), &format!("{digest}\n")] {
        assert!(verify_expected(&raw, bad).is_err());
    }
    assert_eq!(schema()["additionalProperties"], false);
    assert_eq!(schema()["properties"]["bytes"]["maximum"], 32768);
}

#[test]
fn cli_digest_gates_reject_ambiguity_before_output() {
    let d = tempfile::tempdir().unwrap();
    let (a, b) = pair();
    let raw = encode(&a, &b).unwrap();
    let record = identity(&raw).unwrap();
    let digest = record["bundle_sha256"].as_str().unwrap();
    std::fs::write(d.path().join("bundle"), &raw).unwrap();
    std::fs::write(
        d.path().join("identity"),
        serde_json::to_vec(&record).unwrap(),
    )
    .unwrap();
    let run = |args: &[&str]| {
        std::process::Command::new(env!("CARGO_BIN_EXE_toolcapsule"))
            .current_dir(d.path())
            .args(args)
            .output()
            .unwrap()
    };
    assert!(
        run(&[
            "configuration-bundle-verify",
            "bundle",
            "--expected-sha256",
            digest
        ])
        .status
        .success()
    );
    assert!(
        run(&[
            "configuration-bundle-render",
            "bundle",
            "--out",
            "html",
            "--expected-sha256",
            digest
        ])
        .status
        .success()
    );
    assert!(
        run(&[
            "configuration-bundle-verify-export",
            "bundle",
            "html",
            "--format",
            "html",
            "--expected-sha256",
            digest
        ])
        .status
        .success()
    );
    for base in [
        vec!["configuration-bundle-verify", "bundle"],
        vec!["configuration-bundle-render", "bundle", "--out", "blocked"],
        vec![
            "configuration-bundle-verify-export",
            "bundle",
            "html",
            "--format",
            "html",
        ],
    ] {
        let mut args = base.clone();
        args.extend(["--expected-sha256", digest, "--identity", "identity"]);
        assert_eq!(run(&args).status.code(), Some(2));
        let mut args = base;
        args.extend(["--expected-sha256", "invalid"]);
        assert_eq!(run(&args).status.code(), Some(2));
    }
    assert!(!d.path().join("blocked").exists());
    assert_eq!(std::fs::read(d.path().join("bundle")).unwrap(), raw);
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(
            &run(&["configuration-bundle-identity-schema"]).stdout
        )
        .unwrap(),
        schema()
    );
}

#[test]
fn verification_reports_only_checks_that_passed() {
    let d = tempfile::tempdir().unwrap();
    let (a, b) = pair();
    let raw = encode(&a, &b).unwrap();
    let record = identity(&raw).unwrap();
    let digest = record["bundle_sha256"].as_str().unwrap();

    for (name, bytes) in [
        ("bundle", raw.as_slice()),
        ("before", a.as_slice()),
        ("after", b.as_slice()),
    ] {
        std::fs::write(d.path().join(name), bytes).unwrap();
    }
    std::fs::write(
        d.path().join("identity"),
        serde_json::to_vec(&record).unwrap(),
    )
    .unwrap();
    for (name, sources) in [
        ("standalone", None),
        ("with-sources", Some((a.as_slice(), b.as_slice()))),
    ] {
        std::fs::write(
            d.path().join(name),
            toolcapsule::configuration_bundle::render(&raw, "html", sources).unwrap(),
        )
        .unwrap();
    }

    let run = |args: &[&str]| {
        std::process::Command::new(env!("CARGO_BIN_EXE_toolcapsule"))
            .current_dir(d.path())
            .args(args)
            .output()
            .unwrap()
    };
    let wrong_digest = "0".repeat(64);
    for (report_mode, export, kind) in [
        (false, "standalone", "configuration_bundle_export"),
        (true, "standalone", "configuration_bundle"),
    ] {
        let base = |export_name| {
            if report_mode {
                vec!["configuration-bundle-verify", "bundle"]
            } else {
                vec![
                    "configuration-bundle-verify-export",
                    "bundle",
                    export_name,
                    "--format",
                    "html",
                ]
            }
        };
        for source_checked in [false, true] {
            let export_name = if source_checked {
                "with-sources"
            } else {
                export
            };
            for identity_args in [
                &[][..],
                &["--expected-sha256", digest],
                &["--identity", "identity"],
            ] {
                let mut args = base(export_name);
                if source_checked {
                    args.extend(["--before", "before", "--after", "after"]);
                }
                args.extend_from_slice(identity_args);
                let output = run(&args);
                assert!(output.status.success(), "{args:?}");
                assert_eq!(
                    serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap(),
                    json!({
                        "schema_version": 1,
                        "kind": kind,
                        "verified": true,
                        "source_checked": source_checked,
                        "identity_checked": !identity_args.is_empty(),
                        "interpretation": "summary_consistency_not_source_identity_or_authority"
                    })
                );
            }
        }
        for extra in [
            &["--before", "before"][..],
            &["--before", "after", "--after", "before"],
            &["--expected-sha256", wrong_digest.as_str()],
        ] {
            let mut args = base(export);
            args.extend_from_slice(extra);
            let output = run(&args);
            assert_eq!(output.status.code(), Some(2), "{args:?}");
            assert!(output.stdout.is_empty(), "{args:?}");
        }
    }
    let output = run(&[
        "configuration-bundle-verify-export",
        "bundle",
        "standalone",
        "--format",
        "html",
        "--before",
        "before",
        "--after",
        "after",
    ]);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
}
