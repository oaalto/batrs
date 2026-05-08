use crate::ansi::StyledLine;
use crate::triggers::{TriggerContext, TriggerOutput};
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    pub static ref REGEX: Regex =
        Regex::new(r"^Hp:(.+)/(.+) Sp:(.+)/(.+) Ep:(.+)/(.+) Exp:(.+) >$").unwrap();
}

pub fn trigger(ctx: &mut TriggerContext<'_>, styled_line: &mut StyledLine) -> TriggerOutput {
    if let Some(captures) = REGEX.captures(&styled_line.plain_line) {
        let (_, stats): (&str, [&str; 7]) = captures.extract();
        let stats = stats.map(|stat| stat.parse::<i32>().unwrap_or_default());
        ctx.stats.update_from_prompt(stats);
        styled_line.gag = true;
    }

    TriggerOutput::default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::automation::Automation;
    use crate::stats::Stats;

    #[test]
    fn trigger_parses_prompt_stats_and_gags() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        let mut ctx = TriggerContext {
            stats: &mut stats,
            automation: &mut automation,
            rig: None,
        };
        let mut line = StyledLine::new("Hp:1/2 Sp:3/4 Ep:5/6 Exp:7 >");

        let _ = trigger(&mut ctx, &mut line);

        assert!(line.gag);
        assert_eq!(
            format!("{:?}", *ctx.stats),
            format!("{:?}", Stats::new([1, 2, 3, 4, 5, 6, 7]))
        );
    }

    #[test]
    fn trigger_ignores_non_prompt_lines() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        let mut ctx = TriggerContext {
            stats: &mut stats,
            automation: &mut automation,
            rig: None,
        };
        let mut line = StyledLine::new("not a prompt");

        let _ = trigger(&mut ctx, &mut line);

        assert!(!line.gag);
    }
}
