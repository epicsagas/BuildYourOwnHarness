//! Security — secret masking (R20). Mirrors korean-law-rag's `OC=****` pattern
//! and applies to every log line and rendered bundle artifact.

use regex::Regex;

/// Patterns that must never appear in plaintext in logs/output. Captures the
/// key name + redacts the value.
const SECRET_PATTERNS: &[&str] = &[
    // OC API keys (법제처 / korean-law-rag style): LAW_OC=..., OC=..., OC_KEY=...
    // Identifier is [A-Za-z0-9_] and contains OC; followed by = or : then the value.
    r#"(?i)((?:^|[\s,;'"])[A-Z0-9_]*OC[A-Z0-9_]*\s*[:=]\s*)([^\s,;'"]+)"#,
    // Generic bearer / api key prefixes.
    r"(?i)(bearer\s+)([A-Za-z0-9._\-]{8,})",
    r#"(?i)(api[_-]?key\s*[:=]\s*)([^\s,;'"]+)"#,
    r"(?i)(sk-[A-Za-z0-9]{8,})",
];

/// Mask all secret patterns in `input`, replacing values with `****`.
///
/// The key/label is preserved so the log remains useful: `OC=secret123` →
/// `OC=****`. Pure function — deterministic, allocation-only.
pub fn mask(input: &str) -> String {
    let mut out = input.to_string();
    for pat in SECRET_PATTERNS {
        let re = match Regex::new(pat) {
            Ok(r) => r,
            Err(_) => continue,
        };
        // For patterns with 2 capture groups (label + value), keep label.
        out = re
            .replace_all(&out, |caps: &regex::Captures<'_>| {
                if let (Some(label), Some(_val)) = (caps.get(1), caps.get(2)) {
                    format!("{}****", label.as_str())
                } else if let Some(full) = caps.get(0) {
                    // single-group patterns like sk-...
                    full.as_str().replace(|c: char| c.is_alphanumeric(), "*")
                } else {
                    "****".to_string()
                }
            })
            .to_string();
    }
    out
}

/// Assert that `input` contains no recognizable secret (test helper / guard).
pub fn assert_no_secrets(input: &str) -> Result<(), String> {
    for pat in SECRET_PATTERNS {
        if let Ok(re) = Regex::new(pat) {
            if re.is_match(input) {
                return Err(format!("possible secret matched pattern {pat:?}"));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masks_oc_key() {
        // AC19: OC=secret123 → OC=****
        assert_eq!(mask("LAW_OC=secret123 end"), "LAW_OC=**** end");
        assert_eq!(mask("OC=abc123"), "OC=****");
        assert_eq!(mask("OC_KEY=mykey"), "OC_KEY=****");
    }

    #[test]
    fn masks_bearer_and_apikey() {
        assert_eq!(mask("bearer abcdefgh1234"), "bearer ****");
        assert_eq!(mask("api_key=topsecret"), "api_key=****");
        assert_eq!(mask("API-KEY=topsecret"), "API-KEY=****");
    }

    #[test]
    fn masks_sk_prefix() {
        let m = mask("token: sk-1234567890abc");
        assert!(!m.contains("1234567890abc"));
    }

    #[test]
    fn leaves_plain_text_alone() {
        assert_eq!(
            mask("no secrets here, just code"),
            "no secrets here, just code"
        );
    }

    #[test]
    fn assert_no_secrets_passes_clean() {
        assert!(assert_no_secrets("just a normal log line").is_ok());
    }

    #[test]
    fn assert_no_secrets_catches_oc() {
        assert!(assert_no_secrets("OC=secret").is_err());
    }
}
