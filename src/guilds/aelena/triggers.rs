use crate::abilities;
use crate::ansi::{StyledLine, TextStyle};
use crate::automation::Action;
use crate::guilds::AelenaGuild;
use crate::triggers::{Trigger, TriggerEffects, TriggerFacts, TriggerLine};
use regex::Regex;
use std::sync::LazyLock;

static CHAOS_UNPOWERED: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^Your (.+) is no longer powered by Chaos!$").unwrap());
static WOUNDS_SPILL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(.+)'s wounds spill blood onto the floor\.$").unwrap());

impl AelenaGuild {
    pub fn get_triggers(&self) -> Vec<Trigger> {
        vec![Self::aelena_trigger]
    }

    pub fn aelena_trigger(line: &TriggerLine<'_>, _facts: &TriggerFacts) -> TriggerEffects {
        let mut output = TriggerEffects::default();
        let line = line.plain_line;

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
            output = output.style_line(TextStyle::BRIGHT_GREEN);
        } else if line
            == "The surge of magic backlashes at you, just when you're finishing the spell."
            || line == "You fail to chant the spell correctly."
            || line.starts_with(
                "At the last moment you notice the spell is about to turn at you, and abort the",
            )
        {
            output = output.style_line(TextStyle::BRIGHT_RED);
        } else if CHAOS_UNPOWERED.is_match(line) {
            output = output.style_line(TextStyle::RED);
        } else if WOUNDS_SPILL.is_match(line)
            || line == "Your senses sharpen as you fight for you life."
        {
            output = output.style_line(TextStyle::GREEN);
        } else if line == "The connection between you and your blade fades away." {
            output = output.style_line(TextStyle::BRIGHT_RED);
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
    use crate::triggers::{TriggerFacts, TriggerLine};

    fn run(line: &str) -> (TriggerEffects, StyledLine) {
        let mut styled_line = StyledLine::new(line);
        let output = AelenaGuild::aelena_trigger(&TriggerLine::new(line), &TriggerFacts::default());
        output.apply_line_effects_to(&mut styled_line);
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
