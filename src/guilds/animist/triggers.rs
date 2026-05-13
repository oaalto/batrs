use crate::ansi::{StyledLine, TextStyle};
use crate::automation::Action;
use crate::guilds::AnimistGuild;
use crate::triggers::{Trigger, TriggerContext, TriggerOutput};
use lazy_static::lazy_static;
use regex::Regex;

impl AnimistGuild {
    pub fn get_triggers(&self) -> Vec<Trigger> {
        vec![
            Self::soul_companion_status_trigger,
            Self::spirit_appears_trigger,
            Self::soul_companion_training_trigger,
        ]
    }

    pub fn soul_companion_status_trigger(
        ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> TriggerOutput {
        let plain = styled_line.plain_line.trim_end_matches('\r').trim();
        if let Some(captures) = SOUL_COMPANION_STATUS.captures(plain) {
            let percent = captures[2].parse::<i32>().unwrap_or_default();
            let description = captures[3].trim().to_string();
            styled_line.gag = true;
            ctx.stats
                .set_soul_companion(percent.clamp(0, 100), description);
        }

        TriggerOutput::default()
    }

    pub fn spirit_appears_trigger(
        _ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> TriggerOutput {
        let mut output = TriggerOutput::default();
        if SPIRIT_APPEARS.is_match(&styled_line.plain_line) {
            output
                .actions
                .push(Action::Send("@lead my spirit".to_string()));
        }
        output
    }

    pub fn soul_companion_training_trigger(
        _ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> TriggerOutput {
        if styled_line.plain_line
            == "You feel slightly better at fighting with your soul companion."
        {
            styled_line.set_line_style(TextStyle::BLUE);
        }
        TriggerOutput::default()
    }
}

lazy_static! {
    /// Percent may use non-ASCII digits; trailing status text (e.g. `+`) is optional.
    static ref SOUL_COMPANION_STATUS: Regex = Regex::new(
        r"(?i)^Your\s+soul\s+companion\s*:\s*(.+?)\s+\((\d+)%\)\s*(.*?)\s*$",
    )
    .unwrap();
    static ref SPIRIT_APPEARS: Regex =
        Regex::new(r"^(.+) spirit slowly appears, answering your call\.$").unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ansi::AnsiCode;
    use crate::automation::Automation;
    use crate::stats::Stats;

    fn ctx<'a>(stats: &'a mut Stats, automation: &'a mut Automation) -> TriggerContext<'a> {
        TriggerContext {
            stats,
            automation,
            rig: None,
            player_name: None,
        }
    }

    fn stats_line_text(stats: &Stats) -> String {
        stats
            .render_soul_inline()
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect()
    }

    #[test]
    fn soul_companion_status_updates_stats() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        let mut ctx = ctx(&mut stats, &mut automation);
        let mut line = StyledLine::new("Your soul companion: exc (88%) guarding you");

        let _ = AnimistGuild::soul_companion_status_trigger(&mut ctx, &mut line);

        assert!(line.gag);
        assert_eq!(stats_line_text(ctx.stats), "Soul: 88% guarding you");
    }

    #[test]
    fn soul_companion_status_trims_carriage_return() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        let mut ctx = ctx(&mut stats, &mut automation);
        let mut line = StyledLine::new("Your soul companion: exc (88%) +\r");

        let _ = AnimistGuild::soul_companion_status_trigger(&mut ctx, &mut line);

        assert!(line.gag);
        assert_eq!(stats_line_text(ctx.stats), "Soul: 88% +");
    }

    #[test]
    fn soul_companion_status_optional_suffix() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        let mut ctx = ctx(&mut stats, &mut automation);
        let mut line = StyledLine::new("your soul companion: wolf (50%)");

        let _ = AnimistGuild::soul_companion_status_trigger(&mut ctx, &mut line);

        assert!(line.gag);
        assert_eq!(stats_line_text(ctx.stats), "Soul: 50% ");
    }

    #[test]
    fn spirit_appearing_leads_it() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        let mut ctx = ctx(&mut stats, &mut automation);
        let mut line = StyledLine::new("A wolf spirit slowly appears, answering your call.");

        let output = AnimistGuild::spirit_appears_trigger(&mut ctx, &mut line);

        assert!(matches!(
            &output.actions[0],
            Action::Send(command) if command == "@lead my spirit"
        ));
    }

    #[test]
    fn soul_companion_training_is_blue() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        let mut ctx = ctx(&mut stats, &mut automation);
        let mut line =
            StyledLine::new("You feel slightly better at fighting with your soul companion.");

        let _ = AnimistGuild::soul_companion_training_trigger(&mut ctx, &mut line);

        assert_eq!(line.styled_chars[0].color, AnsiCode::Blue);
    }
}
