use lazy_static::lazy_static;
use regex::{Captures, Regex};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Waiter {
    pub pattern: Regex,
    pub actions: Vec<Action>,
    pub consume: bool,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum Action {
    Send(String),
    SetFlag(String, bool),
    SetVar(String, String),
    ClearFlag(String),
    ClearVar(String),
    IfFlag {
        flag: String,
        actions: Vec<Action>,
    },
    IfVarNotEmpty {
        var: String,
        actions: Vec<Action>,
    },
    AddWaiters(Vec<Waiter>),
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

    #[allow(dead_code)]
    pub fn set_flag(&mut self, key: &str, value: bool) {
        self.flags.insert(key.to_string(), value);
    }

    #[allow(dead_code)]
    pub fn clear_flag(&mut self, key: &str) {
        self.flags.remove(key);
    }

    #[allow(dead_code)]
    pub fn set_var(&mut self, key: &str, value: String) {
        self.vars.insert(key.to_string(), value);
    }

    #[allow(dead_code)]
    pub fn clear_var(&mut self, key: &str) {
        self.vars.remove(key);
    }

    pub fn add_waiter(&mut self, waiter: Waiter) {
        self.waiters.push(waiter);
    }

    pub fn apply_actions(&mut self, actions: Vec<Action>) -> Vec<String> {
        let mut sends = Vec::new();
        let mut to_add = Vec::new();
        self.execute_actions(&actions, &mut sends, &mut to_add);
        self.waiters.extend(to_add);
        sends
    }

    pub fn process_line(&mut self, line: &str) -> Vec<String> {
        let mut sends = Vec::new();
        let mut to_add = Vec::new();
        let mut retained = Vec::with_capacity(self.waiters.len());
        let waiters = std::mem::take(&mut self.waiters);

        for waiter in waiters {
            if waiter.pattern.is_match(line) {
                self.execute_actions(&waiter.actions, &mut sends, &mut to_add);
                if !waiter.consume {
                    retained.push(waiter);
                }
            } else {
                retained.push(waiter);
            }
        }

        self.waiters = retained;
        self.waiters.extend(to_add);
        sends
    }

    fn execute_actions(
        &mut self,
        actions: &[Action],
        sends: &mut Vec<String>,
        to_add: &mut Vec<Waiter>,
    ) {
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
                        self.execute_actions(actions, sends, to_add);
                    }
                }
                Action::IfVarNotEmpty { var, actions } => {
                    if self
                        .vars
                        .get(var)
                        .map(|value| !value.is_empty())
                        .unwrap_or(false)
                    {
                        self.execute_actions(actions, sends, to_add);
                    }
                }
                Action::AddWaiters(waiters) => {
                    to_add.extend(waiters.clone());
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
