use super::{AppEvent, ConnectionChannels, ConnectionCoordinator, ConnectionId, ReconnectResult};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::mpsc;

pub type SharedReconnectCalls = Rc<RefCell<Vec<ConnectionId>>>;
pub type SharedReconnectResults = Rc<RefCell<VecDeque<ReconnectResult>>>;
pub type FakeConnectionCoordinatorParts = (
    FakeConnectionCoordinator,
    SharedReconnectCalls,
    SharedReconnectResults,
);

pub struct FakeConnectionCoordinator {
    calls: SharedReconnectCalls,
    results: SharedReconnectResults,
}

impl Default for FakeConnectionCoordinator {
    fn default() -> Self {
        Self::new(Vec::new()).0
    }
}

impl FakeConnectionCoordinator {
    pub fn new(results: Vec<ReconnectResult>) -> FakeConnectionCoordinatorParts {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let results = Rc::new(RefCell::new(results.into()));
        (
            Self {
                calls: Rc::clone(&calls),
                results: Rc::clone(&results),
            },
            calls,
            results,
        )
    }
}

impl ConnectionCoordinator for FakeConnectionCoordinator {
    fn reconnect(&mut self, connection_id: ConnectionId) -> ReconnectResult {
        self.calls.borrow_mut().push(connection_id);
        self.results
            .borrow_mut()
            .pop_front()
            .unwrap_or_else(|| ReconnectResult::Failed("no fake reconnect result".to_string()))
    }
}

pub fn connection_channels() -> (
    ConnectionChannels,
    mpsc::Receiver<String>,
    mpsc::Sender<AppEvent>,
) {
    let (event_sender, event_receiver) = mpsc::channel();
    let (command_sender, command_receiver) = mpsc::channel();
    (
        ConnectionChannels {
            event_receiver,
            command_sender,
        },
        command_receiver,
        event_sender,
    )
}
