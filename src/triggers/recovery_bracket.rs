use crate::ansi::StyledLine;
use crate::triggers::{TriggerContext, TriggerOutput};

const LIE_DOWN_REST: &str = "You lie down and begin to rest for a while.";
const SHORT_REST_DONE: &str = "You awaken from your short rest, and feel slightly better.";
const START_MEDITATING: &str = "You sit down and start meditating.";
const MEDITATION_HARMONY: &str =
    "You feel in harmony with yourself, the universe and life in general.";

pub fn trigger(ctx: &mut TriggerContext<'_>, styled_line: &mut StyledLine) -> TriggerOutput {
    let plain = styled_line.plain_line.trim_end_matches('\r').trim();
    match plain {
        LIE_DOWN_REST => ctx.stats.set_recovery_bracket_camping(false),
        SHORT_REST_DONE => ctx.stats.set_recovery_bracket_camping(true),
        START_MEDITATING => ctx.stats.set_recovery_bracket_meditation(false),
        MEDITATION_HARMONY => ctx.stats.set_recovery_bracket_meditation(true),
        _ => {}
    }
    TriggerOutput::default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::automation::Automation;
    use crate::stats::Stats;
    use ratatui::text::Line;

    fn ctx<'a>(stats: &'a mut Stats, automation: &'a mut Automation) -> TriggerContext<'a> {
        TriggerContext {
            stats,
            automation,
            rig: None,
            player_name: None,
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
        let mut automation = Automation::new();

        assert!(line_plain(&stats.render_inline()).ends_with("  []"));

        {
            let mut ctx = ctx(&mut stats, &mut automation);
            trigger(&mut ctx, &mut StyledLine::new(SHORT_REST_DONE));
        }
        assert!(line_plain(&stats.render_inline()).ends_with("  [c]"));

        {
            let mut ctx = ctx(&mut stats, &mut automation);
            trigger(&mut ctx, &mut StyledLine::new(LIE_DOWN_REST));
        }
        assert!(line_plain(&stats.render_inline()).ends_with("  []"));

        {
            let mut ctx = ctx(&mut stats, &mut automation);
            trigger(&mut ctx, &mut StyledLine::new(MEDITATION_HARMONY));
        }
        assert!(line_plain(&stats.render_inline()).ends_with("  [m]"));

        {
            let mut ctx = ctx(&mut stats, &mut automation);
            trigger(&mut ctx, &mut StyledLine::new(START_MEDITATING));
        }
        assert!(line_plain(&stats.render_inline()).ends_with("  []"));
    }

    #[test]
    fn ignores_other_lines() {
        let mut stats = Stats::default();
        stats.set_recovery_bracket_defaults_for_login();
        let mut automation = Automation::new();

        assert!(line_plain(&stats.render_inline()).ends_with("  [cm]"));

        {
            let mut ctx = ctx(&mut stats, &mut automation);
            trigger(&mut ctx, &mut StyledLine::new("Something irrelevant."));
        }
        assert!(line_plain(&stats.render_inline()).ends_with("  [cm]"));
    }
}
