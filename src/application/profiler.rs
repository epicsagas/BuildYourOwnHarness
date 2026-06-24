//! ProfileOrchestrator — runs S1 → S2 → S3 (ARCH §3.3, §4).

use std::path::Path;

use crate::domain::genre::Genre;
use crate::domain::profile::{DerivedFact, GenreConfidence, ProfileStatus, UserProfile};
use crate::ports::interview::InterviewPort;
use crate::ports::llm::LlmPort;
use crate::ports::source::ProfileSource;
use crate::ports::wizard::WizardPort;

/// Wires the three engines behind a single facade.
pub struct ProfileOrchestrator<'a, S, L, I, W> {
    pub source: &'a S,
    pub llm: &'a L,
    pub interview: &'a I,
    pub wizard: &'a W,
}

impl<'a, S, L, I, W> ProfileOrchestrator<'a, S, L, I, W>
where
    S: ProfileSource,
    L: LlmPort,
    I: InterviewPort,
    W: WizardPort,
{
    pub fn new(source: &'a S, llm: &'a L, interview: &'a I, wizard: &'a W) -> Self {
        Self {
            source,
            llm,
            interview,
            wizard,
        }
    }

    /// S1 autoscan (M1). Non-destructive; fills `candidates` (derived:true).
    pub fn stage1_scan(
        &self,
        profile: &mut UserProfile,
        paths: &[&Path],
    ) -> crate::domain::Result<()> {
        let hits = self.source.scan(paths)?;
        let mut terms: Vec<DerivedFact> = hits
            .iter()
            .map(|h| DerivedFact {
                value: h.term.clone(),
                confidence: 0.5, // autoscan baseline; below re-question threshold
                provenance: vec![h.provenance.clone()],
            })
            .collect();
        terms.sort_by(|a, b| a.value.cmp(&b.value));
        terms.dedup_by(|a, b| a.value == b.value);
        profile.candidates.identity.primary_expertise = terms;

        // Data sources classification.
        for p in paths {
            let ds = self.source.classify(p);
            profile.data_sources.sources.push(ds);
        }
        profile.set_axis(crate::domain::profile::Axis::Data, 0.6);
        profile.updated_at = Some(chrono::Utc::now());
        Ok(())
    }

    /// S2 interview (Suggest-don't-move + optional Council on ambiguous genre).
    /// `answers` maps question id → user answer; missing ⇒ suggestion accepted.
    pub fn stage2_interview(
        &self,
        profile: &mut UserProfile,
        answers: &std::collections::HashMap<String, (String, f64)>,
    ) -> crate::domain::Result<Vec<crate::ports::interview::Question>> {
        // Council: if genre ambiguous, surface 4-voice questions.
        let mut extra: Vec<crate::ports::interview::Question> = Vec::new();
        let genre_conf = profile
            .candidates
            .identity
            .genre
            .as_ref()
            .map(|g| g.confidence)
            .unwrap_or(0.0);
        if genre_conf < 0.7 {
            for (voice, text) in self.llm.council_questions("profile", &profile.language) {
                extra.push(crate::ports::interview::Question {
                    id: format!("council_{}", voice.as_str()),
                    text,
                    axis: crate::domain::profile::Axis::Genre,
                    suggestion: None,
                });
            }
        }

        loop {
            let qs = self.interview.next_questions(profile);
            if qs.is_empty() {
                break;
            }
            for q in &qs {
                let (answer, conf) = answers.get(&q.id).cloned().unwrap_or_else(|| {
                    (
                        q.suggestion
                            .as_ref()
                            .map(|s| s.suggested_answer.clone())
                            .unwrap_or_default(),
                        q.suggestion.as_ref().map(|s| s.confidence).unwrap_or(0.6),
                    )
                });
                if answer.trim().is_empty() {
                    continue;
                }
                self.interview.apply_answer(profile, q, &answer, conf);
            }
            if self.interview.is_complete(profile) {
                break;
            }
            // Avoid infinite loop: if no answers provided and still incomplete, stop.
            if answers.is_empty()
                && qs.iter().all(|q| {
                    q.suggestion
                        .as_ref()
                        .map(|s| s.suggested_answer.is_empty())
                        .unwrap_or(true)
                })
            {
                break;
            }
        }

        if profile.status == ProfileStatus::Draft {
            profile.advance(ProfileStatus::Interviewed)?;
        }
        Ok(extra)
    }

    /// S3 wizard — confirm genre + goal, transition to Confirmed.
    pub fn stage3_confirm(
        &self,
        profile: &mut UserProfile,
        genre: Genre,
        goal_30d: Option<&str>,
    ) -> crate::domain::Result<()> {
        profile.candidates.identity.genre = Some(GenreConfidence {
            value: genre,
            confidence: 1.0,
            provenance: vec!["wizard".into()],
        });
        profile.set_axis(crate::domain::profile::Axis::Genre, 1.0);
        if let Some(g) = goal_30d {
            profile.truth.goals.goal_30d = Some(g.to_string());
            profile.set_axis(crate::domain::profile::Axis::Goals, 1.0);
        }
        profile.advance(ProfileStatus::Confirmed)
    }

    /// Full M0 path with no autoscan: interview + wizard from a draft.
    pub fn run_m0(
        &self,
        profile: &mut UserProfile,
        answers: &std::collections::HashMap<String, (String, f64)>,
        genre: Genre,
        goal_30d: Option<&str>,
    ) -> crate::domain::Result<()> {
        self.stage2_interview(profile, answers)?;
        self.stage3_confirm(profile, genre, goal_30d)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::{FilesystemSource, RuleInterview, RuleLlm, StaticWizard};

    #[test]
    fn full_m0_pipeline_reaches_confirmed() {
        let src = FilesystemSource::new();
        let llm = RuleLlm::new();
        let iv = RuleInterview::new(RuleLlm::new());
        let wz = StaticWizard::new();
        let orch = ProfileOrchestrator::new(&src, &llm, &iv, &wz);

        let mut p = UserProfile::new_draft("dev1", "en");
        let mut answers = std::collections::HashMap::new();
        answers.insert("Q_domain".into(), ("backend".into(), 0.9));
        answers.insert("Q_goal".into(), ("ship faster".into(), 0.9));
        answers.insert("Q_genre".into(), ("developer".into(), 0.9));
        answers.insert("Q_data".into(), ("./vault".into(), 0.8));

        orch.run_m0(&mut p, &answers, Genre::Developer, Some("ship faster"))
            .unwrap();
        assert_eq!(p.status, ProfileStatus::Confirmed);
        assert_eq!(
            p.candidates.identity.genre.as_ref().unwrap().value,
            Genre::Developer
        );
    }
}
