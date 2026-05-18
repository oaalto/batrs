use crate::stats::StatsEffect;
use crate::triggers::{TriggerEffects, TriggerFacts, TriggerLine};
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref ROUND_REGEX: Regex = Regex::new(r"^[\*]+ Round .* [\*]+$").unwrap();
}

pub fn trigger(line: &TriggerLine<'_>, _facts: &TriggerFacts) -> TriggerEffects {
    if ROUND_REGEX.is_match(line.plain_line) {
        return TriggerEffects::none().stat(StatsEffect::StartCombatRound);
    } else if line.plain_line == "You are not in combat right now." {
        return TriggerEffects::none().stat(StatsEffect::EndCombat);
    }

    TriggerEffects::none()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::Stats;

    fn rendered_text(line: &ratatui::text::Line<'_>) -> String {
        line.spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect()
    }

    fn apply_trigger(line: &str, stats: &mut Stats) {
        let output = trigger(&TriggerLine::new(line), &TriggerFacts::default());
        for effect in output.stats {
            stats.apply_effect(effect);
        }
    }

    #[test]
    fn round_header_starts_round_and_clears_old_diffs() {
        let mut stats = Stats::default();
        stats.update_from_short_score([1, 2, 10, 3, 4, 0, 5, 6, 0, 100, 0, 8, 0]);
        apply_trigger(
            "********************** Round 5 **********************",
            &mut stats,
        );

        let status_line = rendered_text(&stats.render_inline());
        assert!(
            !status_line.contains("+10"),
            "round header should clear old diffs; got {status_line:?}"
        );

        stats.update_from_short_score([1, 2, 0, 3, 4, -3, 5, 6, 0, 100, 0, 8, 0]);
        stats.update_from_short_score([1, 2, 0, 3, 4, 0, 5, 6, 0, 100, 0, 8, 0]);

        let status_line = rendered_text(&stats.render_inline());
        assert!(
            status_line.contains("-3"),
            "round header should make later zero-diff sc preserve round totals; got {status_line:?}"
        );
    }

    #[test]
    fn not_in_combat_ends_round_and_preserves_final_diffs() {
        let mut stats = Stats::default();
        stats.start_combat_round();
        stats.update_from_short_score([1, 2, 10, 3, 4, 0, 5, 6, 0, 100, 0, 8, 0]);
        apply_trigger("You are not in combat right now.", &mut stats);

        let status_line = rendered_text(&stats.render_inline());
        assert!(
            status_line.contains("+10"),
            "not-in-combat line should preserve final combat diffs; got {status_line:?}"
        );
    }
}
