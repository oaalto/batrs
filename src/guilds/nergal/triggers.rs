use crate::ansi::{StyledLine, TextStyle};
use crate::automation::Action;
use crate::guilds::NergalGuild;
use crate::stats::NergalMinion;
use crate::triggers::{Trigger, TriggerContext, TriggerOutput};
use lazy_static::lazy_static;
use regex::Regex;

impl NergalGuild {
    pub fn get_triggers(&self) -> Vec<Trigger> {
        vec![Self::nergal_trigger]
    }

    pub fn nergal_trigger(
        ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> TriggerOutput {
        let mut output = TriggerOutput::default();
        let line = styled_line.plain_line.trim_end_matches('\r').trim();

        if let Some(captures) = MINION_STATUS.captures(line) {
            styled_line.gag = true;
            let minion = NergalMinion {
                name: captures[1].to_string(),
                hp: captures[2].parse().unwrap_or(0),
                max_hp: captures[3].parse().unwrap_or(0),
                sp: captures[4].parse().unwrap_or(0),
                max_sp: captures[5].parse().unwrap_or(0),
                ep: captures[6].parse().unwrap_or(0),
                max_ep: captures[7].parse().unwrap_or(0),
            };
            ctx.stats.upsert_nergal_minion(minion);
            return output;
        }

        if UNSUMMON_CONNECTION.is_match(line) || UNSUMMON_END.is_match(line) {
            ctx.stats.clear_nergal_minions();
            return output;
        }

        if line.contains("DEAD, R.I.P.") {
            output.actions.push(Action::Send("@nergal sc".to_string()));
            return output;
        }

        if line.contains("Potentia: 1000/1000") {
            output
                .lines
                .push(echo_notice("***** POTENTIA IS FULL! *****", true));
            return output;
        }

        if line == "Your body can't handle any more of potentia!" {
            styled_line.set_line_style(TextStyle::RED);
            output
                .lines
                .push(echo_notice("***** POTENTIA IS FULL! *****", false));
            return output;
        }

        if line.contains("Vitae: 1000/1000") {
            output
                .lines
                .push(echo_notice("***** VITAE IS FULL! *****", true));
            return output;
        }

        if line == "Your body can't handle any more of vitae!" {
            styled_line.set_line_style(TextStyle::RED);
            output
                .lines
                .push(echo_notice("***** VITAE IS FULL! *****", false));
            return output;
        }

        if line.contains("looks a lot less in pain as colonies start to disappear")
            || HARVEST_VITAE.is_match(line)
            || HARVEST_POTENTIA.is_match(line)
            || line.contains("You feel your insight of evolution expanding")
        {
            styled_line.set_line_style(TextStyle::CYAN);
        } else if line
            .contains("You hear deep inside your head the parasite whispers more secrets of")
            || line.contains(
                "You hear deep inside your head the parasite whispering to you secrets of",
            )
        {
            styled_line.set_line_style(TextStyle::GREEN);
        } else if line.contains("looks relieved as the aether line fades away") {
            styled_line.set_line_style(TextStyle::BLUE);
        } else if AURA_SCRATCH.is_match(line)
            || AURA_PLUNGES.is_match(line)
            || AURA_ESSENCE.is_match(line)
        {
            styled_line.set_line_style(TextStyle::GREEN);
        }

        output
    }
}

fn echo_notice(text: &str, green: bool) -> StyledLine {
    let mut line = StyledLine::new(text);
    line.set_line_style(if green {
        TextStyle::BRIGHT_GREEN
    } else {
        TextStyle::BRIGHT_RED
    });
    line
}

lazy_static! {
    static ref MINION_STATUS: Regex = Regex::new(
        r"^::\.\.:\. (.+) \[Hp: (-?[0-9]+) \(([0-9]+)\)[ \-+()0-9]*, Sp: (-?[0-9]+) \(([0-9]+)\)[ \-+()0-9]*, Ep: (-?[0-9]+) \(([0-9]+)\)[ \-+()0-9]*\]$",
    )
    .unwrap();
    static ref UNSUMMON_CONNECTION: Regex = Regex::new(
        r"^Your connection to your parasite is severed completely\. (.+) jerks violently couple of times and collapses\.$",
    )
    .unwrap();
    static ref UNSUMMON_END: Regex = Regex::new(
        r"^You end the connection to your parasite, making the host jerk couple of times violently\. After couple of seconds (.+) collapses and stops moving at all\.$",
    )
    .unwrap();
    static ref HARVEST_VITAE: Regex =
        Regex::new(r"You feel you harvest (.+) amount of vitae\.\.").unwrap();
    static ref HARVEST_POTENTIA: Regex =
        Regex::new(r"You feel you harvest (.+) amount of potentia\.\.").unwrap();
    static ref AURA_SCRATCH: Regex = Regex::new(
        r"(.*) manages to scratch (.*) skin infecting the tissue under the skin with nasty disease",
    )
    .unwrap();
    static ref AURA_PLUNGES: Regex = Regex::new(
        r"(.*) plunges towards (.*) and manages to sink its disease infecting nails into (.*) flesh!",
    )
    .unwrap();
    static ref AURA_ESSENCE: Regex = Regex::new(
        r"You can feel essence flowing into you from (.*) as (.*) sinks its nails into its victim!",
    )
    .unwrap();
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

    #[test]
    fn minion_status_gags_and_upserts() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        let mut ctx = ctx(&mut stats, &mut automation);
        let line = "::..:. Tick [Hp: 44 (55) (+3), Sp: 1 (1), Ep: 200 (200)]";
        let mut styled = StyledLine::new(line);

        let out = NergalGuild::nergal_trigger(&mut ctx, &mut styled);

        assert!(styled.gag);
        assert!(out.actions.is_empty());
        assert!(out.lines.is_empty());
        let first = ctx.stats.nergal_minions[0].as_ref().unwrap();
        assert_eq!(first.name, "Tick");
        assert_eq!(first.hp, 44);
        assert_eq!(first.max_hp, 55);
        assert_eq!(first.sp, 1);
        assert_eq!(first.max_sp, 1);
        assert_eq!(first.ep, 200);
        assert_eq!(first.max_ep, 200);
    }

    #[test]
    fn unsummon_clears_minions() {
        let mut stats = Stats::default();
        stats.upsert_nergal_minion(NergalMinion {
            name: "a".into(),
            hp: 1,
            max_hp: 1,
            sp: 1,
            max_sp: 1,
            ep: 1,
            max_ep: 1,
        });
        let mut automation = Automation::new();
        let mut ctx = ctx(&mut stats, &mut automation);
        let mut styled = StyledLine::new(
            "Your connection to your parasite is severed completely. Host jerks violently couple of times and collapses.",
        );

        let _ = NergalGuild::nergal_trigger(&mut ctx, &mut styled);

        assert!(!ctx.stats.has_nergal_minions());
    }

    #[test]
    fn death_sends_nergal_score_alias() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        let mut ctx = ctx(&mut stats, &mut automation);
        let mut styled = StyledLine::new("DEAD, R.I.P.");

        let out = NergalGuild::nergal_trigger(&mut ctx, &mut styled);

        assert_eq!(out.actions.len(), 1);
        assert!(matches!(&out.actions[0], Action::Send(command) if command == "@nergal sc"));
    }

    #[test]
    fn potentia_full_echoes_green() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        let mut ctx = ctx(&mut stats, &mut automation);
        let mut styled = StyledLine::new("Potentia: 1000/1000");

        let out = NergalGuild::nergal_trigger(&mut ctx, &mut styled);

        assert_eq!(out.lines.len(), 1);
        assert_eq!(out.lines[0].plain_line, "***** POTENTIA IS FULL! *****");
        assert_eq!(out.lines[0].styled_chars[0].color, AnsiCode::Green);
    }

    #[test]
    fn scratch_aura_line_is_green() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        let mut ctx = ctx(&mut stats, &mut automation);
        let mut styled = StyledLine::new(
            "Foo manages to scratch Bar skin infecting the tissue under the skin with nasty disease",
        );

        let _ = NergalGuild::nergal_trigger(&mut ctx, &mut styled);

        assert_eq!(styled.styled_chars[0].color, AnsiCode::Green);
    }
}
