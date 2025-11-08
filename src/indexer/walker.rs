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

                    if self.is_cyclic(path) {
                        continue;
                    }

                    if !self.should_index(path) {
                        continue;
                    }

                    self.visited.insert(path.to_path_buf());
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

                if self.is_cyclic(path) {
                    return None;
                }

                if !self.should_index(path) {
                    return None;
                }

                self.visited.insert(path.to_path_buf());
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
            if self.visited.contains(&canonical) {
                return true;
            }

            let mut current = canonical.as_path();
            while let Some(parent) = current.parent() {
                if self.visited.contains(parent) && parent != canonical {
                    return true;
                }
                current = parent;
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

        let config = Arc::new(SearchConfig::default());
        let filter = Arc::new(ExclusionFilter::default());
        let walker = DirectoryWalker::new(config, filter);

        let paths = walker.walk(root).unwrap();
        assert!(!paths.is_empty());
    }

    #[test]
    fn test_hidden_file_exclusion() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        fs::write(root.join(".hidden"), "content").unwrap();
        fs::write(root.join("visible.txt"), "content").unwrap();

        let mut config = SearchConfig::default();
        config.index_hidden_files = false;

        let config = Arc::new(config);
        let filter = Arc::new(ExclusionFilter::default());
        let walker = DirectoryWalker::new(config, filter);

        let paths = walker.walk(root).unwrap();
        let visible_count = paths
            .iter()
            .filter(|p| !is_hidden(p))
            .count();

        assert!(visible_count > 0);
    }
}
