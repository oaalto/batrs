use crate::ansi::StyledLine;
use crate::triggers::{TriggerContext, TriggerOutput};
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    //H:571/802 [+20] S:635/635 [] E:311/311 [] $:2786 [] exp:21657 []
    pub static ref SC_REGEX: Regex =
        Regex::new(r"^H:(.+)/(.+) \[(.*)\] S:(.+)/(.+) \[(.*)\] E:(.+)/(.+) \[(.*)\] \$:(.+) \[(.*)\] exp:(.+) \[(.*)\]$").unwrap();
}

pub fn trigger(ctx: &mut TriggerContext<'_>, styled_line: &mut StyledLine) -> TriggerOutput {
    if let Some(captures) = SC_REGEX.captures(&styled_line.plain_line) {
        let (_, stats): (&str, [&str; 13]) = captures.extract();
        let stats = stats.map(|stat| stat.parse::<i32>().unwrap_or_default());
        ctx.stats.update_from_short_score(stats);
        styled_line.gag = true;
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

    #[test]
    fn trigger_parses_short_score_stats_and_gags() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        let mut ctx = TriggerContext {
            stats: &mut stats,
            automation: &mut automation,
            rig: None,
            player_name: None,
        };
        let line_text = "H:571/802 [+20] S:635/635 [] E:311/311 [] $:2786 [] exp:21657 []";
        let mut line = StyledLine::new(line_text);

        let _ = trigger(&mut ctx, &mut line);

        assert!(line.gag);
        let status_line = rendered_text(&ctx.stats.render_inline());
        assert!(status_line.contains("Hp: 571/802 (+20)"));
        assert!(status_line.contains("Sp: 635/635"));
        assert!(status_line.contains("Ep: 311/311"));
        assert!(status_line.contains("$: 2786"));
        assert!(status_line.contains("Exp: 21657"));
    }
}
