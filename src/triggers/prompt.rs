use crate::stats::StatsEffect;
use crate::triggers::{TriggerEffects, TriggerFacts, TriggerLine};
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    pub static ref REGEX: Regex =
        Regex::new(r"^Hp:(.+)/(.+) Sp:(.+)/(.+) Ep:(.+)/(.+) Exp:(.+) >$").unwrap();
}

pub fn trigger(line: &TriggerLine<'_>, _facts: &TriggerFacts) -> TriggerEffects {
    if let Some(captures) = REGEX.captures(line.plain_line) {
        let (_, stats): (&str, [&str; 7]) = captures.extract();
        let stats = stats.map(|stat| stat.parse::<i32>().unwrap_or_default());
        return TriggerEffects::none()
            .stat(StatsEffect::UpdatePrompt(stats))
            .gag();
    }

    TriggerEffects::none()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ansi::StyledLine;
    use crate::stats::Stats;

    #[test]
    fn trigger_parses_prompt_stats_and_gags() {
        let mut stats = Stats::default();
        let output = trigger(
            &TriggerLine::new("Hp:1/2 Sp:3/4 Ep:5/6 Exp:7 >"),
            &TriggerFacts::default(),
        );
        let mut line = StyledLine::new("Hp:1/2 Sp:3/4 Ep:5/6 Exp:7 >");
        for effect in output.stats.clone() {
            stats.apply_effect(effect);
        }
        output.apply_line_effects_to(&mut line);

        assert!(line.gag);
        assert_eq!(
            format!("{stats:?}"),
            format!("{:?}", Stats::new([1, 2, 3, 4, 5, 6, 7]))
        );
    }

    #[test]
    fn trigger_ignores_non_prompt_lines() {
        let output = trigger(&TriggerLine::new("not a prompt"), &TriggerFacts::default());
        let mut line = StyledLine::new("not a prompt");
        output.apply_line_effects_to(&mut line);

        assert!(!line.gag);
    }
}
