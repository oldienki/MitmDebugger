use std::fmt::Write as _;
use std::fs::File;
use std::io::Write as _;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::field::{Field, Visit};
use tracing::{Event, Subscriber};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;

use crate::ring::{LogRecord, RingBuffer, format_ts};

/// Captures `message` plus `key=value` fields from a tracing event.
#[derive(Default)]
struct RecordVisitor {
    message: String,
    fields: String,
}

impl Visit for RecordVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            let _ = write!(self.message, "{value:?}");
        } else {
            if !self.fields.is_empty() {
                self.fields.push(' ');
            }
            let _ = write!(self.fields, "{}={:?}", field.name(), value);
        }
    }
}

/// `tracing` layer that appends every record to a log file and a ring buffer.
pub struct SinkLayer {
    file: Mutex<File>,
    ring: Arc<RingBuffer>,
}

impl SinkLayer {
    pub fn new(file: File, ring: Arc<RingBuffer>) -> Self {
        Self {
            file: Mutex::new(file),
            ring,
        }
    }
}

impl<S: Subscriber> Layer<S> for SinkLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut visitor = RecordVisitor::default();
        event.record(&mut visitor);
        let message = if visitor.fields.is_empty() {
            visitor.message
        } else {
            format!("{} {}", visitor.message, visitor.fields)
        };
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        let record = LogRecord {
            ts: format_ts(millis),
            level: *event.metadata().level(),
            target: event.metadata().target().to_string(),
            message,
        };
        let line = format!(
            "{} [{}] {}: {}",
            record.ts, record.level, record.target, record.message
        );
        if let Ok(mut f) = self.file.lock() {
            let _ = writeln!(f, "{line}");
        }
        self.ring.push(record);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_subscriber::Registry;
    use tracing_subscriber::layer::SubscriberExt;

    #[test]
    fn layer_writes_file_and_ring() {
        let dir = std::env::temp_dir().join(format!("logs-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("t.log");
        let file = File::create(&path).unwrap();
        let ring = Arc::new(RingBuffer::new(8));
        let layer = SinkLayer::new(file, ring.clone());
        let subscriber = Registry::default().with(layer);
        tracing::subscriber::with_default(subscriber, || {
            tracing::info!(answer = 42, "hello {}", "world");
        });
        let snap = ring.snapshot();
        assert_eq!(snap.len(), 1);
        assert!(snap[0].message.contains("hello world"));
        assert!(snap[0].message.contains("answer=42"));
        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(contents.contains("hello world"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
