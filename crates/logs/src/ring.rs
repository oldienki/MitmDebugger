use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tracing::Level;

/// One captured log record.
#[derive(Debug, Clone)]
pub struct LogRecord {
    /// UTC timestamp, `YYYY-MM-DD HH:MM:SS.mmm`.
    pub ts: String,
    /// Severity level.
    pub level: Level,
    /// Module/target that emitted the record.
    pub target: String,
    /// Formatted message plus fields.
    pub message: String,
}

/// Bounded, thread-safe ring of recent log records.
/// Evicts the oldest record once `capacity` is reached.
#[derive(Debug)]
pub struct RingBuffer {
    inner: Mutex<VecDeque<LogRecord>>,
    capacity: usize,
}

impl RingBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: Mutex::new(VecDeque::with_capacity(capacity)),
            capacity,
        }
    }

    pub fn push(&self, record: LogRecord) {
        let mut inner = self.inner.lock().expect("ring buffer poisoned");
        if inner.len() == self.capacity {
            inner.pop_front();
        }
        inner.push_back(record);
    }

    /// Newest-last snapshot of all retained records.
    pub fn snapshot(&self) -> Vec<LogRecord> {
        self.inner
            .lock()
            .expect("ring buffer poisoned")
            .iter()
            .cloned()
            .collect()
    }

    pub fn len(&self) -> usize {
        self.inner.lock().expect("ring buffer poisoned").len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

/// Shareable read handle to the ring buffer.
#[derive(Debug, Clone)]
pub struct RingHandle(Arc<RingBuffer>);

impl RingHandle {
    pub(crate) fn new(buffer: Arc<RingBuffer>) -> Self {
        Self(buffer)
    }

    /// Newest-last snapshot of all retained records.
    pub fn snapshot(&self) -> Vec<LogRecord> {
        self.0.snapshot()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }
}

/// Format `millis` since the Unix epoch as `YYYY-MM-DD HH:MM:SS.mmm` (UTC).
pub fn format_ts(millis: u64) -> String {
    let secs = millis / 1000;
    let ms = millis % 1000;
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let (y, mo, d) = civil_from_days(days);
    format!(
        "{y:04}-{mo:02}-{d:02} {:02}:{:02}:{:02}.{ms:03}",
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}

/// Howard Hinnant's civil-from-days algorithm; `z` is days since 1970-01-01.
fn civil_from_days(z: i64) -> (i64, u64, u64) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rec(msg: &str) -> LogRecord {
        LogRecord {
            ts: "2026-01-01 00:00:00.000".into(),
            level: Level::INFO,
            target: "test".into(),
            message: msg.into(),
        }
    }

    #[test]
    fn push_and_snapshot_keeps_order() {
        let b = RingBuffer::new(4);
        b.push(rec("a"));
        b.push(rec("b"));
        let s = b.snapshot();
        assert_eq!(s.len(), 2);
        assert_eq!(s[0].message, "a");
        assert_eq!(s[1].message, "b");
    }

    #[test]
    fn eviction_at_capacity_drops_oldest() {
        let b = RingBuffer::new(2);
        b.push(rec("a"));
        b.push(rec("b"));
        b.push(rec("c"));
        let s = b.snapshot();
        assert_eq!(s.len(), 2);
        assert_eq!(s[0].message, "b");
        assert_eq!(s[1].message, "c");
    }

    #[test]
    fn unicode_message_roundtrip() {
        let b = RingBuffer::new(4);
        b.push(rec("привет мир 🌍"));
        assert_eq!(b.snapshot()[0].message, "привет мир 🌍");
    }

    #[test]
    fn format_ts_known_epoch() {
        // 2026-01-01T00:00:00Z == 1767225600s
        assert_eq!(format_ts(1_767_225_600_000), "2026-01-01 00:00:00.000");
        assert_eq!(format_ts(0), "1970-01-01 00:00:00.000");
    }
}
