use crate::ansi::{StyledLine, TextStyle};
use crate::guilds::CivmageGuild;
use crate::guilds::magic_lore_analysis::paint_magic_lore_analysis;
use crate::triggers::{Trigger, TriggerContext, TriggerOutput};

impl CivmageGuild {
    pub fn get_triggers(&self) -> Vec<Trigger> {
        vec![Self::civmage_trigger]
    }

    pub fn civmage_trigger(
        _ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> TriggerOutput {
        let mut output = TriggerOutput::default();
        let line = styled_line
            .plain_line
            .trim_end_matches('\r')
            .trim()
            .to_string();

        if line == "You feel odd. Not weaker, but..." {
            styled_line.set_line_style(TextStyle::BRIGHT_RED);
            return output;
        }

        if line == "Your disc wavers dangerously." {
            output.lines.push(disc_notice());
            return output;
        }

        if paint_magic_lore_analysis(styled_line, line.as_str()) {
            return output;
        }

        output
    }
}

fn disc_notice() -> StyledLine {
    let mut line = StyledLine::new("FLOATING DISC IS GOING DOWN!");
    line.set_line_style(TextStyle::BRIGHT_RED);
    line
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ansi::AnsiCode;
    use crate::automation::Automation;
    use crate::stats::Stats;

    fn run_trigger(line: &str) -> (TriggerOutput, StyledLine) {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        let mut ctx = TriggerContext {
            stats: &mut stats,
            automation: &mut automation,
            rig: None,
            player_name: None,
        };
        let mut styled = StyledLine::new(line);
        let output = CivmageGuild::civmage_trigger(&mut ctx, &mut styled);
        (output, styled)
    }

    #[test]
    fn lift_side_effect_line_red_bold() {
        let (_output, styled) = run_trigger("You feel odd. Not weaker, but...");
        assert_eq!(styled.styled_chars[0].color, AnsiCode::Red);
        assert!(styled.styled_chars[0].bold);
    }

    #[test]
    fn disc_waver_inserts_notice() {
        let (output, _) = run_trigger("Your disc wavers dangerously.");
        assert_eq!(output.lines.len(), 1);
        assert_eq!(output.lines[0].plain_line, "FLOATING DISC IS GOING DOWN!");
        assert_eq!(output.lines[0].styled_chars[0].color, AnsiCode::Red);
    }
}
