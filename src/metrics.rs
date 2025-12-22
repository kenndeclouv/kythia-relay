use serde::Serialize;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

/// Metrics collector for monitoring server health and performance
#[derive(Clone)]
pub struct MetricsCollector {
    inner: Arc<MetricsInner>,
}

struct MetricsInner {
    /// Total number of connections established
    total_connections: AtomicU64,

    /// Currently active connections
    active_connections: AtomicUsize,

    /// Total number of rooms created
    total_rooms_created: AtomicU64,

    /// Currently active rooms
    active_rooms: AtomicUsize,

    /// Total messages sent
    messages_sent: AtomicU64,

    /// Total messages received
    messages_received: AtomicU64,

    /// Messages dropped (due to full channel or rate limit)
    messages_dropped: AtomicU64,

    /// Total bytes sent
    bytes_sent: AtomicU64,

    /// Total bytes received
    bytes_received: AtomicU64,
}

/// Snapshot of current metrics
#[derive(Serialize, Clone, Debug)]
pub struct MetricsSnapshot {
    pub total_connections: u64,
    pub active_connections: usize,
    pub total_rooms_created: u64,
    pub active_rooms: usize,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub messages_dropped: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        MetricsCollector {
            inner: Arc::new(MetricsInner {
                total_connections: AtomicU64::new(0),
                active_connections: AtomicUsize::new(0),
                total_rooms_created: AtomicU64::new(0),
                active_rooms: AtomicUsize::new(0),
                messages_sent: AtomicU64::new(0),
                messages_received: AtomicU64::new(0),
                messages_dropped: AtomicU64::new(0),
                bytes_sent: AtomicU64::new(0),
                bytes_received: AtomicU64::new(0),
            }),
        }
    }

    /// Increment total connections
    pub fn connection_established(&self) {
        self.inner.total_connections.fetch_add(1, Ordering::Relaxed);
        self.inner
            .active_connections
            .fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement active connections
    pub fn connection_closed(&self) {
        self.inner
            .active_connections
            .fetch_sub(1, Ordering::Relaxed);
    }

    /// Increment room counters
    pub fn room_created(&self) {
        self.inner
            .total_rooms_created
            .fetch_add(1, Ordering::Relaxed);
        self.inner.active_rooms.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement active rooms
    pub fn room_deleted(&self) {
        self.inner.active_rooms.fetch_sub(1, Ordering::Relaxed);
    }

    /// Increment message sent counter
    pub fn message_sent(&self, bytes: usize) {
        self.inner.messages_sent.fetch_add(1, Ordering::Relaxed);
        self.inner
            .bytes_sent
            .fetch_add(bytes as u64, Ordering::Relaxed);
    }

    /// Increment message received counter
    pub fn message_received(&self, bytes: usize) {
        self.inner.messages_received.fetch_add(1, Ordering::Relaxed);
        self.inner
            .bytes_received
            .fetch_add(bytes as u64, Ordering::Relaxed);
    }

    /// Increment dropped message counter
    pub fn message_dropped(&self) {
        self.inner.messages_dropped.fetch_add(1, Ordering::Relaxed);
    }

    /// Get a snapshot of current metrics
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            total_connections: self.inner.total_connections.load(Ordering::Relaxed),
            active_connections: self.inner.active_connections.load(Ordering::Relaxed),
            total_rooms_created: self.inner.total_rooms_created.load(Ordering::Relaxed),
            active_rooms: self.inner.active_rooms.load(Ordering::Relaxed),
            messages_sent: self.inner.messages_sent.load(Ordering::Relaxed),
            messages_received: self.inner.messages_received.load(Ordering::Relaxed),
            messages_dropped: self.inner.messages_dropped.load(Ordering::Relaxed),
            bytes_sent: self.inner.bytes_sent.load(Ordering::Relaxed),
            bytes_received: self.inner.bytes_received.load(Ordering::Relaxed),
        }
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}
