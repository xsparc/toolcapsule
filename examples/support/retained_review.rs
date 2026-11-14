//! Synthetic CLI acceptance. Only non-executing review commands are admitted.
use serde_json::json;
use std::{
    error::Error,
    fs, io,
    path::Path,
    process::{Command, Output},
};
type Result<T> = std::result::Result<T, Box<dyn Error>>;
fn require(ok: bool) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(io::Error::other("review acceptance failed").into())
    }
}
fn call(binary: &Path, dir: &Path, args: &[&str]) -> Result<Output> {
    require(matches!(
        args.first().copied(),
        Some(
            "configuration-diff"
                | "configuration-diff-identity"
                | "configuration-diff-check"
                | "configuration-diff-html"
                | "configuration-diff-csv"
                | "configuration-diff-html-verify"
                | "configuration-diff-csv-verify"
        )
    ))?;
    Ok(Command::new(binary).current_dir(dir).args(args).output()?)
}
pub fn run(binary: &Path, dir: &Path) -> Result<()> {
    let binary = binary.canonicalize()?;
    fs::create_dir(dir)?;
    let a = json!({"schema_version":1,"kind":"configuration","valid":true,"checks":{"limits":{"status":"valid"},"policy":{"status":"valid","denies_all_tools":true},"trust":{"status":"valid","denies_all_signers":true}},"budget_fit":{"status":"fits","exceeded_fields":[]},"package_admission":"not_checked"});
    let mut b = a.clone();
    b["budget_fit"] = json!({"status":"exceeds","exceeded_fields":["fuel"]});
    for (name, value) in [("before.json", a), ("after.json", b)] {
        fs::write(dir.join(name), serde_json::to_vec_pretty(&value)?)?;
    }
    let cli = |args: &[&str]| call(&binary, dir, args);
    require(
        cli(&[
            "configuration-diff",
            "before.json",
            "after.json",
            "--out",
            "difference.json",
        ])?
        .status
        .success(),
    )?;
    let identity_args = [
        "configuration-diff-identity",
        "difference.json",
        "--out",
        "identity.json",
    ];
    let exported = cli(&identity_args)?;
    require(exported.status.success() && exported.stdout.is_empty())?;
    let stdout = cli(&["configuration-diff-identity", "difference.json"])?;
    require(stdout.status.success())?;
    require(
        serde_json::from_slice::<serde_json::Value>(&stdout.stdout)?
            == serde_json::from_slice::<serde_json::Value>(&fs::read(dir.join("identity.json"))?)?,
    )?;
    let mut saved = Vec::new();
    for name in [
        "before.json",
        "after.json",
        "difference.json",
        "identity.json",
    ] {
        saved.push((name, fs::read(dir.join(name))?));
    }
    require(cli(&identity_args)?.status.code() == Some(2))?;
    for (export, verify, file) in [
        (
            "configuration-diff-html",
            "configuration-diff-html-verify",
            "review.html",
        ),
        (
            "configuration-diff-csv",
            "configuration-diff-csv-verify",
            "review.csv",
        ),
    ] {
        let args = [
            export,
            "difference.json",
            "--out",
            file,
            "--identity",
            "identity.json",
            "--before",
            "before.json",
            "--after",
            "after.json",
        ];
        require(cli(&args)?.status.success())?;
        let raw = fs::read(dir.join(file))?;
        require(cli(&args)?.status.code() == Some(2))?;
        require(
            cli(&[
                verify,
                "difference.json",
                file,
                "--identity",
                "identity.json",
                "--before",
                "before.json",
                "--after",
                "after.json",
            ])?
            .status
            .success(),
        )?;
        require(raw == fs::read(dir.join(file))?)?;
        saved.push((file, raw));
    }
    require(
        cli(&[
            "configuration-diff",
            "before.json",
            "before.json",
            "--out",
            "replacement.json",
        ])?
        .status
        .success(),
    )?;
    require(
        cli(&[
            "configuration-diff-identity",
            "replacement.json",
            "--out",
            "replacement-id.json",
        ])?
        .status
        .success(),
    )?;
    require(
        cli(&[
            "configuration-diff-check",
            "replacement.json",
            "--identity",
            "replacement-id.json",
        ])?
        .status
        .success(),
    )?;
    for (export, verify, file) in [
        (
            "configuration-diff-html",
            "configuration-diff-html-verify",
            "replacement.html",
        ),
        (
            "configuration-diff-csv",
            "configuration-diff-csv-verify",
            "replacement.csv",
        ),
    ] {
        require(
            cli(&[
                export,
                "replacement.json",
                "--out",
                file,
                "--identity",
                "replacement-id.json",
                "--before",
                "before.json",
                "--after",
                "before.json",
            ])?
            .status
            .success(),
        )?;
        require(
            cli(&[
                verify,
                "replacement.json",
                file,
                "--identity",
                "replacement-id.json",
                "--before",
                "before.json",
                "--after",
                "after.json",
            ])?
            .status
            .code()
                == Some(2),
        )?;
        require(
            cli(&[
                export,
                "replacement.json",
                "--out",
                "blocked",
                "--identity",
                "replacement-id.json",
                "--before",
                "before.json",
                "--after",
                "after.json",
            ])?
            .status
            .code()
                == Some(2),
        )?;
        require(!dir.join("blocked").exists())?;
    }
    fs::write(dir.join("invalid.json"), b"{}")?;
    require(
        cli(&[
            "configuration-diff-identity",
            "invalid.json",
            "--out",
            "invalid-id.json",
        ])?
        .status
        .code()
            == Some(2),
    )?;
    require(!dir.join("invalid-id.json").exists())?;
    for (name, raw) in saved {
        require(fs::read(dir.join(name))? == raw)?;
    }
    Ok(())
}
