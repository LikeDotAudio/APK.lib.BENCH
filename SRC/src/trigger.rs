// Part of the APK.audio project — http://APK.audio — made by Anthony Kuzub
// MIT Licence. Free, for everyone, for ever. Full text in LICENSE at the root.
//! Is this GUI press a "go"?

/// Truthy scan trigger: the GUI envelope `{"value":1,...}`, or a bare
/// `1` / `true` / `scan`.
///
/// Was `is_truthy_trigger` in the orchestrator's `main.rs`, read by the VISA
/// write daemon (Rescan, Clear) and by the Discovered mirror (Clear). The two
/// halves now run in different processes and must still agree on what a press
/// is, so the rule is here once.
pub fn is_truthy(payload: &str) -> bool {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(payload) {
        return match v.get("value").unwrap_or(&v) {
            serde_json::Value::Number(n) => n.as_f64().unwrap_or(0.0) != 0.0,
            serde_json::Value::Bool(b) => *b,
            serde_json::Value::String(s) => s == "1" || s.eq_ignore_ascii_case("true") || s.eq_ignore_ascii_case("scan"),
            _ => false,
        };
    }
    payload == "1" || payload.eq_ignore_ascii_case("true") || payload.eq_ignore_ascii_case("scan")
}

#[cfg(test)]
mod tests {
    use super::is_truthy;

    #[test]
    fn the_envelope_and_the_bare_word_both_count() {
        assert!(is_truthy(r#"{"value":1}"#));
        assert!(is_truthy(r#"{"value":"scan"}"#));
        assert!(is_truthy("1"));
        assert!(is_truthy("TRUE"));
        assert!(!is_truthy(r#"{"value":0}"#), "the button at rest is not a press");
        assert!(!is_truthy(""));
        assert!(!is_truthy("0"));
    }
}
