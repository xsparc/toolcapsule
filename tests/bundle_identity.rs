use serde_json::json;
use toolcapsule::{
    bundle_identity::{identity, read_identity, schema, verify_expected, verify_record},
    bundle_verification::{self, MAX_RESPONSE_BYTES, VerificationKind, read_response},
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
                let checked = read_response(&output.stdout).unwrap();
                assert_eq!(
                    checked.kind,
                    if report_mode {
                        VerificationKind::ConfigurationBundle
                    } else {
                        VerificationKind::ConfigurationBundleExport
                    }
                );
                assert_eq!(checked.source_checked, source_checked);
                assert_eq!(checked.identity_checked, !identity_args.is_empty());
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

#[test]
fn verification_response_reader_is_closed_and_bounded() {
    let valid = json!({
        "schema_version": 1,
        "kind": "configuration_bundle",
        "verified": true,
        "source_checked": false,
        "identity_checked": true,
        "interpretation": "summary_consistency_not_source_identity_or_authority"
    });
    let encode = |value: &serde_json::Value| serde_json::to_vec(value).unwrap();
    let raw = encode(&valid);
    assert!(read_response(&raw).is_ok());
    let schema = bundle_verification::schema();
    assert_eq!(schema["additionalProperties"], false);
    let properties = &schema["properties"];
    assert_eq!(properties["schema_version"]["const"], 1);
    assert_eq!(properties["schema_version"]["type"], "integer");
    assert_eq!(properties["verified"]["const"], true);
    assert_eq!(properties["source_checked"]["type"], "boolean");
    assert_eq!(properties["identity_checked"]["type"], "boolean");
    assert_eq!(
        properties["kind"]["enum"],
        json!(["configuration_bundle", "configuration_bundle_export"])
    );
    assert_eq!(
        properties["interpretation"]["const"],
        valid["interpretation"]
    );
    let required = schema["required"].as_array().unwrap();
    assert_eq!(required.len(), valid.as_object().unwrap().len());
    for field in valid.as_object().unwrap().keys() {
        assert!(required.contains(&json!(field)));
        let mut missing = valid.clone();
        missing.as_object_mut().unwrap().remove(field);
        assert!(read_response(&encode(&missing)).is_err(), "{field}");
        let duplicate = format!(
            "{{{}:{},{}",
            json!(field),
            valid[field],
            std::str::from_utf8(&raw[1..]).unwrap()
        );
        assert!(read_response(duplicate.as_bytes()).is_err(), "{field}");
    }
    for (field, value) in [
        ("schema_version", json!(2)),
        ("schema_version", json!(1.0)),
        ("verified", json!(false)),
        ("kind", json!("configuration_bundle_identity")),
        ("identity_checked", json!("true")),
        ("source_checked", json!(null)),
        ("interpretation", json!("trusted_producer")),
        ("authoritative", json!(true)),
    ] {
        let mut malformed = valid.clone();
        malformed[field] = value;
        assert!(read_response(&encode(&malformed)).is_err(), "{field}");
    }
    let mut padded = raw;
    padded.resize(MAX_RESPONSE_BYTES, b' ');
    assert!(read_response(&padded).is_ok());
    padded.push(b' ');
    assert!(matches!(
        read_response(&padded),
        Err(toolcapsule::Error::InputLimit)
    ));
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_toolcapsule"))
        .arg("configuration-bundle-verification-schema")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap(),
        schema,
    );
}

#[test]
fn cli_checks_saved_response_format_without_echoing_evidence() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("private-fixture-response.json");
    let check = || {
        std::process::Command::new(env!("CARGO_BIN_EXE_toolcapsule"))
            .arg("configuration-bundle-verification-check")
            .arg(&path)
            .output()
            .unwrap()
    };
    let interpretation = "summary_consistency_not_source_identity_or_authority";
    let record = json!({
        "schema_version": 1,
        "kind": "configuration_bundle",
        "verified": true,
        "source_checked": true,
        "identity_checked": true,
        "interpretation": interpretation
    });
    let mut valid = format!(" \n\r\t{record}").into_bytes();
    valid.resize(MAX_RESPONSE_BYTES, b' ');
    std::fs::write(&path, &valid).unwrap();
    let result = check();
    assert_eq!(result.status.code(), Some(0));
    assert!(result.stderr.is_empty());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&result.stdout).unwrap(),
        json!({
            "schema_version": 1,
            "valid": true,
            "interpretation": "verification_response_validity_only"
        })
    );
    assert_eq!(std::fs::read(&path).unwrap(), valid);

    let positional = json!([1, true, interpretation, "configuration_bundle", true, true]);
    assert!(matches!(
        read_response(&serde_json::to_vec(&positional).unwrap()),
        Err(toolcapsule::Error::Format)
    ));
    let duplicate = format!("{{\"verified\":false,{}", &record.to_string()[1..]);
    valid.push(b' ');
    for invalid in [
        serde_json::to_vec(&positional).unwrap(),
        duplicate.into_bytes(),
        br#"{"unknown":"private-fixture-value"}"#.to_vec(),
        valid,
    ] {
        std::fs::write(&path, &invalid).unwrap();
        let result = check();
        assert_eq!(result.status.code(), Some(2));
        assert!(result.stdout.is_empty());
        assert_eq!(std::fs::read(&path).unwrap(), invalid);
        let diagnostic: serde_json::Value = serde_json::from_slice(&result.stderr).unwrap();
        assert_eq!(
            diagnostic["kind"],
            "configuration_bundle_verification_response"
        );
        assert_eq!(diagnostic.as_object().unwrap().len(), 3);
        let stderr = String::from_utf8(result.stderr).unwrap();
        assert!(!stderr.contains("private-fixture"));
    }
}

#[test]
fn cli_batch_counts_formats_without_echoing_files_or_flags() {
    let d = tempfile::tempdir().unwrap();
    let run = |args: &[&str]| {
        std::process::Command::new(env!("CARGO_BIN_EXE_toolcapsule"))
            .current_dir(d.path())
            .arg("configuration-bundle-verification-batch")
            .args(args)
            .output()
            .unwrap()
    };
    let valid = serde_json::to_vec(&json!({
        "schema_version": 1,
        "kind": "configuration_bundle_export",
        "verified": true,
        "source_checked": true,
        "identity_checked": false,
        "interpretation": "summary_consistency_not_source_identity_or_authority"
    }))
    .unwrap();
    let malformed = br#"{"unknown":"synthetic-private-marker"}"#;
    let oversized = vec![b' '; MAX_RESPONSE_BYTES + 1];
    for (path, bytes) in [
        ("private-valid", valid.as_slice()),
        ("private-invalid", malformed.as_slice()),
        ("private-large", oversized.as_slice()),
    ] {
        std::fs::write(d.path().join(path), bytes).unwrap();
    }
    std::fs::create_dir(d.path().join("private-dir")).unwrap();
    let result = run(&[
        "private-valid",
        "private-invalid",
        "private-large",
        "private-missing",
        "private-dir",
        "private-valid",
    ]);
    assert_eq!(result.status.code(), Some(2));
    assert!(result.stderr.is_empty());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&result.stdout).unwrap(),
        json!({
            "schema_version": 1,
            "format_valid": 2,
            "invalid_format": 1,
            "oversized": 1,
            "unreadable": 2,
            "total": 6,
            "interpretation": "verification_response_validity_counts_only"
        })
    );
    assert_eq!(
        std::fs::read(d.path().join("private-valid")).unwrap(),
        valid
    );
    assert_eq!(
        std::fs::read(d.path().join("private-invalid")).unwrap(),
        malformed
    );
    assert_eq!(
        std::fs::read(d.path().join("private-large")).unwrap(),
        oversized
    );
    let limit = run(&["private-valid"; 32]);
    assert_eq!(limit.status.code(), Some(0));
    assert!(limit.stderr.is_empty());
    let report: serde_json::Value = serde_json::from_slice(&limit.stdout).unwrap();
    assert_eq!(report["format_valid"], 32);
    assert_eq!(report["total"], 32);
    let above_limit = run(&["private-missing"; 33]);
    assert_eq!(above_limit.status.code(), Some(2));
    assert!(above_limit.stdout.is_empty());
    let diagnostic: serde_json::Value = serde_json::from_slice(&above_limit.stderr).unwrap();
    assert_eq!(diagnostic["error"], "input_too_large");
    assert!(
        !String::from_utf8(above_limit.stderr)
            .unwrap()
            .contains("private-")
    );
}

#[cfg(unix)]
#[test]
fn cli_batch_does_not_read_final_symlink() {
    let d = tempfile::tempdir().unwrap();
    std::fs::write(d.path().join("another-record"), b"{}").unwrap();
    std::os::unix::fs::symlink("another-record", d.path().join("selected-link")).unwrap();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_toolcapsule"))
        .current_dir(d.path())
        .args(["configuration-bundle-verification-batch", "selected-link"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    let counts: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(counts["unreadable"], 1);
    assert_eq!(counts["invalid_format"], 0);
}

#[test]
fn cli_checks_generated_batch_receipts_without_rechecking_inputs() {
    let d = tempfile::tempdir().unwrap();
    let run = |args: &[&str]| {
        std::process::Command::new(env!("CARGO_BIN_EXE_toolcapsule"))
            .current_dir(d.path())
            .args(args)
            .output()
            .unwrap()
    };
    std::fs::write(d.path().join("synthetic-invalid"), b"{}").unwrap();
    let batch = run(&[
        "configuration-bundle-verification-batch",
        "synthetic-invalid",
        "synthetic-missing",
    ]);
    assert_eq!(batch.status.code(), Some(2));
    let valid: serde_json::Value = serde_json::from_slice(&batch.stdout).unwrap();
    assert_eq!(valid["format_valid"], 0);
    assert_eq!(valid["total"], 2);
    std::fs::remove_file(d.path().join("synthetic-invalid")).unwrap();
    let path = d.path().join("synthetic-private-receipt");
    let check = || {
        run(&[
            "configuration-bundle-verification-batch-check",
            "synthetic-private-receipt",
        ])
    };
    std::fs::write(&path, &batch.stdout).unwrap();
    let ok = check();
    assert_eq!(ok.status.code(), Some(0));
    assert!(ok.stderr.is_empty());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&ok.stdout).unwrap(),
        json!({"schema_version":1,"valid":true,"interpretation":"verification_batch_receipt_validity_only"})
    );
    assert_eq!(std::fs::read(&path).unwrap(), batch.stdout);

    let schema = run(&["configuration-bundle-verification-batch-schema"]);
    assert_eq!(schema.status.code(), Some(0));
    let schema: serde_json::Value = serde_json::from_slice(&schema.stdout).unwrap();
    assert_eq!(schema, bundle_verification::batch_schema());
    assert_eq!(schema["additionalProperties"], false);
    assert_eq!(
        schema["properties"]["total"]["maximum"],
        bundle_verification::MAX_BATCH_RESPONSES
    );
    let required = schema["required"].as_array().unwrap();
    assert_eq!(required.len(), valid.as_object().unwrap().len());
    assert!(
        valid
            .as_object()
            .unwrap()
            .keys()
            .all(|key| required.contains(&json!(key)))
    );

    for (field, value) in [
        ("total", json!(3)),
        ("total", json!(0)),
        ("total", json!(33)),
        ("schema_version", json!(2)),
        ("invalid_format", json!(true)),
        ("unreadable", json!(1.0)),
        ("authority", json!("synthetic-private-value")),
    ] {
        let mut wrong = valid.clone();
        wrong[field] = value;
        let bytes = serde_json::to_vec(&wrong).unwrap();
        std::fs::write(&path, &bytes).unwrap();
        let bad = check();
        assert_eq!(bad.status.code(), Some(2), "{field}");
        assert!(bad.stdout.is_empty(), "{field}");
        assert!(
            !String::from_utf8(bad.stderr)
                .unwrap()
                .contains("synthetic-private")
        );
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
    }
    let mut missing = valid.clone();
    missing.as_object_mut().unwrap().remove("unreadable");
    let duplicate = format!("{{\"total\":2,{}", &valid.to_string()[1..]);
    for bytes in [
        serde_json::to_vec(&missing).unwrap(),
        duplicate.into_bytes(),
        b"[]".to_vec(),
        vec![b' '; MAX_RESPONSE_BYTES + 1],
    ] {
        std::fs::write(&path, bytes).unwrap();
        let bad = check();
        assert_eq!(bad.status.code(), Some(2));
        assert!(bad.stdout.is_empty());
    }
}

#[test]
fn cli_batch_check_rejects_nonregular_receipt_path() {
    let d = tempfile::tempdir().unwrap();
    std::fs::create_dir(d.path().join("directory-receipt")).unwrap();
    let mut invalid = vec!["directory-receipt"];
    #[cfg(unix)]
    {
        std::fs::write(d.path().join("receipt-data"), b"{}").unwrap();
        std::os::unix::fs::symlink("receipt-data", d.path().join("symlink-receipt")).unwrap();
        invalid.push("symlink-receipt");
    }
    for name in invalid.drain(..) {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_toolcapsule"))
            .current_dir(d.path())
            .args(["configuration-bundle-verification-batch-check", name])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        let diagnostic: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
        assert_eq!(
            diagnostic,
            json!({
                "schema_version":1,
                "kind":"configuration_bundle_verification_batch_receipt",
                "error":"io_error"
            })
        );
    }
}
