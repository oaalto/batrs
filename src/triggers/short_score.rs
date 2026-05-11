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
        assert_eq!(
            format!("{:?}", *ctx.stats),
            format!(
                "{:?}",
                Stats::new_from_sc([571, 802, 20, 635, 635, 0, 311, 311, 0, 2786, 0, 21657, 0])
            )
        );
    }
}
