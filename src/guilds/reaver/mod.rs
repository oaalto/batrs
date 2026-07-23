mod commands;
mod triggers;

use crate::abilities;
use crate::automation::{Action, Automation, Waiter};
use crate::command::Command;
use crate::guilds::Guild;
use crate::triggers::Trigger;
use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

#[derive(Default)]
pub struct ReaverGuild {}

impl Guild for ReaverGuild {
    fn commands(&self) -> HashMap<String, Command> {
        self.get_commands()
    }

    fn triggers(&self) -> Vec<Trigger> {
        self.get_triggers()
    }

    fn register_automation(&self, automation: &mut Automation) {
        automation.add_waiter(Waiter {
            pattern: PRAYER_DONE_REGEX.clone(),
            consume: false,
            actions: vec![
                Action::SetFlag("prayer_done".to_string(), true),
                Action::IfFlag {
                    flag: "cast_shattered_feast".to_string(),
                    actions: vec![
                        Action::Send(abilities::client_send_line(
                            "cast 'shattered feast' amount 100",
                        )),
                        Action::ClearFlag("cast_shattered_feast".to_string()),
                    ],
                },
                Action::IfFlag {
                    flag: "cast_blood_seeker".to_string(),
                    actions: vec![
                        Action::Send(abilities::client_send_line(
                            "cast 'blood seeker' amount 100",
                        )),
                        Action::ClearFlag("cast_blood_seeker".to_string()),
                    ],
                },
                Action::IfFlag {
                    flag: "cast_call_armour".to_string(),
                    actions: vec![
                        Action::Send(abilities::client_send_line(
                            "cast 'call armour' amount {call_armour_amount}",
                        )),
                        Action::ClearFlag("cast_call_armour".to_string()),
                    ],
                },
                Action::IfFlag {
                    flag: "cast_spirit_drain".to_string(),
                    actions: vec![
                        Action::Send(abilities::client_send_line(
                            "cast 'spirit drain' {spirit_drain_target} amount 100",
                        )),
                        Action::ClearFlag("cast_spirit_drain".to_string()),
                    ],
                },
                Action::IfFlag {
                    flag: "cast_black_hole".to_string(),
                    actions: vec![
                        Action::Send(abilities::client_send_line("cast 'black hole'")),
                        Action::ClearFlag("cast_black_hole".to_string()),
                    ],
                },
            ],
        });

        automation.add_waiter(Waiter {
            pattern: START_CHANT_REGEX.clone(),
            consume: false,
            actions: vec![Action::SetFlag("prayer_done".to_string(), false)],
        });

        automation.add_waiter(Waiter {
            pattern: MEDITATION_RESET_REGEX.clone(),
            consume: false,
            actions: vec![
                Action::ClearFlag("cast_shattered_feast".to_string()),
                Action::ClearFlag("cast_blood_seeker".to_string()),
                Action::ClearFlag("cast_call_armour".to_string()),
                Action::ClearFlag("cast_spirit_drain".to_string()),
                Action::ClearFlag("cast_black_hole".to_string()),
                Action::ClearVar("spirit_drain_target".to_string()),
                Action::ClearVar("call_armour_amount".to_string()),
            ],
        });
    }
}

static PRAYER_DONE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        "You bow your head and concentrate on the destructive energies you have collected\\.",
    )
    .unwrap()
});
static START_CHANT_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("You start chanting\\.").unwrap());
static MEDITATION_RESET_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("That I have set free, return to me").unwrap());
