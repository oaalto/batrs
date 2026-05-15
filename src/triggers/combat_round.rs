use crate::ansi::StyledLine;
use crate::triggers::{TriggerContext, TriggerOutput};
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref ROUND_REGEX: Regex = Regex::new(r"^[\*]+ Round .* [\*]+$").unwrap();
}

pub fn trigger(ctx: &mut TriggerContext<'_>, styled_line: &mut StyledLine) -> TriggerOutput {
    if ROUND_REGEX.is_match(&styled_line.plain_line) {
        ctx.stats.start_combat_round();
    } else if styled_line.plain_line == "You are not in combat right now." {
        ctx.stats.end_combat();
    }

    TriggerOutput::default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::automation::Automation;
    use crate::stats::Stats;

    fn rendered_text(line: &ratatui::text::Line<'_>) -> String {
        line.spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect()
    }

    fn ctx<'a>(stats: &'a mut Stats, automation: &'a mut Automation) -> TriggerContext<'a> {
        TriggerContext {
            stats,
            automation,
            rig: None,
            player_name: None,
        }
    }

    #[test]
    fn round_header_starts_round_and_clears_old_diffs() {
        let mut stats = Stats::default();
        stats.update_from_short_score([1, 2, 10, 3, 4, 0, 5, 6, 0, 100, 0, 8, 0]);
        let mut automation = Automation::new();
        let mut ctx = ctx(&mut stats, &mut automation);
        let mut line = StyledLine::new("********************** Round 5 **********************");

        let _ = trigger(&mut ctx, &mut line);

        let status_line = rendered_text(&ctx.stats.render_inline());
        assert!(
            !status_line.contains("+10"),
            "round header should clear old diffs; got {status_line:?}"
        );

        ctx.stats
            .update_from_short_score([1, 2, 0, 3, 4, -3, 5, 6, 0, 100, 0, 8, 0]);
        ctx.stats
            .update_from_short_score([1, 2, 0, 3, 4, 0, 5, 6, 0, 100, 0, 8, 0]);

        let status_line = rendered_text(&ctx.stats.render_inline());
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
        let mut automation = Automation::new();
        let mut ctx = ctx(&mut stats, &mut automation);
        let mut line = StyledLine::new("You are not in combat right now.");

        let _ = trigger(&mut ctx, &mut line);

        let status_line = rendered_text(&ctx.stats.render_inline());
        assert!(
            status_line.contains("+10"),
            "not-in-combat line should preserve final combat diffs; got {status_line:?}"
        );
    }
}
