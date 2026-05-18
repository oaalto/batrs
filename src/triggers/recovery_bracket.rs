use crate::stats::StatsEffect;
use crate::triggers::{TriggerEffects, TriggerFacts, TriggerLine};

const LIE_DOWN_REST: &str = "You lie down and begin to rest for a while.";
const FEEL_TIRED: &str = "You feel a bit tired.";
const STRETCH_CONSIDER_CAMPING: &str = "You stretch yourself and consider camping.";
const FEEL_LIKE_CAMPING: &str = "You feel like camping a little.";
const START_MEDITATING: &str = "You sit down and start meditating.";
const MEDITATION_HARMONY: &str =
    "You feel in harmony with yourself, the universe and life in general.";

pub fn trigger(line: &TriggerLine<'_>, _facts: &TriggerFacts) -> TriggerEffects {
    let plain = line.plain_line.trim_end_matches('\r').trim();
    match plain {
        LIE_DOWN_REST => TriggerEffects::none().stat(StatsEffect::SetRecoveryBracketCamping(false)),
        FEEL_TIRED | STRETCH_CONSIDER_CAMPING | FEEL_LIKE_CAMPING => {
            TriggerEffects::none().stat(StatsEffect::SetRecoveryBracketCamping(true))
        }
        START_MEDITATING => {
            TriggerEffects::none().stat(StatsEffect::SetRecoveryBracketMeditation(false))
        }
        MEDITATION_HARMONY => {
            TriggerEffects::none().stat(StatsEffect::SetRecoveryBracketMeditation(true))
        }
        _ => TriggerEffects::none(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::Stats;
    use ratatui::text::Line;

    fn apply_trigger(line: &str, stats: &mut Stats) {
        let output = trigger(&TriggerLine::new(line), &TriggerFacts::default());
        for effect in output.stats {
            stats.apply_effect(effect);
        }
    }

    fn line_plain(line: &Line<'_>) -> String {
        line.spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect()
    }

    #[test]
    fn camping_and_meditation_lines_toggle_flags() {
        let mut stats = Stats::default();

        assert!(line_plain(&stats.render_inline()).ends_with("  []"));

        apply_trigger(FEEL_TIRED, &mut stats);
        assert!(line_plain(&stats.render_inline()).ends_with("  [c]"));

        apply_trigger(LIE_DOWN_REST, &mut stats);
        assert!(line_plain(&stats.render_inline()).ends_with("  []"));

        apply_trigger(MEDITATION_HARMONY, &mut stats);
        assert!(line_plain(&stats.render_inline()).ends_with("  [m]"));

        apply_trigger(START_MEDITATING, &mut stats);
        assert!(line_plain(&stats.render_inline()).ends_with("  []"));
    }

    #[test]
    fn ignores_other_lines() {
        let mut stats = Stats::default();
        stats.set_recovery_bracket_defaults_for_login();

        assert!(line_plain(&stats.render_inline()).ends_with("  [cm]"));

        apply_trigger("Something irrelevant.", &mut stats);
        assert!(line_plain(&stats.render_inline()).ends_with("  [cm]"));
    }

    #[test]
    fn each_camping_hint_line_turns_camping_on() {
        for msg in [FEEL_TIRED, STRETCH_CONSIDER_CAMPING, FEEL_LIKE_CAMPING] {
            let mut stats = Stats::default();
            apply_trigger(LIE_DOWN_REST, &mut stats);
            assert!(
                line_plain(&stats.render_inline()).ends_with("  []"),
                "lie down should clear c for {msg:?}"
            );
            apply_trigger(msg, &mut stats);
            assert!(
                line_plain(&stats.render_inline()).ends_with("  [c]"),
                "expected c on for {msg:?}"
            );
        }
    }
}
