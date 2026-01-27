use lazy_static::lazy_static;
use regex::{Captures, Regex};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Waiter {
    pub pattern: Regex,
    pub actions: Vec<Action>,
    pub consume: bool,
}

#[derive(Clone, Debug)]
pub enum Action {
    Send(String),
    SetFlag(String, bool),
    SetVar(String, String),
    ClearFlag(String),
    ClearVar(String),
    IfFlag { flag: String, actions: Vec<Action> },
}

#[derive(Default, Debug)]
pub struct Automation {
    flags: HashMap<String, bool>,
    vars: HashMap<String, String>,
    waiters: Vec<Waiter>,
}

impl Automation {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn snapshot_flags(&self) -> HashMap<String, bool> {
        self.flags.clone()
    }

    pub fn flag_is_set(&self, key: &str) -> bool {
        *self.flags.get(key).unwrap_or(&false)
    }

    pub fn set_flag(&mut self, key: &str, value: bool) {
        self.flags.insert(key.to_string(), value);
    }

    pub fn set_var(&mut self, key: &str, value: String) {
        self.vars.insert(key.to_string(), value);
    }

    pub fn add_waiter(&mut self, waiter: Waiter) {
        self.waiters.push(waiter);
    }

    pub fn apply_actions(&mut self, actions: Vec<Action>) -> Vec<String> {
        let mut sends = Vec::new();
        self.execute_actions(&actions, &mut sends);
        sends
    }

    pub fn process_line(&mut self, line: &str) -> Vec<String> {
        let mut sends = Vec::new();
        let mut retained = Vec::with_capacity(self.waiters.len());
        let waiters = std::mem::take(&mut self.waiters);

        for waiter in waiters {
            if waiter.pattern.is_match(line) {
                self.execute_actions(&waiter.actions, &mut sends);
                if !waiter.consume {
                    retained.push(waiter);
                }
            } else {
                retained.push(waiter);
            }
        }

        self.waiters = retained;
        sends
    }

    fn execute_actions(&mut self, actions: &[Action], sends: &mut Vec<String>) {
        for action in actions {
            match action {
                Action::Send(template) => {
                    sends.push(self.expand_template(template));
                }
                Action::SetFlag(flag, value) => {
                    self.flags.insert(flag.clone(), *value);
                }
                Action::ClearFlag(flag) => {
                    self.flags.remove(flag);
                }
                Action::SetVar(key, value) => {
                    self.vars.insert(key.clone(), value.clone());
                }
                Action::ClearVar(key) => {
                    self.vars.remove(key);
                }
                Action::IfFlag { flag, actions } => {
                    if *self.flags.get(flag).unwrap_or(&false) {
                        self.execute_actions(actions, sends);
                    }
                }
            }
        }
    }

    fn expand_template(&self, template: &str) -> String {
        TEMPLATE_REGEX
            .replace_all(template, |caps: &Captures<'_>| {
                let key = &caps[1];
                self.vars.get(key).cloned().unwrap_or_default()
            })
            .to_string()
    }
}

lazy_static! {
    static ref TEMPLATE_REGEX: Regex = Regex::new(r"\{([A-Za-z0-9_]+)\}").unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_actions_expands_templates_and_sets_flags() {
        let mut automation = Automation::new();
        let sends = automation.apply_actions(vec![
            Action::SetVar("rig".to_string(), "satchel".to_string()),
            Action::SetFlag("in_battle".to_string(), true),
            Action::Send("put all essence in {rig}".to_string()),
        ]);

        assert_eq!(sends, vec!["put all essence in satchel"]);
        assert!(automation.flag_is_set("in_battle"));
    }

    #[test]
    fn apply_actions_replaces_missing_var_with_empty() {
        let mut automation = Automation::new();
        let sends = automation.apply_actions(vec![Action::Send("carry {missing}".to_string())]);

        assert_eq!(sends, vec!["carry "]);
    }

    #[test]
    fn process_line_handles_waiter_consumption() {
        let mut automation = Automation::new();
        automation.add_waiter(Waiter {
            pattern: Regex::new("hello").unwrap(),
            actions: vec![Action::Send("hi".to_string())],
            consume: true,
        });

        let first = automation.process_line("hello");
        let second = automation.process_line("hello");

        assert_eq!(first, vec!["hi"]);
        assert!(second.is_empty());
    }

    #[test]
    fn process_line_keeps_non_consuming_waiters() {
        let mut automation = Automation::new();
        automation.add_waiter(Waiter {
            pattern: Regex::new("pong").unwrap(),
            actions: vec![Action::Send("ping".to_string())],
            consume: false,
        });

        let first = automation.process_line("pong");
        let second = automation.process_line("pong");

        assert_eq!(first, vec!["ping"]);
        assert_eq!(second, vec!["ping"]);
    }
}
