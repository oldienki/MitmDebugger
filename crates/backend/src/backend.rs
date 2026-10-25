use crate::flows::*;
use flume::{Receiver, Sender, TryRecvError, TrySendError};

pub const BACKEND_EVENT_QUEUE_SIZE: usize = 30;
pub const BACKEND_CALL_QUEUE_SIZE: usize = 10;

type CallKey = Option<u64>;

pub enum BackendEvent {
    /// Response to a `List` call; `CallKey` matches the request and flows
    /// carry the requested content level.
    Flows(CallKey, Result<Vec<Flow>, FlowsError>),
    /// Unsolicited update of the latest flows; flows carry the minimal
    /// content level.
    LatestFlows(Vec<Flow>),
    Disconnected,
    Finished,
    Failed(BackendError),
}

pub enum BackendCall {
    List(CallKey, FlowsRange, ContentLevel),
}

#[derive(Debug, thiserror::Error)]
pub enum BackendError {
    #[error("Backend disconnected")]
    Disconnected,
    #[error("Backend call queue is full")]
    CallQueueOverflow,
    #[error("Call queue error: {0}")]
    Queue(String),
    #[error("{0}")]
    General(String),
}

pub struct Backend {
    rx: Receiver<BackendEvent>,
    tx: Sender<BackendCall>,
}

impl Backend {
    pub fn new(rx: Receiver<BackendEvent>, tx: Sender<BackendCall>) -> Self {
        Self { rx, tx }
    }

    pub fn list(
        &self,
        range: FlowsRange,
        content_level: ContentLevel,
        key: CallKey,
    ) -> Result<(), BackendError> {
        match self
            .tx
            .try_send(BackendCall::List(key, range, content_level))
        {
            Ok(()) => Ok(()),
            Err(TrySendError::Full(_)) => Err(BackendError::CallQueueOverflow),
            Err(TrySendError::Disconnected(_)) => Err(BackendError::Disconnected),
        }
    }

    pub fn poll(&self) -> Option<BackendEvent> {
        match self.rx.try_recv() {
            Ok(msg) => Some(msg),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => Some(BackendEvent::Disconnected),
        }
    }
}
