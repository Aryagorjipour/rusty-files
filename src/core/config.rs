use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    pub index_path: PathBuf,
    pub thread_count: usize,
    pub max_file_size_for_content: u64,
    pub enable_content_search: bool,
    pub enable_fuzzy_search: bool,
    pub fuzzy_threshold: f64,
    pub cache_size: usize,
    pub bloom_filter_capacity: usize,
    pub bloom_filter_error_rate: f64,
    pub max_search_results: usize,
    pub batch_size: usize,
    pub follow_symlinks: bool,
    pub index_hidden_files: bool,
    pub exclusion_patterns: Vec<String>,
    pub watch_debounce_ms: u64,
    pub enable_access_tracking: bool,
    pub db_pool_size: u32,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            index_path: PathBuf::from("./filesearch.db"),
            thread_count: num_cpus() * 2,
            max_file_size_for_content: 10 * 1024 * 1024,
            enable_content_search: false,
            enable_fuzzy_search: true,
            fuzzy_threshold: 0.7,
            cache_size: 1000,
            bloom_filter_capacity: 10_000_000,
            bloom_filter_error_rate: 0.0001,
            max_search_results: 1000,
            batch_size: 1000,
            follow_symlinks: false,
            index_hidden_files: false,
            exclusion_patterns: vec![
                ".git".to_string(),
                "node_modules".to_string(),
                "target".to_string(),
                ".DS_Store".to_string(),
            ],
            watch_debounce_ms: 500,
            enable_access_tracking: true,
            db_pool_size: 10,
        }
    }
}

impl SearchConfig {
    pub fn from_file(path: &PathBuf) -> crate::core::error::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = if path.extension().and_then(|s| s.to_str()) == Some("json") {
            serde_json::from_str(&content)
                .map_err(|e| crate::core::error::SearchError::Configuration(e.to_string()))?
        } else {
            toml::from_str(&content)
                .map_err(|e| crate::core::error::SearchError::Configuration(e.to_string()))?
        };
        Ok(config)
    }

    pub fn to_file(&self, path: &PathBuf) -> crate::core::error::Result<()> {
        let content = if path.extension().and_then(|s| s.to_str()) == Some("json") {
            serde_json::to_string_pretty(self)
                .map_err(|e| crate::core::error::SearchError::Configuration(e.to_string()))?
        } else {
            toml::to_string_pretty(self)
                .map_err(|e| crate::core::error::SearchError::Configuration(e.to_string()))?
        };
        std::fs::write(path, content)?;
        Ok(())
    }
}

pub struct SearchConfigBuilder {
    config: SearchConfig,
}

impl SearchConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: SearchConfig::default(),
        }
    }

    pub fn index_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.config.index_path = path.into();
        self
    }

    pub fn thread_count(mut self, count: usize) -> Self {
        self.config.thread_count = count;
        self
    }

    pub fn max_file_size_for_content(mut self, size: u64) -> Self {
        self.config.max_file_size_for_content = size;
        self
    }

    pub fn enable_content_search(mut self, enable: bool) -> Self {
        self.config.enable_content_search = enable;
        self
    }

    pub fn enable_fuzzy_search(mut self, enable: bool) -> Self {
        self.config.enable_fuzzy_search = enable;
        self
    }

    pub fn fuzzy_threshold(mut self, threshold: f64) -> Self {
        self.config.fuzzy_threshold = threshold;
        self
    }

    pub fn cache_size(mut self, size: usize) -> Self {
        self.config.cache_size = size;
        self
    }

    pub fn max_search_results(mut self, max: usize) -> Self {
        self.config.max_search_results = max;
        self
    }

    pub fn batch_size(mut self, size: usize) -> Self {
        self.config.batch_size = size;
        self
    }

    pub fn follow_symlinks(mut self, follow: bool) -> Self {
        self.config.follow_symlinks = follow;
        self
    }

    pub fn index_hidden_files(mut self, index: bool) -> Self {
        self.config.index_hidden_files = index;
        self
    }

    pub fn exclusion_patterns(mut self, patterns: Vec<String>) -> Self {
        self.config.exclusion_patterns = patterns;
        self
    }

    pub fn add_exclusion_pattern<S: Into<String>>(mut self, pattern: S) -> Self {
        self.config.exclusion_patterns.push(pattern.into());
        self
    }

    pub fn watch_debounce_ms(mut self, ms: u64) -> Self {
        self.config.watch_debounce_ms = ms;
        self
    }

    pub fn enable_access_tracking(mut self, enable: bool) -> Self {
        self.config.enable_access_tracking = enable;
        self
    }

    pub fn db_pool_size(mut self, size: u32) -> Self {
        self.config.db_pool_size = size;
        self
    }

    pub fn build(self) -> SearchConfig {
        self.config
    }
}

impl Default for SearchConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}
