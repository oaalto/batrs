use lazy_static::lazy_static;
use regex::Regex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginState {
    Choice,
    Name,
    Password,
    LoggedIn,
}

#[derive(Debug)]
pub struct SessionState {
    login_state: LoginState,
    login_name: Option<String>,
    last_login_input: Option<String>,
}

impl SessionState {
    pub fn new() -> Self {
        Self {
            login_state: LoginState::Choice,
            login_name: None,
            last_login_input: None,
        }
    }

    pub fn login_state(&self) -> LoginState {
        self.login_state
    }

    pub fn is_logged_in(&self) -> bool {
        self.login_state == LoginState::LoggedIn
    }

    pub fn update_login_state(&mut self, line: &str) -> bool {
        let line = line.trim_end();
        if line == "You entered a wrong password!" {
            self.login_state = LoginState::Choice;
            self.login_name = None;
            self.last_login_input = None;
            return true;
        }

        if line == "Please enter your choice or name:" {
            self.login_state = LoginState::Choice;
            self.login_name = None;
            self.last_login_input = None;
            return true;
        }

        if line.starts_with("What is your name:") {
            self.login_state = LoginState::Name;
            return true;
        }

        if line.starts_with("Password:") {
            self.login_state = LoginState::Password;
            if self.login_name.is_none() {
                self.login_name = self.last_login_input.clone();
            }
            return true;
        }

        if !self.is_logged_in()
            && (PROMPT_REGEX.is_match(line) || SHORT_SCORE_REGEX.is_match(line))
        {
            self.login_state = LoginState::LoggedIn;
        }

        false
    }

    pub fn set_login_name(&mut self, name: String) {
        self.login_name = Some(name);
    }

    pub fn set_last_login_input(&mut self, input: String) {
        self.last_login_input = Some(input);
    }

    pub fn login_name(&self) -> Option<&str> {
        self.login_name.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::{LoginState, SessionState};

    #[test]
    fn tracks_login_flow_and_logged_in_prompt() {
        let mut session = SessionState::new();

        assert_eq!(session.login_state(), LoginState::Choice);
        assert!(session.update_login_state("Please enter your choice or name:"));
        assert_eq!(session.login_state(), LoginState::Choice);

        assert!(session.update_login_state("What is your name:"));
        assert_eq!(session.login_state(), LoginState::Name);

        session.set_last_login_input("tester".to_string());
        assert!(session.update_login_state("Password:"));
        assert_eq!(session.login_state(), LoginState::Password);

        assert!(!session.is_logged_in());
        assert!(!session.update_login_state("Random text"));
        assert!(!session.is_logged_in());

        assert!(!session.update_login_state("Hp:1/2 Sp:3/4 Ep:5/6 Exp:7 >"));
        assert!(session.is_logged_in());
    }
}

lazy_static! {
    static ref PROMPT_REGEX: Regex =
        Regex::new(r"^Hp:(.+)/(.+) Sp:(.+)/(.+) Ep:(.+)/(.+) Exp:(.+) >$").unwrap();
    static ref SHORT_SCORE_REGEX: Regex =
        Regex::new(r"^H:(.+)/(.+) \[(.*)\] S:(.+)/(.+) \[(.*)\] E:(.+)/(.+) \[(.*)\] \\$:(.+) \[(.*)\] exp:(.+) \[(.*)\]$")
            .unwrap();
}
