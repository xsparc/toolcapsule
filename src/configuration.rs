//! Read-only configuration checks and their public JSON contract.
use crate::{
    Error, Limits, Policy,
    storage::{read_file, read_json},
    trust_check,
};
use serde_json::{Value, json};
use std::path::Path;

pub fn error_code(error: &Error) -> &'static str {
    match error {
        Error::Format => "invalid_format",
        Error::Policy => "invalid_configuration",
        Error::InputLimit => "input_too_large",
        Error::Io => "io_error",
        _ => "validation_failed",
    }
}

fn result_summary<T>(result: &Result<T, Error>, success: impl FnOnce(&T) -> Value) -> Value {
    match result {
        Ok(value) => success(value),
        Err(error) => json!({"status": "invalid", "error": error_code(error)}),
    }
}

/// Check every input even if another fails. Invalid inputs take precedence over non-fit.
pub fn check(limits: &Path, policy: &Path, trust: &Path) -> (Value, u8) {
    let limits = read_json::<Limits>(limits, 4096).and_then(|v| v.validate().map(|()| v));
    let policy = read_json::<Policy>(policy, 64 * 1024).and_then(|v| v.validate().map(|()| v));
    let trust =
        read_file(trust, trust_check::MAX_TRUST_BYTES).and_then(|v| trust_check::check_trust(&v));
    let valid = limits.is_ok() && policy.is_ok() && trust.is_ok();
    let (budget, fits) = match (&limits, &policy) {
        (Ok(limits), Ok(policy)) => {
            // Both shapes have already passed their independent validation.
            match limits.exceeded_fields(&policy.max_limits) {
                Ok(fields) => {
                    let fits = fields.is_empty();
                    (
                        json!({"status": if fits {"fits"} else {"exceeds"}, "exceeded_fields": fields}),
                        fits,
                    )
                }
                Err(_) => (json!({"status":"not_checked","exceeded_fields":[]}), false),
            }
        }
        _ => (json!({"status":"not_checked","exceeded_fields":[]}), false),
    };
    let report = json!({
        "schema_version": 1, "kind": "configuration", "valid": valid,
        "checks": {
            "limits": result_summary(&limits, |_| json!({"status":"valid"})),
            "policy": result_summary(&policy, |p| json!({"status":"valid", "denies_all_tools":p.allowed_tools.is_empty()})),
            "trust": result_summary(&trust, |n| json!({"status":"valid", "denies_all_signers": *n == 0})),
        },
        "budget_fit": budget, "package_admission":"not_checked"
    });
    (
        report,
        if !valid {
            2
        } else if fits {
            0
        } else {
            1
        },
    )
}

pub fn schema() -> Value {
    serde_json::from_str(include_str!("../schemas/validation.schema.json"))
        .expect("checked-in validation schema must be valid JSON")
}
