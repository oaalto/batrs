use super::super::ConnectionId;

pub(crate) fn is_stale(
    active_connection_id: ConnectionId,
    event_connection_id: ConnectionId,
) -> bool {
    event_connection_id != active_connection_id
}
