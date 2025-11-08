use crate::core::config::SearchConfig;
use crate::core::error::Result;
use crate::filters::ExclusionFilter;
use crate::indexer::incremental::IncrementalIndexer;
use crate::storage::Database;
use crate::watcher::debouncer::FileEventType;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;

#[derive(Clone, Debug)]
pub struct FileEvent {
    pub path: PathBuf,
    pub event_type: FileEventType,
}

pub struct IndexSynchronizer {
    indexer: Arc<IncrementalIndexer>,
    event_receiver: Option<mpsc::UnboundedReceiver<FileEvent>>,
    event_sender: mpsc::UnboundedSender<FileEvent>,
}

impl IndexSynchronizer {
    pub fn new(
        database: Arc<Database>,
        config: Arc<SearchConfig>,
        exclusion_filter: Arc<ExclusionFilter>,
    ) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();

        let indexer = Arc::new(IncrementalIndexer::new(database, config, exclusion_filter));

        Self {
            indexer,
            event_receiver: Some(receiver),
            event_sender: sender,
        }
    }

    pub fn get_sender(&self) -> mpsc::UnboundedSender<FileEvent> {
        self.event_sender.clone()
    }

    pub async fn start(&mut self) -> Result<()> {
        let mut receiver = self.event_receiver.take().ok_or_else(|| {
            crate::core::error::SearchError::NotInitialized(
                "Synchronizer already started".to_string(),
            )
        })?;

        while let Some(event) = receiver.recv().await {
            if let Err(e) = self.handle_event(event).await {
                log::error!("Failed to handle file event: {}", e);
            }
        }

        Ok(())
    }

    async fn handle_event(&self, event: FileEvent) -> Result<()> {
        match event.event_type {
            FileEventType::Created | FileEventType::Modified => {
                self.indexer.update_file(&event.path)?;
            }
            FileEventType::Deleted => {
                self.indexer
                    .update_file(&event.path)?;
            }
            FileEventType::Renamed => {
                self.indexer.update_file(&event.path)?;
            }
        }

        Ok(())
    }

    pub fn sync_path(&self, path: PathBuf) -> Result<()> {
        self.indexer.update_file(path)?;
        Ok(())
    }

    pub fn sync_paths(&self, paths: Vec<PathBuf>) -> Result<usize> {
        self.indexer.update_files(&paths)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::SearchConfig;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_synchronizer() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        fs::write(&file_path, "content").unwrap();

        let db = Arc::new(Database::in_memory(10).unwrap());
        let config = Arc::new(SearchConfig::default());
        let filter = Arc::new(ExclusionFilter::default());

        let synchronizer = IndexSynchronizer::new(db, config, filter);

        let result = synchronizer.sync_path(file_path);
        assert!(result.is_ok());
    }
}
