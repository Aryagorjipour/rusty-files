use crate::core::config::SearchConfig;
use crate::core::error::Result;
use crate::filters::ExclusionFilter;
use crate::storage::Database;
use crate::watcher::debouncer::{EventDebouncer, FileEventType};
use crate::watcher::synchronizer::{FileEvent, IndexSynchronizer};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct FileSystemMonitor {
    exclusion_filter: Arc<ExclusionFilter>,
    synchronizer: Arc<IndexSynchronizer>,
    debouncer: Arc<EventDebouncer>,
    is_running: Arc<AtomicBool>,
    watcher: Option<RecommendedWatcher>,
}

impl FileSystemMonitor {
    pub fn new(
        database: Arc<Database>,
        config: Arc<SearchConfig>,
        exclusion_filter: Arc<ExclusionFilter>,
    ) -> Self {
        let synchronizer = Arc::new(IndexSynchronizer::new(
            database,
            Arc::clone(&config),
            Arc::clone(&exclusion_filter),
        ));

        let debouncer = Arc::new(EventDebouncer::new(config.watch_debounce_ms));

        Self {
            exclusion_filter,
            synchronizer,
            debouncer,
            is_running: Arc::new(AtomicBool::new(false)),
            watcher: None,
        }
    }

    pub fn start<P: AsRef<Path>>(&mut self, root: P) -> Result<()> {
        if self.is_running.load(Ordering::Relaxed) {
            return Ok(());
        }

        let sender = self.synchronizer.get_sender();
        let debouncer = Arc::clone(&self.debouncer);
        let exclusion_filter = Arc::clone(&self.exclusion_filter);

        let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            if let Ok(event) = res {
                Self::handle_notify_event(event, &sender, &debouncer, &exclusion_filter);
            }
        })?;

        watcher.watch(root.as_ref(), RecursiveMode::Recursive)?;

        self.watcher = Some(watcher);
        self.is_running.store(true, Ordering::Relaxed);

        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        if !self.is_running.load(Ordering::Relaxed) {
            return Ok(());
        }

        self.watcher = None;
        self.is_running.store(false, Ordering::Relaxed);

        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::Relaxed)
    }

    fn handle_notify_event(
        event: Event,
        sender: &mpsc::UnboundedSender<FileEvent>,
        debouncer: &Arc<EventDebouncer>,
        exclusion_filter: &Arc<ExclusionFilter>,
    ) {
        let event_type = match event.kind {
            EventKind::Create(_) => FileEventType::Created,
            EventKind::Modify(_) => FileEventType::Modified,
            EventKind::Remove(_) => FileEventType::Deleted,
            EventKind::Any => FileEventType::Modified,
            _ => return,
        };

        for path in event.paths {
            if exclusion_filter.is_excluded(&path) {
                continue;
            }

            if !debouncer.should_process(path.clone(), event_type) {
                continue;
            }

            let file_event = FileEvent { path, event_type };

            if sender.send(file_event).is_err() {
                log::error!("Failed to send file event to synchronizer");
            }
        }
    }

    pub async fn run_cleanup_task(&self) {
        use tokio::time::{interval, Duration};

        let mut interval = interval(Duration::from_secs(60));
        let debouncer = Arc::clone(&self.debouncer);

        loop {
            interval.tick().await;
            debouncer.cleanup_old_events(Duration::from_secs(300));
        }
    }
}

impl Drop for FileSystemMonitor {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_monitor_creation() {
        let db = Arc::new(Database::in_memory(10).unwrap());
        let config = Arc::new(SearchConfig::default());
        let filter = Arc::new(ExclusionFilter::default());

        let monitor = FileSystemMonitor::new(db, config, filter);
        assert!(!monitor.is_running());
    }

    #[test]
    fn test_monitor_start_stop() {
        let temp_dir = TempDir::new().unwrap();

        let db = Arc::new(Database::in_memory(10).unwrap());
        let config = Arc::new(SearchConfig::default());
        let filter = Arc::new(ExclusionFilter::default());

        let mut monitor = FileSystemMonitor::new(db, config, filter);

        assert!(monitor.start(temp_dir.path()).is_ok());
        assert!(monitor.is_running());

        assert!(monitor.stop().is_ok());
        assert!(!monitor.is_running());
    }
}
