use crate::ansi::StyledLine;
use crate::triggers::{TriggerContext, TriggerOutput};

const LIE_DOWN_REST: &str = "You lie down and begin to rest for a while.";
const FEEL_TIRED: &str = "You feel a bit tired.";
const STRETCH_CONSIDER_CAMPING: &str = "You stretch yourself and consider camping.";
const FEEL_LIKE_CAMPING: &str = "You feel like camping a little.";
const START_MEDITATING: &str = "You sit down and start meditating.";
const MEDITATION_HARMONY: &str =
    "You feel in harmony with yourself, the universe and life in general.";

pub fn trigger(ctx: &mut TriggerContext<'_>, styled_line: &mut StyledLine) -> TriggerOutput {
    let plain = styled_line.plain_line.trim_end_matches('\r').trim();
    match plain {
        LIE_DOWN_REST => ctx.stats.set_recovery_bracket_camping(false),
        FEEL_TIRED | STRETCH_CONSIDER_CAMPING | FEEL_LIKE_CAMPING => {
            ctx.stats.set_recovery_bracket_camping(true);
        }
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
            trigger(&mut ctx, &mut StyledLine::new(FEEL_TIRED));
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

    #[test]
    fn each_camping_hint_line_turns_camping_on() {
        for msg in [
            FEEL_TIRED,
            STRETCH_CONSIDER_CAMPING,
            FEEL_LIKE_CAMPING,
        ] {
            let mut stats = Stats::default();
            {
                let mut automation = Automation::new();
                let mut ctx = ctx(&mut stats, &mut automation);
                trigger(&mut ctx, &mut StyledLine::new(LIE_DOWN_REST));
            }
            assert!(
                line_plain(&stats.render_inline()).ends_with("  []"),
                "lie down should clear c for {msg:?}"
            );
            {
                let mut automation = Automation::new();
                let mut ctx = ctx(&mut stats, &mut automation);
                trigger(&mut ctx, &mut StyledLine::new(msg));
            }
            assert!(
                line_plain(&stats.render_inline()).ends_with("  [c]"),
                "expected c on for {msg:?}"
            );
        }
    }
}
