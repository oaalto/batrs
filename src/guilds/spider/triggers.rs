use crate::ansi::{AnsiCode, StyledLine};
use crate::guilds::SpiderGuild;
use crate::triggers::{TriggerContext, TriggerOutput};
use lazy_static::lazy_static;
use regex::Regex;

const HEAVY_WEIGHT_EXPIRY: &str = "You feel lighter, but it doesn't seem to affect your weight!";
const QUEEN_SMILES_HELPS: &str = "Spider Queen smiles upon you and helps you control the demon.";
const LOSING_BODY_CONTROL: &str = "You are losing the battle for control over your body!";
const STAB_BLOCKED: &str = "You make a great stabbing maneuver, but your enemy blocks your attack.";

impl SpiderGuild {
    pub fn get_triggers(&self) -> Vec<crate::triggers::Trigger> {
        vec![Self::spider_highlight_trigger]
    }

    pub fn spider_highlight_trigger(
        _ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> TriggerOutput {
        let mut output = TriggerOutput::default();
        let line = styled_line.plain_line.trim_end_matches('\r').trim();

        if line == HEAVY_WEIGHT_EXPIRY {
            styled_line.set_line_color(AnsiCode::Magenta, true);
            output.lines.push(heavy_weight_banner());
            return output;
        }

        if line == QUEEN_SMILES_HELPS {
            styled_line.set_line_color(AnsiCode::Green, true);
            return output;
        }

        if DEMON_HELP_YELLOW.is_match(line) {
            styled_line.set_line_color(AnsiCode::Yellow, true);
            return output;
        }

        if DEMON_POWER_BRIGHTRED.is_match(line) {
            styled_line.set_line_color(AnsiCode::Red, true);
            return output;
        }

        if BLOOD_PALM_GREEN.is_match(line)
            || SHOWER_PALM_GREEN.is_match(line)
            || BLOOD_REFRESH_GREEN.is_match(line)
            || STAB_BLOOD_GREEN.is_match(line)
            || TWIST_BLADE_GREEN.is_match(line)
            || VENOM_CRINGE_GREEN.is_match(line)
            || POISON_FLOW_GREEN.is_match(line)
        {
            styled_line.set_line_color(AnsiCode::Green, true);
            return output;
        }

        if line == LOSING_BODY_CONTROL || line == STAB_BLOCKED || FAIL_STAB_RED.is_match(line) {
            styled_line.set_line_color(AnsiCode::Red, true);
            return output;
        }

        output
    }
}

fn heavy_weight_banner() -> StyledLine {
    let mut banner = StyledLine::new("HEAVY WEIGHT OFF!");
    banner.set_line_color(AnsiCode::Magenta, true);
    banner
}

lazy_static! {
    static ref DEMON_HELP_YELLOW: Regex = Regex::new(
        r"^(.+)'s demon feels easier to control than usual\.$",
    ).unwrap();

    /// Intentional regex quirk: `\(.+)s spider demon` matches `'s'` without an apostrophe.
    static ref DEMON_POWER_BRIGHTRED: Regex = Regex::new(
        r"^(.+)s spider demon draws power from you\.$",
    ).unwrap();

    static ref BLOOD_PALM_GREEN: Regex = Regex::new(
        r"^A wound opens on your palm and you guide the (.+) blood at (.+)!$",
    ).unwrap();

    static ref SHOWER_PALM_GREEN: Regex = Regex::new(
        r"^A shower of (.+) blood flies from (.+)'s palm at (.+)!$",
    ).unwrap();

    static ref BLOOD_REFRESH_GREEN: Regex = Regex::new(
        r"^Stream of blood from (.+)'s wound flies to you, tasting refreshing!$",
    ).unwrap();

    static ref STAB_BLOOD_GREEN: Regex =
        Regex::new(r"^You stab with (.+) causing blood to fly everywhere!$").unwrap();

    static ref TWIST_BLADE_GREEN: Regex =
        Regex::new(r"^You twist your blade inside (.+)'s belly!$").unwrap();

    static ref VENOM_CRINGE_GREEN: Regex = Regex::new(
        r"^(.+) cringes from pain as your venomed blade bites into (.+) flesh!$",
    ).unwrap();

    static ref POISON_FLOW_GREEN: Regex = Regex::new(
        r"^(.+) suffers as poison from your blade flows into (.+) system!$",
    ).unwrap();

    static ref FAIL_STAB_RED: Regex = Regex::new(r"^You fail to stab (.+) with (.+)!$").unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn heavy_weight_line_magenta_and_banner() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        let mut ctx = ctx(&mut stats, &mut automation);
        let mut styled = StyledLine::new(HEAVY_WEIGHT_EXPIRY);

        let out = SpiderGuild::spider_highlight_trigger(&mut ctx, &mut styled);

        assert_eq!(styled.styled_chars[0].color, AnsiCode::Magenta);
        assert!(styled.styled_chars[0].bold);
        assert_eq!(out.lines.len(), 1);
        assert_eq!(out.lines[0].plain_line, "HEAVY WEIGHT OFF!");
        assert_eq!(out.lines[0].styled_chars[0].color, AnsiCode::Magenta);
    }

    #[test]
    fn queen_smiles_green() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        let mut ctx = ctx(&mut stats, &mut automation);
        let mut styled = StyledLine::new(QUEEN_SMILES_HELPS);

        let out = SpiderGuild::spider_highlight_trigger(&mut ctx, &mut styled);

        assert_eq!(styled.styled_chars[0].color, AnsiCode::Green);
        assert!(out.lines.is_empty());
    }

    #[test]
    fn demon_help_yellow() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        let mut ctx = ctx(&mut stats, &mut automation);
        let mut styled = StyledLine::new("Goblin's demon feels easier to control than usual.");

        let _ = SpiderGuild::spider_highlight_trigger(&mut ctx, &mut styled);

        assert_eq!(styled.styled_chars[0].color, AnsiCode::Yellow);
    }

    #[test]
    fn demon_power_red_bright_parity_regex() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        let mut ctx = ctx(&mut stats, &mut automation);
        let mut styled = StyledLine::new("Goblins spider demon draws power from you.");

        let _ = SpiderGuild::spider_highlight_trigger(&mut ctx, &mut styled);

        assert_eq!(styled.styled_chars[0].color, AnsiCode::Red);
        assert!(styled.styled_chars[0].bold);
    }

    #[test]
    fn stab_success_green_sample() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        let mut ctx = ctx(&mut stats, &mut automation);
        let mut styled =
            StyledLine::new("You stab with rusty sword causing blood to fly everywhere!");

        let _ = SpiderGuild::spider_highlight_trigger(&mut ctx, &mut styled);

        assert_eq!(styled.styled_chars[0].color, AnsiCode::Green);
    }
}
