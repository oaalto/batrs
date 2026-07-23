use super::super::ConnectionCoordinator;
use super::SessionLifecycle;
use super::fresh_session::FreshSessionPlan;
use super::reconnect::execute_reconnect;

pub fn prepare_connect(
    lifecycle: &mut SessionLifecycle,
    pre_connect_login_name: Option<String>,
) -> Result<FreshSessionPlan, ()> {
    if *lifecycle.reconnect_in_progress_mut() {
        return Err(());
    }

    *lifecycle.reconnect_in_progress_mut() = true;
    Ok(lifecycle.begin_fresh_session(pre_connect_login_name))
}

pub fn complete_connect(
    lifecycle: &mut SessionLifecycle,
    coordinator: &mut dyn ConnectionCoordinator,
    plan: FreshSessionPlan,
) -> super::reconnect::ReconnectAttemptResult {
    execute_reconnect(lifecycle.reconnect_in_progress_mut(), coordinator, plan)
}
