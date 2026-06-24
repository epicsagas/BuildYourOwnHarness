//! Wizard port — S3 decisive self-describing options (B4).

use crate::domain::genre::Genre;

/// One selectable option, carrying its own "why this fits" explanation (B4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WizardOption {
    pub id: String,
    pub label_en: String,
    pub label_ko: String,
    pub why_en: String,
    pub why_ko: String,
}

/// S3 wizard. Presents decisive choices; the user `Confirm`s.
pub trait WizardPort {
    /// Genre options with self-describing rationale.
    fn genre_options(&self, language: &str) -> Vec<WizardOption>;

    /// Goal-framing options for the chosen genre.
    fn goal_options(&self, genre: Genre, language: &str) -> Vec<WizardOption>;

    /// Render an option's "why" for the active language.
    fn render_option(&self, opt: &WizardOption, language: &str) -> String {
        if language == "ko" {
            format!("{} — {}", opt.label_ko, opt.why_ko)
        } else {
            format!("{} — {}", opt.label_en, opt.why_en)
        }
    }
}
