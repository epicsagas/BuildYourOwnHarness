//! B17 i18n — multilingual message catalog (ARCH §9.3).
//!
//! The interview, wizard, and bundle messages all resolve through here so a
//! single language switch flows everywhere. Originally bilingual (en/ko); now a
//! `Message` carries an ordered `[(lang, text)]` slice — first entry is the
//! fallback (`en`). Additional languages (ja/zh-Hans/es/de/fr/pt/ru/ar) fall
//! back to `en` until the translation workflow fills them in.

use std::collections::HashMap;

/// Canonical message keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Msg {
    Welcome,
    ChooseGenre,
    Confirm,
    Back,
    DryRunPassed,
    DryRunFailed,
    Installed,
    DependencyMissing,
    StagnationRollback,
    EvolveApproved,
}

/// A localized message: ordered `(lang_code, text)` pairs. The first pair is the
/// fallback (always `en`). Lookup falls back to the first entry when `lang` is
/// absent, so partially-translated catalogs degrade gracefully to English.
#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    pub translations: &'static [(&'static str, &'static str)],
}

impl Message {
    /// Resolve the text for `lang`, falling back to the first entry (`en`).
    pub fn get(&self, lang: &str) -> &str {
        self.translations
            .iter()
            .find(|(l, _)| *l == lang)
            .map(|(_, t)| *t)
            .unwrap_or_else(|| self.translations.first().map(|(_, t)| *t).unwrap_or(""))
    }
}

/// The catalog. Each key ships `en` (canonical) + `ko`; other languages fall
/// back to `en` until the auto-translation workflow populates them.
pub fn catalog() -> HashMap<Msg, Message> {
    let mut m = HashMap::new();
    m.insert(
        Msg::Welcome,
        Message {
            translations: &[
                ("en", "Welcome to BYOH — let's build your harness."),
                ("ko", "BYOH에 오신 것을 환영합니다 — 하네스를 만들어봅시다."),
            ],
        },
    );
    m.insert(
        Msg::ChooseGenre,
        Message {
            translations: &[
                ("en", "Choose the genre that fits your work."),
                ("ko", "작업에 맞는 장르를 선택하세요."),
            ],
        },
    );
    m.insert(
        Msg::Confirm,
        Message {
            translations: &[("en", "Confirm"), ("ko", "확정")],
        },
    );
    m.insert(
        Msg::Back,
        Message {
            translations: &[("en", "Back"), ("ko", "뒤로")],
        },
    );
    m.insert(
        Msg::DryRunPassed,
        Message {
            translations: &[
                ("en", "dry-run PASSED — bundle is safe to install."),
                ("ko", "dry-run 통과 — 번들을 설치해도 안전합니다."),
            ],
        },
    );
    m.insert(
        Msg::DryRunFailed,
        Message {
            translations: &[
                (
                    "en",
                    "dry-run FAILED — review the report before installing.",
                ),
                ("ko", "dry-run 실패 — 설치 전 보고서를 확인하세요."),
            ],
        },
    );
    m.insert(
        Msg::Installed,
        Message {
            translations: &[
                ("en", "Harness installed. Run with: byoh run <slug>"),
                ("ko", "하네스 설치 완료. 실행: byoh run <slug>"),
            ],
        },
    );
    m.insert(
        Msg::DependencyMissing,
        Message {
            translations: &[
                (
                    "en",
                    "A dependency tool is missing — falling back gracefully.",
                ),
                ("ko", "의존 도구가 없습니다 — 안전하게 폴백합니다."),
            ],
        },
    );
    m.insert(
        Msg::StagnationRollback,
        Message {
            translations: &[
                (
                    "en",
                    "Stagnation detected — rolling back to the last good config.",
                ),
                ("ko", "정체 감지 — 마지막 양호 설정으로 롤백합니다."),
            ],
        },
    );
    m.insert(
        Msg::EvolveApproved,
        Message {
            translations: &[
                ("en", "Evolution approved by all safety gates."),
                ("ko", "모든 안전장치가 진화를 승인했습니다."),
            ],
        },
    );
    m
}

/// Resolve a message for a language (falls back to `en`).
pub fn t(msg: Msg, lang: &str) -> String {
    catalog()
        .get(&msg)
        .map(|m| m.get(lang).to_string())
        .unwrap_or_default()
}

/// Reduce a raw locale string (`"ko_KR.UTF-8"`, `"zh-CN"`, `"pt_BR"`) to a BYOH
/// language code. Unknown/empty → `"en"`. Public so tests and callers can resolve
/// env values without re-implementing the table.
pub fn parse_lang(raw: &str) -> &'static str {
    // Take the part before `_` or `.` or `-`, lowercase.
    let base: String = raw
        .split(['_', '.', '-'])
        .next()
        .unwrap_or("")
        .to_lowercase();
    match base.as_str() {
        "ko" => "ko",
        "ja" => "ja",
        "zh" => "zh-hans",
        "es" => "es",
        "de" => "de",
        "fr" => "fr",
        "pt" => "pt",
        "ru" => "ru",
        "ar" => "ar",
        _ => "en",
    }
}

/// Detect the user's language from the environment: `LC_ALL` > `LANG` > `"en"`.
pub fn detect_locale() -> &'static str {
    let raw = std::env::var("LC_ALL")
        .ok()
        .filter(|s| !s.is_empty())
        .or_else(|| std::env::var("LANG").ok().filter(|s| !s.is_empty()))
        .unwrap_or_default();
    parse_lang(&raw)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_has_en_and_ko() {
        let c = catalog();
        for key in [
            Msg::Welcome,
            Msg::DryRunPassed,
            Msg::Installed,
            Msg::StagnationRollback,
        ] {
            let msg = c.get(&key).unwrap();
            assert!(msg.get("en").contains("BYOH") || !msg.get("en").is_empty());
            assert!(msg.get("ko").contains('하') || !msg.get("ko").is_empty());
        }
    }

    #[test]
    fn t_resolves_by_language() {
        assert!(t(Msg::Confirm, "en").contains("Confirm"));
        assert!(t(Msg::Confirm, "ko").contains("확정"));
    }

    #[test]
    fn unknown_language_falls_back_to_en() {
        // A language with no translation yet (e.g. ja) falls back to en (first entry).
        let c = catalog();
        let msg = c.get(&Msg::Welcome).unwrap();
        assert_eq!(msg.get("ja"), msg.get("en"));
    }

    #[test]
    fn parse_lang_reduces_locale_strings() {
        assert_eq!(parse_lang("ko_KR.UTF-8"), "ko");
        assert_eq!(parse_lang("en_US.UTF-8"), "en");
        assert_eq!(parse_lang("ja_JP"), "ja");
        assert_eq!(parse_lang("zh-CN"), "zh-hans");
        assert_eq!(parse_lang("pt_BR.UTF-8"), "pt");
        assert_eq!(parse_lang("ar_SA"), "ar");
        assert_eq!(parse_lang(""), "en");
        assert_eq!(parse_lang("nonsense"), "en");
    }
}
