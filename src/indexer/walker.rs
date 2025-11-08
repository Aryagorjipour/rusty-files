use crate::core::config::SearchConfig;
use crate::core::error::Result;
use crate::filters::ExclusionFilter;
use crate::utils::path::is_hidden;
use dashmap::DashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use walkdir::{DirEntry, WalkDir};

pub struct DirectoryWalker {
    config: Arc<SearchConfig>,
    exclusion_filter: Arc<ExclusionFilter>,
    visited: Arc<DashSet<PathBuf>>,
}

impl DirectoryWalker {
    pub fn new(config: Arc<SearchConfig>, exclusion_filter: Arc<ExclusionFilter>) -> Self {
        Self {
            config,
            exclusion_filter,
            visited: Arc::new(DashSet::new()),
        }
    }

    pub fn walk<P: AsRef<Path>>(&self, root: P) -> Result<Vec<PathBuf>> {
        let root = root.as_ref();
        let mut paths = Vec::new();

        for entry in WalkDir::new(root)
            .follow_links(self.config.follow_symlinks)
            .into_iter()
            .filter_entry(|e| self.should_visit(e))
        {
            match entry {
                Ok(entry) => {
                    let path = entry.path();

                    if !self.should_index(path) {
                        continue;
                    }

                    if self.is_cyclic(path) {
                        continue;
                    }

                    // Insert canonical path to match is_cyclic check
                    if let Ok(canonical) = dunce::canonicalize(path) {
                        self.visited.insert(canonical);
                    } else {
                        self.visited.insert(path.to_path_buf());
                    }
                    paths.push(path.to_path_buf());
                }
                Err(e) => {
                    log::warn!("Error walking directory: {}", e);
                }
            }
        }

        Ok(paths)
    }

    pub fn walk_parallel<P: AsRef<Path>>(
        &self,
        root: P,
    ) -> Result<Vec<PathBuf>> {
        use rayon::prelude::*;

        let root = root.as_ref();
        let entries: Vec<_> = WalkDir::new(root)
            .follow_links(self.config.follow_symlinks)
            .into_iter()
            .filter_entry(|e| self.should_visit(e))
            .filter_map(|e| e.ok())
            .collect();

        let paths: Vec<PathBuf> = entries
            .par_iter()
            .filter_map(|entry| {
                let path = entry.path();

                if !self.should_index(path) {
                    return None;
                }

                if self.is_cyclic(path) {
                    return None;
                }

                // Insert canonical path to match is_cyclic check
                if let Ok(canonical) = dunce::canonicalize(path) {
                    self.visited.insert(canonical);
                } else {
                    self.visited.insert(path.to_path_buf());
                }
                Some(path.to_path_buf())
            })
            .collect();

        Ok(paths)
    }

    fn should_visit(&self, entry: &DirEntry) -> bool {
        let path = entry.path();

        if self.exclusion_filter.is_excluded(path) {
            return false;
        }

        if !self.config.index_hidden_files && is_hidden(path) {
            return false;
        }

        true
    }

    fn should_index(&self, path: &Path) -> bool {
        // Only index files, not directories
        if path.is_dir() {
            return false;
        }

        if self.exclusion_filter.is_excluded(path) {
            return false;
        }

        if !self.config.index_hidden_files && is_hidden(path) {
            return false;
        }

        true
    }

    fn is_cyclic(&self, path: &Path) -> bool {
        if let Ok(canonical) = dunce::canonicalize(path) {
            // Only check if we've already visited this exact path
            if self.visited.contains(&canonical) {
                return true;
            }
        }

        false
    }

    pub fn clear_visited(&self) {
        self.visited.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::SearchConfig;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_directory_walker() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        fs::create_dir(root.join("dir1")).unwrap();
        fs::write(root.join("file1.txt"), "content").unwrap();
        fs::write(root.join("dir1/file2.txt"), "content").unwrap();

        // Enable hidden files indexing since temp dirs often start with a dot
        let mut config = SearchConfig::default();
        config.index_hidden_files = true;
        let config = Arc::new(config);
        // Use empty exclusion filter to avoid any pattern matching issues
        let filter = Arc::new(ExclusionFilter::from_patterns(&[]).unwrap());
        let walker = DirectoryWalker::new(config, filter);

        let paths = walker.walk(root).unwrap();
        assert!(!paths.is_empty(), "Expected at least 2 files but found {}", paths.len());
        assert_eq!(paths.len(), 2, "Expected exactly 2 files");
    }

    #[test]
    fn test_hidden_file_exclusion() {
        let temp_dir = TempDir::new().unwrap();
        // Create a subdirectory that doesn't start with a dot to avoid issues with temp dir names
        let test_root = temp_dir.path().join("test_dir");
        fs::create_dir(&test_root).unwrap();

        fs::write(test_root.join(".hidden"), "content").unwrap();
        fs::write(test_root.join("visible.txt"), "content").unwrap();

        // First test with hidden files enabled to make sure they're both indexed
        let mut config = SearchConfig::default();
        config.index_hidden_files = true;
        let config_all = Arc::new(config);
        let filter = Arc::new(ExclusionFilter::from_patterns(&[]).unwrap());
        let walker_all = DirectoryWalker::new(config_all, filter.clone());
        let all_paths = walker_all.walk(&test_root).unwrap();
        assert_eq!(all_paths.len(), 2, "Expected both files when indexing hidden files");

        // Now test with hidden files disabled
        let mut config = SearchConfig::default();
        config.index_hidden_files = false;
        let config = Arc::new(config);
        let walker = DirectoryWalker::new(config, filter);

        // Need to clear visited set from previous walk
        walker.clear_visited();

        let paths = walker.walk(&test_root).unwrap();
        // Should only get the visible file, not the hidden one
        assert_eq!(paths.len(), 1, "Expected only visible file");
        assert!(paths.iter().all(|p| !is_hidden(p)), "Should not have hidden files");
    }
}
