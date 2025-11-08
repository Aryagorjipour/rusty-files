use crate::core::config::{SearchConfig, SearchConfigBuilder};
use crate::core::error::Result;
use crate::core::types::{IndexStats, ProgressCallback, SearchResult};
use crate::filters::ExclusionFilter;
use crate::indexer::{IndexBuilder, IncrementalIndexer};
use crate::search::{Query, QueryParser, SearchExecutor};
use crate::storage::{Database, FileBloomFilter, LruCache};
use crate::watcher::FileSystemMonitor;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct SearchEngine {
    database: Arc<Database>,
    config: Arc<SearchConfig>,
    exclusion_filter: Arc<ExclusionFilter>,
    cache: Arc<LruCache>,
    bloom_filter: Arc<FileBloomFilter>,
    index_builder: Arc<IndexBuilder>,
    incremental_indexer: Arc<IncrementalIndexer>,
    search_executor: Arc<SearchExecutor>,
    monitor: Option<FileSystemMonitor>,
}

impl SearchEngine {
    pub fn new<P: AsRef<Path>>(index_path: P) -> Result<Self> {
        let config = SearchConfig::default();
        Self::with_config(index_path, config)
    }

    pub fn with_config<P: AsRef<Path>>(index_path: P, config: SearchConfig) -> Result<Self> {
        let database = Arc::new(Database::new(index_path, config.db_pool_size)?);
        let config = Arc::new(config);

        let exclusion_rules = database.get_exclusion_rules()?;
        let exclusion_filter = if exclusion_rules.is_empty() {
            Arc::new(ExclusionFilter::from_patterns(&config.exclusion_patterns)?)
        } else {
            Arc::new(ExclusionFilter::new(exclusion_rules)?)
        };

        let cache = Arc::new(LruCache::new(config.cache_size));
        let bloom_filter = Arc::new(FileBloomFilter::new(
            config.bloom_filter_capacity,
            config.bloom_filter_error_rate,
        ));

        let index_builder = Arc::new(IndexBuilder::new(
            Arc::clone(&database),
            Arc::clone(&config),
            Arc::clone(&exclusion_filter),
        ));

        let incremental_indexer = Arc::new(IncrementalIndexer::new(
            Arc::clone(&database),
            Arc::clone(&config),
            Arc::clone(&exclusion_filter),
        ));

        let search_executor = Arc::new(SearchExecutor::new(
            Arc::clone(&database),
            Arc::clone(&config),
            Arc::clone(&cache),
            Arc::clone(&bloom_filter),
        ));

        Ok(Self {
            database,
            config,
            exclusion_filter,
            cache,
            bloom_filter,
            index_builder,
            incremental_indexer,
            search_executor,
            monitor: None,
        })
    }

    pub fn builder() -> SearchEngineBuilder {
        SearchEngineBuilder::new()
    }

    pub fn index_directory<P: AsRef<Path>>(
        &self,
        root: P,
        progress_callback: Option<ProgressCallback>,
    ) -> Result<usize> {
        self.index_builder.build(root, progress_callback)
    }

    pub fn update_index<P: AsRef<Path>>(
        &self,
        root: P,
        progress_callback: Option<ProgressCallback>,
    ) -> Result<crate::indexer::UpdateStats> {
        self.incremental_indexer.update(root, progress_callback)
    }

    pub fn search(&self, query_str: &str) -> Result<Vec<SearchResult>> {
        let query = QueryParser::parse(query_str)?;
        self.search_executor.execute(&query)
    }

    pub fn search_with_query(&self, query: &Query) -> Result<Vec<SearchResult>> {
        self.search_executor.execute(query)
    }

    pub fn start_watching<P: AsRef<Path>>(&mut self, root: P) -> Result<()> {
        if self.monitor.is_none() {
            let mut monitor = FileSystemMonitor::new(
                Arc::clone(&self.database),
                Arc::clone(&self.config),
                Arc::clone(&self.exclusion_filter),
            );

            monitor.start(root)?;
            self.monitor = Some(monitor);
        }

        Ok(())
    }

    pub fn stop_watching(&mut self) -> Result<()> {
        if let Some(mut monitor) = self.monitor.take() {
            monitor.stop()?;
        }
        Ok(())
    }

    pub fn is_watching(&self) -> bool {
        self.monitor.as_ref().map(|m| m.is_running()).unwrap_or(false)
    }

    pub fn get_stats(&self) -> Result<IndexStats> {
        self.database.get_stats()
    }

    pub fn clear_index(&self) -> Result<()> {
        self.database.clear_all()?;
        self.cache.clear();
        self.bloom_filter.clear();
        Ok(())
    }

    pub fn vacuum(&self) -> Result<()> {
        self.database.vacuum()
    }

    pub fn verify_index<P: AsRef<Path>>(
        &self,
        root: P,
    ) -> Result<crate::indexer::VerificationStats> {
        self.incremental_indexer.verify_index(root)
    }

    pub fn add_exclusion_pattern(&self, pattern: String) -> Result<()> {
        use crate::core::types::{ExclusionRule, ExclusionRuleType};

        let rule = ExclusionRule {
            pattern,
            rule_type: ExclusionRuleType::Glob,
        };

        self.database.add_exclusion_rule(&rule)?;
        Ok(())
    }

    pub fn get_config(&self) -> &SearchConfig {
        &self.config
    }

    pub fn cache_stats(&self) -> (usize, bool) {
        (self.cache.len(), self.cache.is_empty())
    }
}

pub struct SearchEngineBuilder {
    config_builder: SearchConfigBuilder,
    index_path: Option<PathBuf>,
}

impl SearchEngineBuilder {
    pub fn new() -> Self {
        Self {
            config_builder: SearchConfigBuilder::new(),
            index_path: None,
        }
    }

    pub fn index_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.index_path = Some(path.into());
        self.config_builder = self.config_builder.index_path(
            self.index_path.as_ref().unwrap().clone()
        );
        self
    }

    pub fn thread_count(mut self, count: usize) -> Self {
        self.config_builder = self.config_builder.thread_count(count);
        self
    }

    pub fn enable_content_search(mut self, enable: bool) -> Self {
        self.config_builder = self.config_builder.enable_content_search(enable);
        self
    }

    pub fn enable_fuzzy_search(mut self, enable: bool) -> Self {
        self.config_builder = self.config_builder.enable_fuzzy_search(enable);
        self
    }

    pub fn cache_size(mut self, size: usize) -> Self {
        self.config_builder = self.config_builder.cache_size(size);
        self
    }

    pub fn max_search_results(mut self, max: usize) -> Self {
        self.config_builder = self.config_builder.max_search_results(max);
        self
    }

    pub fn exclusion_patterns(mut self, patterns: Vec<String>) -> Self {
        self.config_builder = self.config_builder.exclusion_patterns(patterns);
        self
    }

    pub fn build(self) -> Result<SearchEngine> {
        let config = self.config_builder.build();
        let index_path = self.index_path.unwrap_or_else(|| config.index_path.clone());

        SearchEngine::with_config(index_path, config)
    }
}

impl Default for SearchEngineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_search_engine_creation() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = temp_dir.path().join("index.db");

        let engine = SearchEngine::new(&index_path).unwrap();
        assert!(!engine.is_watching());
    }

    #[test]
    fn test_search_engine_builder() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = temp_dir.path().join("index.db");

        let engine = SearchEngine::builder()
            .index_path(index_path)
            .thread_count(4)
            .enable_content_search(false)
            .build()
            .unwrap();

        assert_eq!(engine.get_config().thread_count, 4);
    }

    #[test]
    fn test_indexing_and_search() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().join("data");
        fs::create_dir(&root).unwrap();

        fs::write(root.join("test1.txt"), "content1").unwrap();
        fs::write(root.join("test2.txt"), "content2").unwrap();

        let index_path = temp_dir.path().join("index.db");
        let engine = SearchEngine::new(&index_path).unwrap();

        let count = engine.index_directory(&root, None).unwrap();
        assert!(count > 0);

        let results = engine.search("test").unwrap();
        assert!(!results.is_empty());
    }

    #[test]
    fn test_stats() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().join("data");
        fs::create_dir(&root).unwrap();

        fs::write(root.join("file.txt"), "content").unwrap();

        let index_path = temp_dir.path().join("index.db");
        let engine = SearchEngine::new(&index_path).unwrap();

        engine.index_directory(&root, None).unwrap();

        let stats = engine.get_stats().unwrap();
        assert!(stats.total_files > 0);
    }
}
