use super::super::{ConnectionChannels, ConnectionCoordinator, ReconnectResult};
use super::fresh_session::FreshSessionPlan;

pub enum ReconnectAttemptResult {
    Connected(ConnectionChannels),
    Failed(String),
}

pub(crate) fn execute_reconnect(
    reconnect_in_progress: &mut bool,
    coordinator: &mut dyn ConnectionCoordinator,
    plan: FreshSessionPlan,
) -> ReconnectAttemptResult {
    let connection_id = plan.connection_id;

    let result = match coordinator.reconnect(connection_id) {
        ReconnectResult::Connected(channels) => ReconnectAttemptResult::Connected(channels),
        ReconnectResult::Failed(error) => ReconnectAttemptResult::Failed(error),
    };

    *reconnect_in_progress = false;
    result
}
