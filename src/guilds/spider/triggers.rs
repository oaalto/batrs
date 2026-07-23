use crate::ansi::{StyledLine, TextStyle};
use crate::guilds::SpiderGuild;
use crate::triggers::{TriggerEffects, TriggerFacts, TriggerLine};
use regex::Regex;
use std::sync::LazyLock;

const HEAVY_WEIGHT_EXPIRY: &str = "You feel lighter, but it doesn't seem to affect your weight!";
const QUEEN_SMILES_HELPS: &str = "Spider Queen smiles upon you and helps you control the demon.";
const LOSING_BODY_CONTROL: &str = "You are losing the battle for control over your body!";
const STAB_BLOCKED: &str = "You make a great stabbing maneuver, but your enemy blocks your attack.";

impl SpiderGuild {
    pub fn get_triggers(&self) -> Vec<crate::triggers::Trigger> {
        vec![Self::spider_highlight_trigger]
    }

    pub fn spider_highlight_trigger(
        line: &TriggerLine<'_>,
        _facts: &TriggerFacts,
    ) -> TriggerEffects {
        let line = line.plain_line.trim_end_matches('\r').trim();

        if line == HEAVY_WEIGHT_EXPIRY {
            return TriggerEffects::none()
                .style_line(TextStyle::BRIGHT_MAGENTA)
                .emit(heavy_weight_banner());
        }

        if line == QUEEN_SMILES_HELPS {
            return TriggerEffects::none().style_line(TextStyle::BRIGHT_GREEN);
        }

        if DEMON_HELP_YELLOW.is_match(line) {
            return TriggerEffects::none().style_line(TextStyle::BRIGHT_YELLOW);
        }

        if DEMON_POWER_BRIGHTRED.is_match(line) {
            return TriggerEffects::none().style_line(TextStyle::BRIGHT_RED);
        }

        if BLOOD_PALM_GREEN.is_match(line)
            || SHOWER_PALM_GREEN.is_match(line)
            || BLOOD_REFRESH_GREEN.is_match(line)
            || STAB_BLOOD_GREEN.is_match(line)
            || TWIST_BLADE_GREEN.is_match(line)
            || VENOM_CRINGE_GREEN.is_match(line)
            || POISON_FLOW_GREEN.is_match(line)
        {
            return TriggerEffects::none().style_line(TextStyle::BRIGHT_GREEN);
        }

        if line == LOSING_BODY_CONTROL || line == STAB_BLOCKED || FAIL_STAB_RED.is_match(line) {
            return TriggerEffects::none().style_line(TextStyle::BRIGHT_RED);
        }

        TriggerEffects::none()
    }
}

fn heavy_weight_banner() -> StyledLine {
    let mut banner = StyledLine::new("HEAVY WEIGHT OFF!");
    banner.set_line_style(TextStyle::BRIGHT_MAGENTA);
    banner
}

static DEMON_HELP_YELLOW: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(.+)'s demon feels easier to control than usual\.$").unwrap());
/// Intentional regex quirk: `\(.+)s spider demon` matches `'s'` without an apostrophe.
static DEMON_POWER_BRIGHTRED: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(.+)s spider demon draws power from you\.$").unwrap());
static BLOOD_PALM_GREEN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^A wound opens on your palm and you guide the (.+) blood at (.+)!$").unwrap()
});
static SHOWER_PALM_GREEN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^A shower of (.+) blood flies from (.+)'s palm at (.+)!$").unwrap()
});
static BLOOD_REFRESH_GREEN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^Stream of blood from (.+)'s wound flies to you, tasting refreshing!$").unwrap()
});
static STAB_BLOOD_GREEN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^You stab with (.+) causing blood to fly everywhere!$").unwrap());
static TWIST_BLADE_GREEN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^You twist your blade inside (.+)'s belly!$").unwrap());
static VENOM_CRINGE_GREEN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(.+) cringes from pain as your venomed blade bites into (.+) flesh!$").unwrap()
});
static POISON_FLOW_GREEN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(.+) suffers as poison from your blade flows into (.+) system!$").unwrap()
});
static FAIL_STAB_RED: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^You fail to stab (.+) with (.+)!$").unwrap());

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ansi::AnsiCode;
    use crate::triggers::{TriggerFacts, TriggerLine};

    fn run(line: &str) -> (TriggerEffects, StyledLine) {
        let output = SpiderGuild::spider_highlight_trigger(
            &TriggerLine::new(line),
            &TriggerFacts::default(),
        );
        let mut styled = StyledLine::new(line);
        output.apply_line_effects_to(&mut styled);
        (output, styled)
    }

    #[test]
    fn heavy_weight_line_magenta_and_banner() {
        let (out, styled) = run(HEAVY_WEIGHT_EXPIRY);

        assert_eq!(styled.styled_chars[0].color, AnsiCode::Magenta);
        assert!(styled.styled_chars[0].bold);
        assert_eq!(out.lines.len(), 1);
        assert_eq!(out.lines[0].plain_line, "HEAVY WEIGHT OFF!");
        assert_eq!(out.lines[0].styled_chars[0].color, AnsiCode::Magenta);
    }

    #[test]
    fn queen_smiles_green() {
        let (out, styled) = run(QUEEN_SMILES_HELPS);

        assert_eq!(styled.styled_chars[0].color, AnsiCode::Green);
        assert!(out.lines.is_empty());
    }

    #[test]
    fn demon_help_yellow() {
        let (_, styled) = run("Goblin's demon feels easier to control than usual.");

        assert_eq!(styled.styled_chars[0].color, AnsiCode::Yellow);
    }

    #[test]
    fn demon_power_red_bright_parity_regex() {
        let (_, styled) = run("Goblins spider demon draws power from you.");

        assert_eq!(styled.styled_chars[0].color, AnsiCode::Red);
        assert!(styled.styled_chars[0].bold);
    }

    #[test]
    fn stab_success_green_sample() {
        let (_, styled) = run("You stab with rusty sword causing blood to fly everywhere!");

        assert_eq!(styled.styled_chars[0].color, AnsiCode::Green);
    }
}
