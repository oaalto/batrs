mod commands;
mod triggers;

use crate::automation::{Action, Automation, Waiter};
use crate::command::Command;
use crate::guilds::Guild;
use crate::triggers::Trigger;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;

pub const DISMOUNTED_FLAG: &str = "tzarakk_dismounted";
pub const MOUNT_SUMMONED_FLAG: &str = "tzarakk_mount_summoned";
pub const TZARAKK_MOUNT_VAR: &str = "tzarakk_mount";

#[derive(Default)]
pub struct TzarakkGuild {}

impl Guild for TzarakkGuild {
    fn commands(&self) -> HashMap<String, Command> {
        self.get_commands()
    }

    fn triggers(&self) -> Vec<Trigger> {
        self.get_triggers()
    }

    fn register_automation(&self, automation: &mut Automation) {
        automation.set_flag(DISMOUNTED_FLAG, false);
        automation.set_flag(MOUNT_SUMMONED_FLAG, false);
        automation.set_var(TZARAKK_MOUNT_VAR, "Vedir".to_string());

        // Sleep and camping outcomes trigger re-mounting.
        automation.add_waiter(Waiter {
            pattern: REMOUNT_REGEX.clone(),
            consume: false,
            actions: vec![Action::IfFlag {
                flag: MOUNT_SUMMONED_FLAG.to_string(),
                actions: vec![Action::Send("@mount {tzarakk_mount}".to_string())],
            }],
        });
    }
}

lazy_static! {
    static ref REMOUNT_REGEX: Regex = Regex::new(concat!(
        r"^(You awaken from your short rest, and feel slightly better\.",
        r"|You don't quite feel like camping at the moment\.",
        r"|You wake up!",
        r"|It'll be inconvenient to camp here with all this water\.",
        r"|You fail to reach the state of inner harmony\.",
        r"|Something disturbs you and you cannot concentrate any longer\.",
        r"|You don't feel to be in harmony with yourself\.)$"
    ))
    .unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    const REMOUNT_LINES: [&str; 7] = [
        "You awaken from your short rest, and feel slightly better.",
        "You don't quite feel like camping at the moment.",
        "You wake up!",
        "It'll be inconvenient to camp here with all this water.",
        "You fail to reach the state of inner harmony.",
        "Something disturbs you and you cannot concentrate any longer.",
        "You don't feel to be in harmony with yourself.",
    ];

    #[test]
    fn automation_registration_sets_initial_state() {
        let mut automation = Automation::new();
        let guild = TzarakkGuild::default();

        guild.register_automation(&mut automation);

        assert!(!automation.flag_is_set(DISMOUNTED_FLAG));
        assert!(!automation.flag_is_set(MOUNT_SUMMONED_FLAG));
        assert_eq!(
            automation.get_var(TZARAKK_MOUNT_VAR),
            Some(&"Vedir".to_string())
        );
    }

    #[test]
    fn remount_lines_mount_tracked_tzarakk_mount() {
        let mut automation = Automation::new();
        TzarakkGuild::default().register_automation(&mut automation);
        automation.set_flag(MOUNT_SUMMONED_FLAG, true);
        automation.set_var(TZARAKK_MOUNT_VAR, "Orthos".to_string());

        for line in REMOUNT_LINES {
            assert_eq!(automation.process_line(line), vec!["@mount Orthos"]);
        }
    }

    #[test]
    fn remount_lines_do_nothing_without_summoned_mount() {
        let mut automation = Automation::new();
        TzarakkGuild::default().register_automation(&mut automation);
        automation.set_var(TZARAKK_MOUNT_VAR, "Orthos".to_string());

        for line in REMOUNT_LINES {
            assert!(automation.process_line(line).is_empty());
        }
    }
}
