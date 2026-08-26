use serde::{Deserialize, Serialize};

use crate::DevelopmentContract;

use super::HarnessRegistry;

/// Evidence explaining why a harness profile was selected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HarnessRoutingEvidence {
    pub profile_id: String,
    pub score: u32,
    pub matched_terms: Vec<String>,
}

/// Deterministic result of routing a development contract to harness profiles.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HarnessRoute {
    pub selected: Vec<HarnessRoutingEvidence>,
}

/// Whole-token trigger match: `term` must appear in `text` not immediately
/// flanked by alphanumeric/underscore characters, so e.g. the trigger "idea"
/// does not match "identify" or "ideal".
fn contains_term(text: &str, term: &str) -> bool {
    if term.is_empty() || term.len() > text.len() {
        return false;
    }
    let is_word_char = |c: char| c.is_alphanumeric() || c == '_';
    let mut start = 0;
    while let Some(offset) = text[start..].find(term) {
        let begin = start + offset;
        let end = begin + term.len();
        let before_free = !text[..begin]
            .chars()
            .next_back()
            .map(is_word_char)
            .unwrap_or(false);
        let after_free = !text[end..]
            .chars()
            .next()
            .map(is_word_char)
            .unwrap_or(false);
        if before_free && after_free {
            return true;
        }
        start = begin + 1;
        while start < text.len() && !text.is_char_boundary(start) {
            start += 1;
        }
    }
    false
}

/// Select at most `max_profiles` relevant harness profiles, best first.
///
/// V0 routing is deliberately model-free. It scores whole-token trigger-term
/// matches across the contract goal, acceptance criteria, and constraints,
/// then uses stable profile ID ordering as the deterministic tie-break. The
/// returned evidence is advisory only and cannot authorize or execute effects.
pub fn route_harness(
    registry: &HarnessRegistry,
    contract: &DevelopmentContract,
    max_profiles: usize,
) -> HarnessRoute {
    let text = searchable_text(contract);
    let mut selected: Vec<HarnessRoutingEvidence> = registry
        .profiles()
        .iter()
        .filter_map(|profile| {
            let mut score = 0u32;
            let mut matched_terms = Vec::new();

            for trigger in &profile.triggers {
                let term = trigger.trim().to_ascii_lowercase();
                if !term.is_empty() && contains_term(&text, &term) {
                    score += 10;
                    matched_terms.push(term);
                }
            }

            (score > 0).then(|| HarnessRoutingEvidence {
                profile_id: profile.id.clone(),
                score,
                matched_terms,
            })
        })
        .collect();

    selected.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| a.profile_id.cmp(&b.profile_id))
    });
    selected.truncate(max_profiles);

    HarnessRoute { selected }
}

fn searchable_text(contract: &DevelopmentContract) -> String {
    let mut parts =
        Vec::with_capacity(1 + contract.acceptance_criteria.len() + contract.constraints.len());
    parts.push(contract.goal.as_str());
    parts.extend(contract.acceptance_criteria.iter().map(String::as_str));
    parts.extend(contract.constraints.iter().map(String::as_str));
    parts.join(" ").to_ascii_lowercase()
}
