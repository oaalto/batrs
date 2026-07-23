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

pub const CEREMONY_DONE_FLAG: &str = "animist_ceremony_done";
pub const SEPARATING_SOUL_FLAG: &str = "animist_separating_soul";
pub const JOINING_SOUL_FLAG: &str = "animist_joining_soul";
pub const CONJURING_MOUNT_FLAG: &str = "animist_conjuring_mount";
pub const DISMISSING_MOUNT_FLAG: &str = "animist_dismissing_mount";

#[derive(Default)]
pub struct AnimistGuild {}

impl Guild for AnimistGuild {
    fn commands(&self) -> HashMap<String, Command> {
        self.get_commands()
    }

    fn triggers(&self) -> Vec<Trigger> {
        self.get_triggers()
    }

    fn register_automation(&self, automation: &mut Automation) {
        automation.set_flag(CEREMONY_DONE_FLAG, false);
        for flag in PENDING_FLAGS {
            automation.set_flag(flag, false);
        }

        automation.add_waiter(Waiter {
            pattern: START_CHANTING_REGEX.clone(),
            consume: false,
            actions: vec![Action::SetFlag(CEREMONY_DONE_FLAG.to_string(), false)],
        });

        automation.add_waiter(Waiter {
            pattern: CHANT_DONE_REGEX.clone(),
            consume: false,
            actions: clear_pending_actions(),
        });

        automation.add_waiter(Waiter {
            pattern: CEREMONY_DONE_REGEX.clone(),
            consume: false,
            actions: vec![
                Action::SetFlag(CEREMONY_DONE_FLAG.to_string(), true),
                queued_cast(SEPARATING_SOUL_FLAG, "cast 'separate soul'"),
                queued_cast(JOINING_SOUL_FLAG, "cast 'join soul'"),
                queued_cast(CONJURING_MOUNT_FLAG, "cast 'conjure animal soul'"),
                queued_cast(DISMISSING_MOUNT_FLAG, "cast 'animal soul link' dismiss"),
            ],
        });
    }
}

pub const PENDING_FLAGS: [&str; 4] = [
    SEPARATING_SOUL_FLAG,
    JOINING_SOUL_FLAG,
    CONJURING_MOUNT_FLAG,
    DISMISSING_MOUNT_FLAG,
];

pub fn clear_pending_actions() -> Vec<Action> {
    PENDING_FLAGS
        .into_iter()
        .map(|flag| Action::SetFlag(flag.to_string(), false))
        .collect()
}

fn queued_cast(flag: &str, logical_command: &str) -> Action {
    Action::IfFlag {
        flag: flag.to_string(),
        actions: vec![
            Action::Send(abilities::client_send_line(logical_command)),
            Action::SetFlag(flag.to_string(), false),
        ],
    }
}

static START_CHANTING_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^You start chanting\.$").unwrap());
static CHANT_DONE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^You are done with the chant\.$").unwrap());
static CEREMONY_DONE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^You perform the ceremony\.$").unwrap());

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ceremony_success_runs_queued_cast_once() {
        let mut automation = Automation::new();
        AnimistGuild::default().register_automation(&mut automation);

        automation.apply_actions(vec![Action::SetFlag(JOINING_SOUL_FLAG.to_string(), true)]);

        assert_eq!(
            automation.process_line("You perform the ceremony."),
            vec!["@cast 'join soul'"]
        );
        assert!(
            automation
                .process_line("You perform the ceremony.")
                .is_empty()
        );
    }

    #[test]
    fn chanting_resets_ceremony_and_done_with_chant_clears_queue() {
        let mut automation = Automation::new();
        AnimistGuild::default().register_automation(&mut automation);

        automation.apply_actions(vec![
            Action::SetFlag(CEREMONY_DONE_FLAG.to_string(), true),
            Action::SetFlag(SEPARATING_SOUL_FLAG.to_string(), true),
        ]);

        assert!(automation.process_line("You start chanting.").is_empty());
        assert!(!automation.flag_is_set(CEREMONY_DONE_FLAG));
        assert!(
            automation
                .process_line("You are done with the chant.")
                .is_empty()
        );
        assert!(!automation.flag_is_set(SEPARATING_SOUL_FLAG));
    }
}
