mod commands;
mod triggers;

use crate::automation::{Action, Automation, Waiter};
use crate::command::Command;
use crate::guilds::Guild;
use crate::triggers::Trigger;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;

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
                        Action::Send("@cast shattered feast at amount 100".to_string()),
                        Action::ClearFlag("cast_shattered_feast".to_string()),
                    ],
                },
                Action::IfFlag {
                    flag: "cast_blood_seeker".to_string(),
                    actions: vec![
                        Action::Send("@cast blood seeker at amount 100".to_string()),
                        Action::ClearFlag("cast_blood_seeker".to_string()),
                    ],
                },
                Action::IfFlag {
                    flag: "cast_call_armour".to_string(),
                    actions: vec![
                        Action::Send(
                            "@cast call armour at amount {call_armour_amount}".to_string(),
                        ),
                        Action::ClearFlag("cast_call_armour".to_string()),
                    ],
                },
                Action::IfFlag {
                    flag: "cast_spirit_drain".to_string(),
                    actions: vec![
                        Action::Send(
                            "@cast spirit drain at {spirit_drain_target} amount 100".to_string(),
                        ),
                        Action::ClearFlag("cast_spirit_drain".to_string()),
                    ],
                },
                Action::IfFlag {
                    flag: "cast_black_hole".to_string(),
                    actions: vec![
                        Action::Send("@cast black hole".to_string()),
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

lazy_static! {
    static ref PRAYER_DONE_REGEX: Regex = Regex::new(
        "You bow your head and concentrate on the destructive energies you have collected\\."
    )
    .unwrap();
    static ref START_CHANT_REGEX: Regex = Regex::new("You start chanting\\.").unwrap();
    static ref MEDITATION_RESET_REGEX: Regex =
        Regex::new("That I have set free, return to me").unwrap();
}
