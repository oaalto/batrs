mod connect_command;
mod fresh_session;
mod reconnect;
mod stale_events;

pub use connect_command::{complete_connect, prepare_connect};
pub use fresh_session::{FreshSessionPlan, FreshSessionReset};
pub use reconnect::ReconnectAttemptResult;

use super::{ConnectionId, INITIAL_CONNECTION_ID};

pub struct SessionLifecycle {
    active_connection_id: ConnectionId,
    reconnect_in_progress: bool,
    // ponytail: ticket 02 reads this for post-connect scrollback disposition
    pre_connect_login_name: Option<String>,
}

impl SessionLifecycle {
    pub fn new() -> Self {
        Self {
            active_connection_id: INITIAL_CONNECTION_ID,
            reconnect_in_progress: false,
            pre_connect_login_name: None,
        }
    }

    pub fn is_stale(&self, connection_id: ConnectionId) -> bool {
        stale_events::is_stale(self.active_connection_id, connection_id)
    }

    pub fn begin_fresh_session(
        &mut self,
        pre_connect_login_name: Option<String>,
    ) -> FreshSessionPlan {
        self.pre_connect_login_name = pre_connect_login_name;
        self.active_connection_id = self.active_connection_id.saturating_add(1);
        FreshSessionPlan::new(self.active_connection_id)
    }

    pub(crate) fn reconnect_in_progress_mut(&mut self) -> &mut bool {
        &mut self.reconnect_in_progress
    }

    #[cfg(test)]
    pub fn set_reconnect_in_progress(&mut self, in_progress: bool) {
        self.reconnect_in_progress = in_progress;
    }
}

#[cfg(test)]
mod tests {
    use super::super::{INITIAL_CONNECTION_ID, ReconnectResult};
    use super::*;
    use crate::app::fake_connection_coordinator::{FakeConnectionCoordinator, connection_channels};

    #[test]
    fn begin_fresh_session_bumps_connection_id() {
        let mut lifecycle = SessionLifecycle::new();

        let plan = lifecycle.begin_fresh_session(Some("hero".to_string()));

        assert_eq!(plan.connection_id, 1);
        let second = lifecycle.begin_fresh_session(None);
        assert_eq!(second.connection_id, 2);
    }

    #[test]
    fn duplicate_reconnect_reports_already_in_progress() {
        let (_coordinator, calls, _results) = FakeConnectionCoordinator::new(Vec::new());
        let mut lifecycle = SessionLifecycle::new();
        lifecycle.set_reconnect_in_progress(true);

        let plan = prepare_connect(&mut lifecycle, None);

        assert!(plan.is_err());
        assert!(calls.borrow().is_empty());
    }

    #[test]
    fn failed_reconnect_clears_guard_and_retains_fresh_session_id() {
        let (mut coordinator, calls, _results) =
            FakeConnectionCoordinator::new(vec![ReconnectResult::Failed(
                "socket refused".to_string(),
            )]);
        let mut lifecycle = SessionLifecycle::new();

        let plan = prepare_connect(&mut lifecycle, Some("hero".to_string())).unwrap();
        let result = complete_connect(&mut lifecycle, &mut coordinator, plan);

        assert_eq!(*calls.borrow(), vec![1]);
        assert!(matches!(result, ReconnectAttemptResult::Failed(_)));
        assert!(!*lifecycle.reconnect_in_progress_mut());
        assert!(lifecycle.is_stale(INITIAL_CONNECTION_ID));
    }

    #[test]
    fn successful_reconnect_returns_channels_and_clears_guard() {
        let (channels, _command_receiver, _event_sender) = connection_channels();
        let (mut coordinator, calls, _results) =
            FakeConnectionCoordinator::new(vec![ReconnectResult::Connected(channels)]);
        let mut lifecycle = SessionLifecycle::new();

        let plan = prepare_connect(&mut lifecycle, None).unwrap();
        let result = complete_connect(&mut lifecycle, &mut coordinator, plan);

        assert_eq!(*calls.borrow(), vec![1]);
        assert!(matches!(result, ReconnectAttemptResult::Connected(_)));
        assert!(!*lifecycle.reconnect_in_progress_mut());
    }

    #[test]
    fn stale_events_use_active_connection_id() {
        let mut lifecycle = SessionLifecycle::new();
        lifecycle.begin_fresh_session(None);

        assert!(lifecycle.is_stale(INITIAL_CONNECTION_ID));
        assert!(!lifecycle.is_stale(1));
    }

    #[test]
    fn retry_after_failure_uses_next_connection_id() {
        let (channels, _command_receiver, _event_sender) = connection_channels();
        let (mut coordinator, calls, _results) = FakeConnectionCoordinator::new(vec![
            ReconnectResult::Failed("offline".to_string()),
            ReconnectResult::Connected(channels),
        ]);
        let mut lifecycle = SessionLifecycle::new();

        let first_plan = prepare_connect(&mut lifecycle, None).unwrap();
        let first = complete_connect(&mut lifecycle, &mut coordinator, first_plan);
        let second_plan = prepare_connect(&mut lifecycle, None).unwrap();
        let second = complete_connect(&mut lifecycle, &mut coordinator, second_plan);

        assert_eq!(*calls.borrow(), vec![1, 2]);
        assert!(matches!(first, ReconnectAttemptResult::Failed(_)));
        assert!(matches!(second, ReconnectAttemptResult::Connected(_)));
        assert!(!lifecycle.is_stale(2));
        assert!(lifecycle.is_stale(1));
    }
}
