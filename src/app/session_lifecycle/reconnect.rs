use super::super::{ConnectionCoordinator, ReconnectResult};
use super::fresh_session::FreshSessionPlan;

pub use super::super::ReconnectResult as ReconnectAttemptResult;

pub(crate) fn execute_reconnect(
    reconnect_in_progress: &mut bool,
    coordinator: &mut dyn ConnectionCoordinator,
    plan: FreshSessionPlan,
) -> ReconnectAttemptResult {
    let result = coordinator.reconnect(plan.connection_id);
    if !matches!(result, ReconnectResult::Pending(_)) {
        *reconnect_in_progress = false;
    }
    result
}
