//! Canonical UTF-8 CSV for validated configuration differences.
use crate::{
    Error,
    configuration_diff::{FIELDS, read_difference, verify_difference},
};
use serde_json::Value;
pub const MAX_CSV_BYTES: usize = 64 * 1024;

fn quoted(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}
fn text(value: &str) -> String {
    quoted(&format!("text:{value}"))
}
fn cell(value: Option<&Value>) -> String {
    text(
        &value
            .map(Value::to_string)
            .unwrap_or_else(|| "not_reported".into()),
    )
}

/// Quote every cell and prefix data text so spreadsheet formula markers are not leading.
pub fn render(raw: &[u8], sources: Option<(&[u8], &[u8])>) -> Result<Vec<u8>, Error> {
    let diff = read_difference(raw)?;
    if let Some((before, after)) = sources {
        verify_difference(raw, before, after)?;
    }
    let mut csv = [
        "schema_version",
        "direction",
        "field",
        "before",
        "after",
        "change",
        "interpretation",
    ]
    .map(quoted)
    .join(",")
        + "\r\n";
    for field in FIELDS {
        let before = diff["before"].pointer(field);
        let after = diff["after"].pointer(field);
        csv.push_str(
            &[
                quoted("1"),
                text("before_to_after"),
                text(field),
                cell(before),
                cell(after),
                text(if before == after {
                    "unchanged"
                } else {
                    "changed"
                }),
                text("summary_changes_not_source_identity_or_authority"),
            ]
            .join(","),
        );
        csv.push_str("\r\n");
    }
    if csv.len() > MAX_CSV_BYTES {
        return Err(Error::InputLimit);
    }
    Ok(csv.into_bytes())
}

/// Exact bytes only: no newline normalization, decoding repairs or writes.
pub fn verify(raw: &[u8], csv: &[u8], sources: Option<(&[u8], &[u8])>) -> Result<(), Error> {
    if csv.len() > MAX_CSV_BYTES {
        return Err(Error::InputLimit);
    }
    if csv != render(raw, sources)? {
        return Err(Error::Format);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn formula_text_and_quotes_are_literal() {
        assert_eq!(super::text("=SUM(1,2)\""), "\"text:=SUM(1,2)\"\"\"");
    }
}
