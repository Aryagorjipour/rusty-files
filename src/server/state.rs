use crate::SearchEngine;
use crate::server::config::ServerConfig;
use crate::server::models::FileChangeEvent;
use dashmap::DashMap;
use parking_lot::RwLock;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use chrono::{DateTime, Utc};
use tokio::sync::broadcast;

pub struct AppState {
    pub engine: Arc<RwLock<SearchEngine>>,
    pub config: Arc<ServerConfig>,
    pub metrics: Arc<Metrics>,
    pub watchers: Arc<DashMap<String, WatchHandle>>,
    pub event_tx: broadcast::Sender<FileChangeEvent>,
    pub start_time: Instant,
}

impl AppState {
    pub fn new(engine: SearchEngine, config: ServerConfig) -> Self {
        let (event_tx, _) = broadcast::channel(1000);

        Self {
            engine: Arc::new(RwLock::new(engine)),
            config: Arc::new(config),
            metrics: Arc::new(Metrics::new()),
            watchers: Arc::new(DashMap::new()),
            event_tx,
            start_time: Instant::now(),
        }
    }

    pub fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }
}

pub struct Metrics {
    pub total_searches: AtomicU64,
    pub total_search_time_ms: AtomicU64,
    pub cache_hits: AtomicU64,
    pub cache_misses: AtomicU64,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            total_searches: AtomicU64::new(0),
            total_search_time_ms: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
        }
    }

    pub fn record_search(&self, duration_ms: u64) {
        self.total_searches.fetch_add(1, Ordering::Relaxed);
        self.total_search_time_ms
            .fetch_add(duration_ms, Ordering::Relaxed);
    }

    pub fn avg_search_time_ms(&self) -> f64 {
        let total = self.total_searches.load(Ordering::Relaxed);
        if total == 0 {
            return 0.0;
        }
        self.total_search_time_ms.load(Ordering::Relaxed) as f64 / total as f64
    }

    pub fn cache_hit_rate(&self) -> f32 {
        let hits = self.cache_hits.load(Ordering::Relaxed);
        let misses = self.cache_misses.load(Ordering::Relaxed);
        let total = hits + misses;

        if total == 0 {
            return 0.0;
        }

        hits as f32 / total as f32
    }
}

pub struct WatchHandle {
    pub path: PathBuf,
    pub recursive: bool,
    pub created_at: DateTime<Utc>,
}
