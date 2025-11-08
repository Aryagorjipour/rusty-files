use crate::core::config::SearchConfig;
use crate::core::error::Result;
use crate::core::types::{FileEntry, Progress, ProgressCallback};
use crate::filters::ExclusionFilter;
use crate::indexer::content::ContentAnalyzer;
use crate::indexer::metadata::MetadataExtractor;
use crate::indexer::walker::DirectoryWalker;
use crate::storage::Database;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

pub struct IndexBuilder {
    database: Arc<Database>,
    config: Arc<SearchConfig>,
    exclusion_filter: Arc<ExclusionFilter>,
    content_analyzer: Arc<ContentAnalyzer>,
    cancelled: Arc<AtomicBool>,
}

impl IndexBuilder {
    pub fn new(
        database: Arc<Database>,
        config: Arc<SearchConfig>,
        exclusion_filter: Arc<ExclusionFilter>,
    ) -> Self {
        let content_analyzer = Arc::new(ContentAnalyzer::new(config.max_file_size_for_content));

        Self {
            database,
            config,
            exclusion_filter,
            content_analyzer,
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn build<P: AsRef<Path>>(
        &self,
        root: P,
        progress_callback: Option<ProgressCallback>,
    ) -> Result<usize> {
        let walker = DirectoryWalker::new(
            Arc::clone(&self.config),
            Arc::clone(&self.exclusion_filter),
        );

        let paths = walker.walk_parallel(root)?;
        let total_paths = paths.len();

        if let Some(ref callback) = progress_callback {
            callback(Progress::new(
                0,
                total_paths,
                "Starting indexing...".to_string(),
            ));
        }

        let processed = Arc::new(AtomicUsize::new(0));
        let batch_size = self.config.batch_size;
        let mut indexed_count = 0;

        for chunk in paths.chunks(batch_size) {
            if self.cancelled.load(Ordering::Relaxed) {
                break;
            }

            let entries = self.process_batch(chunk)?;
            self.database.insert_files_batch(&entries)?;

            if self.config.enable_content_search {
                self.index_content_batch(&entries)?;
            }

            indexed_count += entries.len();
            processed.fetch_add(entries.len(), Ordering::Relaxed);

            if let Some(ref callback) = progress_callback {
                callback(Progress::new(
                    processed.load(Ordering::Relaxed),
                    total_paths,
                    format!("Indexed {} files", processed.load(Ordering::Relaxed)),
                ));
            }
        }

        Ok(indexed_count)
    }

    fn process_batch(&self, paths: &[impl AsRef<Path> + Sync]) -> Result<Vec<FileEntry>> {
        let results = MetadataExtractor::extract_batch(paths);

        let entries: Vec<FileEntry> = results
            .into_iter()
            .filter_map(|result| match result {
                Ok(entry) => Some(entry),
                Err(e) => {
                    log::warn!("Failed to extract metadata: {}", e);
                    None
                }
            })
            .collect();

        Ok(entries)
    }

    fn index_content_batch(&self, entries: &[FileEntry]) -> Result<()> {
        let text_files: Vec<_> = entries
            .iter()
            .filter(|e| !e.is_directory)
            .collect();

        if text_files.is_empty() {
            return Ok(());
        }

        let paths: Vec<_> = text_files.iter().map(|e| &e.path).collect();
        let results = self.content_analyzer.analyze_batch(&paths);

        for (idx, result) in results {
            if let Ok(Some(preview)) = result {
                if let Some(file_id) = text_files[idx].id {
                    if let Err(e) = self.database.insert_content(file_id, &preview) {
                        log::warn!("Failed to insert content: {}", e);
                    }

                    if let Err(e) = self.database.insert_fts_entry(
                        file_id,
                        &text_files[idx].name,
                        &text_files[idx].path.to_string_lossy(),
                        &preview.preview,
                    ) {
                        log::warn!("Failed to insert FTS entry: {}", e);
                    }
                }
            }
        }

        Ok(())
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }

    pub fn reset_cancellation(&self) {
        self.cancelled.store(false, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::SearchConfig;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_index_builder() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        fs::write(root.join("file1.txt"), "content1").unwrap();
        fs::write(root.join("file2.txt"), "content2").unwrap();
        fs::create_dir(root.join("subdir")).unwrap();
        fs::write(root.join("subdir/file3.txt"), "content3").unwrap();

        let db = Arc::new(Database::in_memory(10).unwrap());
        // Enable hidden files indexing since temp dirs often start with a dot
        let mut config = SearchConfig::default();
        config.index_hidden_files = true;
        let config = Arc::new(config);
        // Use empty exclusion filter to avoid any pattern matching issues
        let filter = Arc::new(ExclusionFilter::from_patterns(&[]).unwrap());

        let builder = IndexBuilder::new(db.clone(), config, filter);
        let count = builder.build(root, None).unwrap();

        assert!(count > 0);
        assert_eq!(count, 3, "Expected 3 files to be indexed");
    }

    #[test]
    fn test_cancellation() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        for i in 0..100 {
            fs::write(root.join(format!("file{}.txt", i)), "content").unwrap();
        }

        let db = Arc::new(Database::in_memory(10).unwrap());
        let config = Arc::new(SearchConfig::default());
        let filter = Arc::new(ExclusionFilter::default());

        let builder = IndexBuilder::new(db, config, filter);
        builder.cancel();

        let count = builder.build(root, None).unwrap();
        assert_eq!(count, 0);
    }
}
