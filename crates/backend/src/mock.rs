use flume::{Receiver, Sender, bounded};

use crate::*;

#[allow(dead_code)]
pub struct MockBackend {
    tx: Sender<BackendEvent>,
    rx: Receiver<BackendCall>,
    latest: u64,
}

impl MockBackend {
    pub fn new(tx: Sender<BackendEvent>, rx: Receiver<BackendCall>) -> Self {
        Self { tx, rx, latest: 0 }
    }

    pub async fn run(&self) {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await
        }
    }
}

pub fn run() -> Result<Backend, BackendError> {
    let (call_tx, call_rx) = bounded(BACKEND_CALL_QUEUE_SIZE);
    let (event_tx, event_rx) = bounded(BACKEND_EVENT_QUEUE_SIZE);
    let async_backend = MockBackend::new(event_tx, call_rx);
    tokio::spawn(async move { async_backend.run().await });
    Ok(Backend::new(event_rx, call_tx))
}
