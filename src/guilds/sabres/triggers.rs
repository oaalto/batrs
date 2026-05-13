use crate::abilities;
use crate::ansi::{StyledLine, TextStyle};
use crate::automation::Action;
use crate::guilds::sabres::{SABRE_WEAPON_VAR, SabresGuild};
use crate::triggers::{TriggerContext, TriggerOutput};
use lazy_static::lazy_static;
use regex::Regex;

const FENCE_RED_HILITE: &[&str] = &[
    "Your attack is skillfully avoided by your enemy, an elegant move!",
    "You frantically try to grab a weapon, but cannot get a grip in time.",
];

const FENCE_GREEN_HILITE: &[&str] =
    &["Your superb skill allows you to execute your technique faster!"];

lazy_static! {
    static ref LOUNGING_YELLOW: &'static str = "You are in a mood for a bit of lounging again.";
    static ref LOUNGING_GREEN: &'static str =
        "You are done with your lounging for now, you feel better!";
    static ref BATTLE_CADENCE: &'static str = "Your battle cadence grants you another attack!";
    static ref FUMBLING_LINE: &'static str =
        "You are still fumbling with your weapon. It is one slippery thing!";
    static ref GLOVEKNOCK_WIELD_LINES: Vec<Regex> = vec![
        Regex::new(r"^You swing your arm and hit (.+) straight on the nose, bloodying it bad!$")
            .unwrap(),
        Regex::new(r"^You slam (.+) on the jaw very hard making (.+) cry out in pain!$").unwrap(),
        Regex::new(r"^With a swift and precise punch you strike (.+) on his face,$").unwrap(),
    ];
    static ref GREEN_WIELD: Regex = Regex::new(r"^You wield (.+) in your right (.+)\.$").unwrap();
}

impl SabresGuild {
    pub fn get_triggers(&self) -> Vec<crate::triggers::Trigger> {
        vec![
            Self::notify_triggers,
            Self::fence_hilites_trigger,
            Self::proficiency_blue_trigger,
            Self::gloveknock_wield_trigger,
            Self::green_wield_trigger,
        ]
    }

    fn configured_weapon(ctx: &TriggerContext<'_>) -> Option<String> {
        let raw = ctx.automation.get_var(SABRE_WEAPON_VAR)?;
        let trimmed = raw.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    }

    pub fn notify_triggers(
        _ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> TriggerOutput {
        let plain = styled_line.plain_line.as_str();
        if plain == *BATTLE_CADENCE {
            styled_line.set_line_style(TextStyle::BLUE);
        } else if plain == *LOUNGING_YELLOW {
            styled_line.set_line_style(TextStyle::BRIGHT_YELLOW);
        } else if plain == *LOUNGING_GREEN {
            styled_line.set_line_style(TextStyle::BRIGHT_GREEN);
        } else if plain == *FUMBLING_LINE {
            styled_line.set_line_style(TextStyle::BRIGHT_RED);
        }
        TriggerOutput::default()
    }

    pub fn fence_hilites_trigger(
        _ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> TriggerOutput {
        let plain = styled_line.plain_line.as_str();
        if FENCE_RED_HILITE.contains(&plain) {
            styled_line.set_line_style(TextStyle::BRIGHT_RED);
        } else if FENCE_GREEN_HILITE.contains(&plain) {
            styled_line.set_line_style(TextStyle::BRIGHT_GREEN);
        }
        TriggerOutput::default()
    }

    pub fn proficiency_blue_trigger(
        _ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> TriggerOutput {
        if styled_line
            .plain_line
            .starts_with("You feel more proficient in")
        {
            styled_line.set_line_style(TextStyle::BLUE);
        }
        TriggerOutput::default()
    }

    pub fn gloveknock_wield_trigger(
        ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> TriggerOutput {
        let mut output = TriggerOutput::default();
        let plain = styled_line.plain_line.as_str();
        let matches_hit = GLOVEKNOCK_WIELD_LINES.iter().any(|re| re.is_match(plain));
        let matches_fail =
            plain == "You frantically try to grab a weapon, but cannot get a grip in time.";
        match Self::configured_weapon(ctx) {
            Some(weapon) if matches_hit || matches_fail => {
                output
                    .actions
                    .push(Action::Send(abilities::client_send_line(&format!(
                        "wield {weapon}"
                    ))));
            }
            _ => {}
        }
        output
    }

    pub fn green_wield_trigger(
        _ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> TriggerOutput {
        if GREEN_WIELD.is_match(styled_line.plain_line.as_str()) {
            styled_line.set_line_style(TextStyle::GREEN);
        }
        TriggerOutput::default()
    }
}
