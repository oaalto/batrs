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
pub struct DiscipleGuild {}

impl Guild for DiscipleGuild {
    fn commands(&self) -> HashMap<String, Command> {
        self.get_commands()
    }

    fn triggers(&self) -> Vec<Trigger> {
        self.get_triggers()
    }

    fn register_automation(&self, automation: &mut Automation) {
        automation.add_waiter(Waiter {
            pattern: CHAOTIC_SPAWN_TRANSFORM_REGEX.clone(),
            consume: false,
            actions: vec![Action::Send("@wearall".to_string())],
        });
    }
}

lazy_static! {
    static ref CHAOTIC_SPAWN_TRANSFORM_REGEX: Regex = Regex::new(
        "The pain increases as your body starts to push out organs and limbs that should not be there."
    )
    .unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::automation::Automation;

    #[test]
    fn chaotic_spawn_triggers_wearall() {
        let mut automation = Automation::new();
        DiscipleGuild::default().register_automation(&mut automation);

        assert_eq!(
            automation.process_line(
                "The pain increases as your body starts to push out organs and limbs that should not be there."
            ),
            vec!["@wearall"]
        );
    }

    #[test]
    fn wearall_trigger_fires_only_on_match() {
        let mut automation = Automation::new();
        DiscipleGuild::default().register_automation(&mut automation);

        assert!(automation.process_line("Some random line").is_empty());
    }
}
