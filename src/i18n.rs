//! B17 bilingual i18n — ko/en message catalog (ARCH §9.3).
//!
//! The interview, wizard, and bundle messages all resolve through here so a
//! single language switch flows everywhere.

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

/// A bilingual string.
#[derive(Debug, Clone, PartialEq)]
pub struct Bilingual {
    pub en: String,
    pub ko: String,
}

impl Bilingual {
    pub fn get(&self, lang: &str) -> &str {
        if lang == "ko" {
            &self.ko
        } else {
            &self.en
        }
    }
}

/// The catalog.
pub fn catalog() -> HashMap<Msg, Bilingual> {
    let mut m = HashMap::new();
    m.insert(
        Msg::Welcome,
        Bilingual {
            en: "Welcome to BYOH — let's build your harness.".into(),
            ko: "BYOH에 오신 것을 환영합니다 — 하네스를 만들어봅시다.".into(),
        },
    );
    m.insert(
        Msg::ChooseGenre,
        Bilingual {
            en: "Choose the genre that fits your work.".into(),
            ko: "작업에 맞는 장르를 선택하세요.".into(),
        },
    );
    m.insert(
        Msg::Confirm,
        Bilingual {
            en: "Confirm".into(),
            ko: "확정".into(),
        },
    );
    m.insert(
        Msg::Back,
        Bilingual {
            en: "Back".into(),
            ko: "뒤로".into(),
        },
    );
    m.insert(
        Msg::DryRunPassed,
        Bilingual {
            en: "dry-run PASSED — bundle is safe to install.".into(),
            ko: "dry-run 통과 — 번들을 설치해도 안전합니다.".into(),
        },
    );
    m.insert(
        Msg::DryRunFailed,
        Bilingual {
            en: "dry-run FAILED — review the report before installing.".into(),
            ko: "dry-run 실패 — 설치 전 보고서를 확인하세요.".into(),
        },
    );
    m.insert(
        Msg::Installed,
        Bilingual {
            en: "Harness installed. Run with: byoh run <slug>".into(),
            ko: "하네스 설치 완료. 실행: byoh run <slug>".into(),
        },
    );
    m.insert(
        Msg::DependencyMissing,
        Bilingual {
            en: "A dependency tool is missing — falling back gracefully.".into(),
            ko: "의존 도구가 없습니다 — 안전하게 폴백합니다.".into(),
        },
    );
    m.insert(
        Msg::StagnationRollback,
        Bilingual {
            en: "Stagnation detected — rolling back to the last good config.".into(),
            ko: "정체 감지 — 마지막 양호 설정으로 롤백합니다.".into(),
        },
    );
    m.insert(
        Msg::EvolveApproved,
        Bilingual {
            en: "Evolution approved by all safety gates.".into(),
            ko: "모든 안전장치가 진화를 승인했습니다.".into(),
        },
    );
    m
}

/// Resolve a message for a language.
pub fn t(msg: Msg, lang: &str) -> String {
    catalog()
        .get(&msg)
        .map(|b| b.get(lang).to_string())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_has_both_languages() {
        let c = catalog();
        for key in [
            Msg::Welcome,
            Msg::DryRunPassed,
            Msg::Installed,
            Msg::StagnationRollback,
        ] {
            let b = c.get(&key).unwrap();
            assert!(!b.en.is_empty());
            assert!(!b.ko.is_empty());
        }
    }

    #[test]
    fn t_resolves_by_language() {
        assert!(t(Msg::Confirm, "en").contains("Confirm"));
        assert!(t(Msg::Confirm, "ko").contains("확정"));
    }
}
