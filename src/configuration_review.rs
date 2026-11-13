//! Offline, script-free review of strictly validated configuration differences.
use crate::{
    Error,
    configuration_diff::{FIELDS, read_difference, verify_difference},
};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use sha2::{Digest, Sha256};

pub const MAX_HTML_BYTES: usize = 64 * 1024;
const STYLE: &str = "body{margin:0;background:#f4f6f8;color:#172b3a;font:16px/1.5 system-ui}main{max-width:1000px;margin:auto;padding:20px}h1{line-height:1.2}.table-wrap{overflow-x:auto}table{border-collapse:collapse;width:100%;min-width:640px;background:white}th,td{text-align:left;border-bottom:1px solid #ccd5dd;padding:12px;overflow-wrap:anywhere}caption{text-align:left;font-weight:bold;padding:12px 0}.changed{font-weight:bold}p{overflow-wrap:anywhere}@media(max-width:600px){main{padding:12px}th,td{padding:8px}}";

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// The optional pair checks independent summaries before any rendering.
pub fn render(raw: &[u8], sources: Option<(&[u8], &[u8])>) -> Result<String, Error> {
    let diff = read_difference(raw)?;
    if let Some((before, after)) = sources {
        verify_difference(raw, before, after)?;
    }
    let labels = [
        "Configuration valid",
        "Limits status",
        "Limits error",
        "Policy status",
        "Policy error",
        "Denies all tools",
        "Trust status",
        "Trust error",
        "Denies all signers",
        "Budget fit",
        "Exceeded budget fields",
    ];
    let mut rows = String::new();
    for (field, label) in FIELDS.iter().zip(labels) {
        let before = diff["before"].pointer(field);
        let after = diff["after"].pointer(field);
        let cell = |value: Option<&serde_json::Value>| {
            escape(&match value {
                None => "Not reported".to_owned(),
                Some(serde_json::Value::String(s)) => s.clone(),
                Some(other) => other.to_string(),
            })
        };
        let changed = before != after;
        rows.push_str(&format!(
            "<tr><th scope=\"row\">{}</th><td>{}</td><td>{}</td><td class=\"{}\">{}</td></tr>",
            escape(label),
            cell(before),
            cell(after),
            if changed { "changed" } else { "unchanged" },
            if changed { "Changed" } else { "Unchanged" }
        ));
    }
    let style_hash = STANDARD.encode(Sha256::digest(STYLE.as_bytes()));
    let checked = if sources.is_some() {
        "Matched explicit before/after summaries."
    } else {
        "Internal consistency checked; independent summaries not supplied."
    };
    let identical = if diff["identical_summaries"] == true {
        "No summary changes."
    } else {
        "Summary changes detected."
    };
    let html = format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; base-uri 'none'; form-action 'none'; style-src 'sha256-{style_hash}'\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>Configuration difference review</title><style>{STYLE}</style></head><body><main><h1>Configuration difference review</h1><p>Direction: before to after. {identical}</p><p>{checked}</p><p>Summaries do not establish source identity, grants or package admission. Invalid configuration can still be valid review evidence.</p><div class=\"table-wrap\" tabindex=\"0\" role=\"region\" aria-label=\"Configuration summary comparison\"><table><caption>Before and after configuration summaries</caption><thead><tr><th scope=\"col\">Field</th><th scope=\"col\">Before</th><th scope=\"col\">After</th><th scope=\"col\">Change</th></tr></thead><tbody>{rows}</tbody></table></div></main></body></html>\n"
    );
    if html.len() > MAX_HTML_BYTES {
        return Err(Error::InputLimit);
    }
    Ok(html)
}

/// Verify exact rendered bytes without parsing or repairing HTML.
pub fn verify(raw: &[u8], html: &[u8], sources: Option<(&[u8], &[u8])>) -> Result<(), Error> {
    if html.len() > MAX_HTML_BYTES {
        return Err(Error::InputLimit);
    }
    if html != render(raw, sources)?.as_bytes() {
        return Err(Error::Format);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn escapes_hostile_markup() {
        assert_eq!(
            super::escape("<script>\"'&</script>"),
            "&lt;script&gt;&quot;&#39;&amp;&lt;/script&gt;"
        );
    }
}
