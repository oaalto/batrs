use crate::stats::StatsEffect;
use crate::triggers::{TriggerEffects, TriggerFacts, TriggerLine};
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    //H:571/802 [+20] S:635/635 [] E:311/311 [] $:2786 [] exp:21657 []
    pub static ref SC_REGEX: Regex =
        Regex::new(r"^H:(.+)/(.+) \[(.*)\] S:(.+)/(.+) \[(.*)\] E:(.+)/(.+) \[(.*)\] \$:(.+) \[(.*)\] exp:(.+) \[(.*)\]$").unwrap();
}

pub fn trigger(line: &TriggerLine<'_>, _facts: &TriggerFacts) -> TriggerEffects {
    if let Some(captures) = SC_REGEX.captures(line.plain_line) {
        let (_, stats): (&str, [&str; 13]) = captures.extract();
        let stats = stats.map(|stat| stat.parse::<i32>().unwrap_or_default());
        return TriggerEffects::none()
            .stat(StatsEffect::UpdateShortScore(stats))
            .gag();
    }

    TriggerEffects::none()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ansi::StyledLine;
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
        let line_text = "H:571/802 [+20] S:635/635 [] E:311/311 [] $:2786 [] exp:21657 []";
        let mut line = StyledLine::new(line_text);
        let output = trigger(&TriggerLine::new(line_text), &TriggerFacts::default());

        for effect in output.stats.clone() {
            stats.apply_effect(effect);
        }
        output.apply_line_effects_to(&mut line);

        assert!(line.gag);
        let status_line = rendered_text(&stats.render_inline());
        assert!(status_line.contains("Hp: 571/802 (+20)"));
        assert!(status_line.contains("Sp: 635/635"));
        assert!(status_line.contains("Ep: 311/311"));
        assert!(status_line.contains("$: 2786"));
        assert!(status_line.contains("Exp: 21657"));
    }
}
