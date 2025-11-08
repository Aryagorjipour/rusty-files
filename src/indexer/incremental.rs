use crate::core::config::SearchConfig;
use crate::core::error::Result;
use crate::core::types::ProgressCallback;
use crate::filters::ExclusionFilter;
use crate::indexer::builder::IndexBuilder;
use crate::indexer::metadata::MetadataExtractor;
use crate::storage::Database;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct IncrementalIndexer {
    database: Arc<Database>,
    config: Arc<SearchConfig>,
    _builder: Arc<IndexBuilder>,
}

impl IncrementalIndexer {
    pub fn new(
        database: Arc<Database>,
        config: Arc<SearchConfig>,
        exclusion_filter: Arc<ExclusionFilter>,
    ) -> Self {
        let builder = Arc::new(IndexBuilder::new(
            Arc::clone(&database),
            Arc::clone(&config),
            exclusion_filter,
        ));

        Self {
            database,
            config,
            _builder: builder,
        }
    }

    pub fn update<P: AsRef<Path>>(
        &self,
        root: P,
        progress_callback: Option<ProgressCallback>,
    ) -> Result<UpdateStats> {
        let root = root.as_ref();

        let existing_files = self.get_indexed_files(root)?;
        let current_files = self.scan_current_files(root)?;

        let mut stats = UpdateStats::default();

        for path in &current_files {
            if !existing_files.contains(path) {
                if let Ok(entry) = MetadataExtractor::extract(path) {
                    self.database.insert_file(&entry)?;
                    stats.added += 1;
                }
            } else if self.needs_update(path)? {
                if let Ok(entry) = MetadataExtractor::extract(path) {
                    self.database.insert_file(&entry)?;
                    stats.updated += 1;
                }
            }
        }

        for path in &existing_files {
            if !current_files.contains(path) {
                self.database.delete_by_path(path)?;
                stats.removed += 1;
            }
        }

        if let Some(callback) = progress_callback {
            callback(crate::core::types::Progress::new(
                stats.total(),
                stats.total(),
                format!("Update complete: {} changes", stats.total()),
            ));
        }

        Ok(stats)
    }

    pub fn update_file<P: AsRef<Path>>(&self, path: P) -> Result<bool> {
        let path = path.as_ref();

        if !path.exists() {
            self.database.delete_by_path(path)?;
            return Ok(true);
        }

        let entry = MetadataExtractor::extract(path)?;
        self.database.insert_file(&entry)?;

        Ok(true)
    }

    pub fn update_files(&self, paths: &[PathBuf]) -> Result<usize> {
        let mut updated = 0;

        for path in paths {
            if self.update_file(path).is_ok() {
                updated += 1;
            }
        }

        Ok(updated)
    }

    fn get_indexed_files<P: AsRef<Path>>(&self, root: P) -> Result<HashSet<PathBuf>> {
        let root = root.as_ref();
        let mut files = HashSet::new();
        let mut offset = 0;
        let limit = 1000;

        loop {
            let batch = self.database.get_all_files(limit, offset)?;
            if batch.is_empty() {
                break;
            }

            for entry in batch {
                if entry.path.starts_with(root) {
                    files.insert(entry.path);
                }
            }

            offset += limit;
        }

        Ok(files)
    }

    fn scan_current_files<P: AsRef<Path>>(&self, root: P) -> Result<HashSet<PathBuf>> {
        use crate::indexer::walker::DirectoryWalker;

        let walker = DirectoryWalker::new(
            Arc::clone(&self.config),
            Arc::new(ExclusionFilter::default()),
        );

        let paths = walker.walk_parallel(root)?;
        Ok(paths.into_iter().collect())
    }

    fn needs_update<P: AsRef<Path>>(&self, path: P) -> Result<bool> {
        let path = path.as_ref();

        if let Some(existing) = self.database.find_by_path(path)? {
            if let Some(last_modified) = existing.modified_at {
                return MetadataExtractor::is_modified_since(path, last_modified);
            }
        }

        Ok(true)
    }

    pub fn verify_index<P: AsRef<Path>>(&self, root: P) -> Result<VerificationStats> {
        let root = root.as_ref();
        let indexed_files = self.get_indexed_files(root)?;

        let mut stats = VerificationStats::default();
        stats.total_indexed = indexed_files.len();

        for path in indexed_files {
            if !path.exists() {
                stats.missing += 1;
            } else if self.needs_update(&path)? {
                stats.outdated += 1;
            } else {
                stats.valid += 1;
            }
        }

        Ok(stats)
    }
}

#[derive(Debug, Default, Clone)]
pub struct UpdateStats {
    pub added: usize,
    pub updated: usize,
    pub removed: usize,
}

impl UpdateStats {
    pub fn total(&self) -> usize {
        self.added + self.updated + self.removed
    }
}

#[derive(Debug, Default, Clone)]
pub struct VerificationStats {
    pub total_indexed: usize,
    pub valid: usize,
    pub outdated: usize,
    pub missing: usize,
}

impl VerificationStats {
    pub fn health_percentage(&self) -> f64 {
        if self.total_indexed == 0 {
            return 100.0;
        }
        (self.valid as f64 / self.total_indexed as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_incremental_update() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        fs::write(root.join("file1.txt"), "content1").unwrap();

        let db = Arc::new(Database::in_memory(10).unwrap());
        let config = Arc::new(SearchConfig::default());
        let filter = Arc::new(ExclusionFilter::default());

        let indexer = IncrementalIndexer::new(db.clone(), config, filter);

        let stats = indexer.update(root, None).unwrap();
        assert!(stats.added > 0);

        fs::write(root.join("file2.txt"), "content2").unwrap();

        let stats = indexer.update(root, None).unwrap();
        assert!(stats.added > 0);
    }

    #[test]
    fn test_file_removal_detection() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let file_path = root.join("file.txt");

        fs::write(&file_path, "content").unwrap();

        let db = Arc::new(Database::in_memory(10).unwrap());
        let config = Arc::new(SearchConfig::default());
        let filter = Arc::new(ExclusionFilter::default());

        let indexer = IncrementalIndexer::new(db.clone(), config, filter);

        let stats = indexer.update(root, None).unwrap();
        assert!(stats.added > 0);

        fs::remove_file(&file_path).unwrap();

        let stats = indexer.update(root, None).unwrap();
        assert!(stats.removed > 0);
    }
}
