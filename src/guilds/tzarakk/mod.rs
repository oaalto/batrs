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

        // Wake triggers re-mounting
        automation.add_waiter(Waiter {
            pattern: WAKE_REGEX.clone(),
            consume: false,
            actions: vec![Action::IfFlag {
                flag: MOUNT_SUMMONED_FLAG.to_string(),
                actions: vec![Action::Send("@mount {tzarakk_mount}".to_string())],
            }],
        });
    }
}

lazy_static! {
    static ref WAKE_REGEX: Regex =
        Regex::new(r"You (awaken from your short rest|wake up)!").unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
