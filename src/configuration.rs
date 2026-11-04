//! Read-only configuration checks and their public JSON contract.
use crate::{
    Error, Limits, Policy,
    storage::{read_file, read_json},
    trust_check,
};
use serde::{Deserialize, Serialize};
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

pub const MAX_RESULT_BYTES: usize = 4096;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
enum Diagnostic {
    InvalidFormat,
    InvalidConfiguration,
    InputTooLarge,
    IoError,
    ValidationFailed,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
enum LimitsCheck {
    Valid {},
    Invalid { error: Diagnostic },
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
enum PolicyCheck {
    Valid { denies_all_tools: bool },
    Invalid { error: Diagnostic },
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
enum TrustCheck {
    Valid { denies_all_signers: bool },
    Invalid { error: Diagnostic },
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct Checks {
    limits: LimitsCheck,
    policy: PolicyCheck,
    trust: TrustCheck,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
enum BudgetStatus {
    Fits,
    Exceeds,
    NotChecked,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct BudgetCheck {
    status: BudgetStatus,
    exceeded_fields: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct RawResult {
    schema_version: u8,
    kind: String,
    valid: bool,
    checks: Checks,
    budget_fit: BudgetCheck,
    package_admission: String,
}

/// A checked summary created only by the bounded reader, not package authority.
#[derive(Debug, Serialize, PartialEq)]
#[serde(transparent)]
pub struct ConfigurationResult(RawResult);

pub fn read_result(raw: &[u8]) -> Result<ConfigurationResult, Error> {
    if raw.len() > MAX_RESULT_BYTES {
        return Err(Error::InputLimit);
    }
    let report: RawResult = serde_json::from_slice(raw).map_err(|_| Error::Format)?;
    let limits_valid = matches!(report.checks.limits, LimitsCheck::Valid {});
    let policy_valid = matches!(report.checks.policy, PolicyCheck::Valid { .. });
    let trust_valid = matches!(report.checks.trust, TrustCheck::Valid { .. });
    let fields = [
        "fuel",
        "memory_bytes",
        "timeout_ms",
        "input_bytes",
        "output_bytes",
    ];
    let mut previous = None;
    for field in &report.budget_fit.exceeded_fields {
        let index = fields
            .iter()
            .position(|name| name == field)
            .ok_or(Error::Format)?;
        if previous.is_some_and(|p| p >= index) {
            return Err(Error::Format);
        }
        previous = Some(index);
    }
    let budget_valid = match report.budget_fit.status {
        BudgetStatus::NotChecked => !(limits_valid && policy_valid) && previous.is_none(),
        BudgetStatus::Fits => limits_valid && policy_valid && previous.is_none(),
        BudgetStatus::Exceeds => limits_valid && policy_valid && previous.is_some(),
    };
    if report.schema_version != 1
        || report.kind != "configuration"
        || report.package_admission != "not_checked"
        || report.valid != (limits_valid && policy_valid && trust_valid)
        || !budget_valid
    {
        return Err(Error::Format);
    }
    Ok(ConfigurationResult(report))
}

/// Equal summaries establish local consistency only, not identical source contents.
pub fn verify_result(raw: &[u8], limits: &Path, policy: &Path, trust: &Path) -> Result<(), Error> {
    let retained = read_result(raw)?;
    let (current, _) = check(limits, policy, trust);
    let retained = serde_json::to_value(retained).map_err(|_| Error::Format)?;
    if retained != current {
        return Err(Error::Format);
    }
    Ok(())
}
