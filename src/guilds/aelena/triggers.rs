use crate::abilities;
use crate::ansi::{StyledLine, TextStyle};
use crate::automation::Action;
use crate::guilds::AelenaGuild;
use crate::triggers::{Trigger, TriggerContext, TriggerOutput};
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref CHAOS_UNPOWERED: Regex =
        Regex::new(r"^Your (.+) is no longer powered by Chaos!$").unwrap();
    static ref WOUNDS_SPILL: Regex =
        Regex::new(r"^(.+)'s wounds spill blood onto the floor\.$").unwrap();
}

impl AelenaGuild {
    pub fn get_triggers(&self) -> Vec<Trigger> {
        vec![Self::aelena_trigger]
    }

    pub fn aelena_trigger(
        _ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> TriggerOutput {
        let mut output = TriggerOutput::default();
        let line = styled_line.plain_line.as_str();

        if line.contains("and harvests a bloody bodypart 'spleen'.") {
            output
                .actions
                .push(Action::Send(abilities::client_send_line(
                    "familiar store slow death",
                )));
        }
        if line.contains("and harvests a bloody bodypart 'lung'.") {
            output
                .actions
                .push(Action::Send(abilities::client_send_line(
                    "familiar store rusted blade",
                )));
        }
        if line.contains("and harvests a bloody bodypart 'eye'.") {
            output
                .actions
                .push(Action::Send(abilities::client_send_line(
                    "familiar store black trance",
                )));
        }

        if line == "Your Shadow Familiar shrieks as it advances a level!" {
            styled_line.set_line_style(TextStyle::BRIGHT_GREEN);
        } else if line
            == "The surge of magic backlashes at you, just when you're finishing the spell."
            || line == "You fail to chant the spell correctly."
            || line.starts_with(
                "At the last moment you notice the spell is about to turn at you, and abort the",
            )
        {
            styled_line.set_line_style(TextStyle::BRIGHT_RED);
        } else if CHAOS_UNPOWERED.is_match(line) {
            styled_line.set_line_style(TextStyle::RED);
        } else if WOUNDS_SPILL.is_match(line)
            || line == "Your senses sharpen as you fight for you life."
        {
            styled_line.set_line_style(TextStyle::GREEN);
        } else if line == "The connection between you and your blade fades away." {
            styled_line.set_line_style(TextStyle::BRIGHT_RED);
            let mut echo = StyledLine::new("Command Blade down!");
            echo.set_line_style(TextStyle::BRIGHT_MAGENTA);
            output.lines.push(echo);
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ansi::AnsiCode;
    use crate::automation::Automation;
    use crate::stats::Stats;

    fn run(line: &str) -> (TriggerOutput, StyledLine) {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        let mut ctx = TriggerContext {
            stats: &mut stats,
            automation: &mut automation,
            rig: None,
            player_name: None,
        };
        let mut styled_line = StyledLine::new(line);
        let output = AelenaGuild::aelena_trigger(&mut ctx, &mut styled_line);
        (output, styled_line)
    }

    #[test]
    fn harvest_spleen_sends_store_slow_death() {
        let line = "Your imp and harvests a bloody bodypart 'spleen'.";
        let (output, _) = run(line);
        assert!(output.actions.iter().any(|a| matches!(
            a,
            Action::Send(s) if s == "@familiar store slow death"
        )));
    }

    #[test]
    fn spell_backlash_paints_red_bold() {
        let line = "You fail to chant the spell correctly.";
        let (_, styled) = run(line);
        assert_eq!(styled.styled_chars[0].color, AnsiCode::Red);
        assert!(styled.styled_chars[0].bold);
    }

    #[test]
    fn command_blade_fade_echoes_magenta() {
        let line = "The connection between you and your blade fades away.";
        let (output, styled) = run(line);
        assert_eq!(styled.styled_chars[0].color, AnsiCode::Red);
        assert!(styled.styled_chars[0].bold);
        assert_eq!(output.lines.len(), 1);
        assert_eq!(output.lines[0].plain_line, "Command Blade down!");
        assert_eq!(output.lines[0].styled_chars[0].color, AnsiCode::Magenta);
        assert!(output.lines[0].styled_chars[0].bold);
    }
}
