//! Application logging: every `tracing` record is written to a log file
//! and appended to a fixed-size in-memory ring buffer.
//!
//! [`init`] installs the global subscriber and returns a [`RingHandle`]
//! for reading recent records (for example, a `Ctrl+L` log viewer).

mod layer;
mod ring;

pub use ring::{LogRecord, RingBuffer, RingHandle, format_ts};

use std::fs::OpenOptions;
use std::path::Path;
use std::sync::Arc;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// Install the global `tracing` subscriber.
///
/// Records flow through `RUST_LOG` (`env-filter`) to `log_file` (appended)
/// and to a bounded ring buffer of `capacity` records.
///
/// Returns [`Err`] when the file cannot be opened or a global subscriber
/// is already installed.
pub fn init(log_file: &Path, capacity: usize) -> anyhow::Result<RingHandle> {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file)?;
    let ring = Arc::new(RingBuffer::new(capacity));
    let handle = RingHandle::new(ring.clone());
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    tracing_subscriber::registry()
        .with(filter)
        .with(layer::SinkLayer::new(file, ring))
        .try_init()?;
    Ok(handle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_fails_on_unwritable_path() {
        let result = init(Path::new("/proc/nonexistent-dir-xyz/log.txt"), 8);
        assert!(result.is_err());
    }
}
