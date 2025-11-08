use dashmap::DashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct EventDebouncer {
    events: Arc<DashMap<PathBuf, DebouncedEvent>>,
    debounce_duration: Duration,
}

#[derive(Clone)]
struct DebouncedEvent {
    last_event_time: Instant,
    event_type: FileEventType,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileEventType {
    Created,
    Modified,
    Deleted,
    Renamed,
}

impl EventDebouncer {
    pub fn new(debounce_ms: u64) -> Self {
        Self {
            events: Arc::new(DashMap::new()),
            debounce_duration: Duration::from_millis(debounce_ms),
        }
    }

    pub fn should_process(&self, path: PathBuf, event_type: FileEventType) -> bool {
        let now = Instant::now();

        if let Some(mut entry) = self.events.get_mut(&path) {
            let elapsed = now.duration_since(entry.last_event_time);

            if elapsed < self.debounce_duration {
                entry.last_event_time = now;
                entry.event_type = event_type;
                return false;
            }

            entry.last_event_time = now;
            entry.event_type = event_type;
            true
        } else {
            self.events.insert(
                path.clone(),
                DebouncedEvent {
                    last_event_time: now,
                    event_type,
                },
            );
            true
        }
    }

    pub fn cleanup_old_events(&self, max_age: Duration) {
        let now = Instant::now();
        self.events.retain(|_, event| {
            now.duration_since(event.last_event_time) < max_age
        });
    }

    pub fn clear(&self) {
        self.events.clear();
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

impl Default for EventDebouncer {
    fn default() -> Self {
        Self::new(500)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_debouncer_basic() {
        let debouncer = EventDebouncer::new(100);
        let path = PathBuf::from("/test/file.txt");

        assert!(debouncer.should_process(path.clone(), FileEventType::Modified));

        assert!(!debouncer.should_process(path.clone(), FileEventType::Modified));
    }

    #[test]
    fn test_debouncer_after_delay() {
        let debouncer = EventDebouncer::new(50);
        let path = PathBuf::from("/test/file.txt");

        assert!(debouncer.should_process(path.clone(), FileEventType::Modified));

        thread::sleep(Duration::from_millis(100));

        assert!(debouncer.should_process(path.clone(), FileEventType::Modified));
    }

    #[test]
    fn test_cleanup_old_events() {
        let debouncer = EventDebouncer::new(100);
        let path = PathBuf::from("/test/file.txt");

        debouncer.should_process(path.clone(), FileEventType::Modified);
        assert_eq!(debouncer.len(), 1);

        thread::sleep(Duration::from_millis(200));
        debouncer.cleanup_old_events(Duration::from_millis(100));

        assert_eq!(debouncer.len(), 0);
    }
}
