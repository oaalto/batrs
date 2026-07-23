use super::SessionLifecycle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputDisposition {
    KeepScrollback,
    ClearOutput,
}

impl SessionLifecycle {
    pub fn on_post_connect_login(&mut self, login_name: &str) -> Option<OutputDisposition> {
        let pre_connect = self.post_connect_name_snapshot.take()?;
        Some(match (pre_connect.as_deref(), login_name) {
            (Some(pre), post) if pre.eq_ignore_ascii_case(post) => {
                OutputDisposition::KeepScrollback
            }
            (None, "") => OutputDisposition::KeepScrollback,
            _ => OutputDisposition::ClearOutput,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_character_keeps_scrollback() {
        let mut lifecycle = SessionLifecycle::new();
        lifecycle.begin_fresh_session(Some("hero".to_string()));

        let disposition = lifecycle.on_post_connect_login("hero");

        assert_eq!(disposition, Some(OutputDisposition::KeepScrollback));
        assert!(lifecycle.on_post_connect_login("hero").is_none());
    }

    #[test]
    fn same_character_keeps_scrollback_case_insensitively() {
        let mut lifecycle = SessionLifecycle::new();
        lifecycle.begin_fresh_session(Some("Hero".to_string()));

        assert_eq!(
            lifecycle.on_post_connect_login("hero"),
            Some(OutputDisposition::KeepScrollback)
        );
    }

    #[test]
    fn different_character_clears_output() {
        let mut lifecycle = SessionLifecycle::new();
        lifecycle.begin_fresh_session(Some("hero".to_string()));

        assert_eq!(
            lifecycle.on_post_connect_login("other"),
            Some(OutputDisposition::ClearOutput)
        );
    }

    #[test]
    fn connect_before_login_clears_on_first_login() {
        let mut lifecycle = SessionLifecycle::new();
        lifecycle.begin_fresh_session(None);

        assert_eq!(
            lifecycle.on_post_connect_login("hero"),
            Some(OutputDisposition::ClearOutput)
        );
    }

    #[test]
    fn login_without_connect_is_noop() {
        let mut lifecycle = SessionLifecycle::new();

        assert!(lifecycle.on_post_connect_login("hero").is_none());
    }
}
