use crate::core::config::SearchConfig;
use crate::core::error::Result;
use crate::core::types::{FileEntry, MatchMode, SearchResult, SearchScope};
use crate::filters::{apply_date_filter, apply_extension_filter, apply_size_filter};
use crate::search::fuzzy::FuzzyMatcher;
use crate::search::matcher::create_matcher;
use crate::search::query::Query;
use crate::search::ranker::ResultRanker;
use crate::storage::{Database, FileBloomFilter, LruCache};
use std::sync::Arc;

pub struct SearchExecutor {
    database: Arc<Database>,
    config: Arc<SearchConfig>,
    _cache: Arc<LruCache>,
    _bloom_filter: Arc<FileBloomFilter>,
    ranker: ResultRanker,
}

impl SearchExecutor {
    pub fn new(
        database: Arc<Database>,
        config: Arc<SearchConfig>,
        cache: Arc<LruCache>,
        bloom_filter: Arc<FileBloomFilter>,
    ) -> Self {
        let ranker = ResultRanker::new(config.fuzzy_threshold);

        Self {
            database,
            config,
            _cache: cache,
            _bloom_filter: bloom_filter,
            ranker,
        }
    }

    pub fn execute(&self, query: &Query) -> Result<Vec<SearchResult>> {
        if self.config.enable_fuzzy_search && query.match_mode == MatchMode::Fuzzy {
            return self.execute_fuzzy_search(query);
        }

        let candidates = self.get_candidates(query)?;
        let filtered = self.apply_filters(candidates, query)?;
        let matched = self.apply_matchers(filtered, query)?;
        let results = self.create_search_results(matched, query);

        let ranked = self.ranker.rank(results, &query.pattern);

        let max_results = query
            .max_results
            .unwrap_or(self.config.max_search_results);

        Ok(ranked.into_iter().take(max_results).collect())
    }

    fn get_candidates(&self, query: &Query) -> Result<Vec<FileEntry>> {
        match query.scope {
            SearchScope::Name => {
                if !query.extensions.is_empty() && query.extensions.len() == 1 {
                    self.database.search_by_extension(
                        &query.extensions[0],
                        self.config.max_search_results * 2,
                    )
                } else {
                    self.database.search_by_name(
                        &query.pattern,
                        self.config.max_search_results * 2,
                    )
                }
            }
            SearchScope::Path => self.database.search_by_name(
                &query.pattern,
                self.config.max_search_results * 2,
            ),
            SearchScope::Content => {
                if self.config.enable_content_search {
                    let file_ids = self.database.search_content(
                        &query.pattern,
                        self.config.max_search_results * 2,
                    )?;

                    let mut files = Vec::new();
                    for id in file_ids {
                        if let Ok(Some(file)) = self.database.find_by_id(id) {
                            files.push(file);
                        }
                    }
                    Ok(files)
                } else {
                    Ok(Vec::new())
                }
            }
            SearchScope::All => self.database.get_all_files(
                self.config.max_search_results * 2,
                0,
            ),
        }
    }

    fn apply_filters(&self, candidates: Vec<FileEntry>, query: &Query) -> Result<Vec<FileEntry>> {
        let filtered = candidates
            .into_iter()
            .filter(|entry| {
                if !query.extensions.is_empty()
                    && !apply_extension_filter(entry, &query.extensions)
                {
                    return false;
                }

                if let Some(ref size_filter) = query.size_filter {
                    if !apply_size_filter(entry, size_filter) {
                        return false;
                    }
                }

                if let Some(ref date_filter) = query.date_filter {
                    if !apply_date_filter(entry, date_filter) {
                        return false;
                    }
                }

                true
            })
            .collect();

        Ok(filtered)
    }

    fn apply_matchers(&self, candidates: Vec<FileEntry>, query: &Query) -> Result<Vec<FileEntry>> {
        let matcher = create_matcher(&query.pattern, query.match_mode)?;

        let matched = candidates
            .into_iter()
            .filter(|entry| {
                match query.scope {
                    SearchScope::Name => matcher.is_match(&entry.name),
                    SearchScope::Path => matcher.is_match(&entry.path.to_string_lossy()),
                    SearchScope::Content => true,
                    SearchScope::All => matcher.is_match(&entry.name),
                }
            })
            .collect();

        Ok(matched)
    }

    fn execute_fuzzy_search(&self, query: &Query) -> Result<Vec<SearchResult>> {
        let fuzzy_matcher = FuzzyMatcher::new(self.config.fuzzy_threshold);
        let mut all_files = self.database.get_all_files(10000, 0)?;

        if !query.extensions.is_empty() {
            all_files.retain(|f| apply_extension_filter(f, &query.extensions));
        }

        if let Some(ref size_filter) = query.size_filter {
            all_files.retain(|f| apply_size_filter(f, size_filter));
        }

        if let Some(ref date_filter) = query.date_filter {
            all_files.retain(|f| apply_date_filter(f, date_filter));
        }

        let mut scored_results: Vec<(FileEntry, i64)> = all_files
            .into_iter()
            .filter_map(|entry| {
                fuzzy_matcher
                    .fuzzy_match_with_threshold(&entry.name, &query.pattern)
                    .map(|score| (entry, score))
            })
            .collect();

        scored_results.sort_by(|a, b| b.1.cmp(&a.1));

        let max_results = query
            .max_results
            .unwrap_or(self.config.max_search_results);

        let results: Vec<SearchResult> = scored_results
            .into_iter()
            .take(max_results)
            .map(|(file, score)| SearchResult {
                file,
                score: score as f64 / 100.0,
                snippet: None,
                matches: vec![],
            })
            .collect();

        Ok(results)
    }

    fn create_search_results(&self, files: Vec<FileEntry>, _query: &Query) -> Vec<SearchResult> {
        files
            .into_iter()
            .map(|file| SearchResult {
                file,
                score: 0.0,
                snippet: None,
                matches: vec![],
            })
            .collect()
    }

    pub fn search_with_cache(&self, query: &Query) -> Result<Vec<SearchResult>> {
        self.execute(query)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::SearchConfig;
    use crate::filters::ExclusionFilter;
    use crate::indexer::builder::IndexBuilder;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_search_executor() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        fs::write(root.join("test1.txt"), "content1").unwrap();
        fs::write(root.join("test2.txt"), "content2").unwrap();
        fs::write(root.join("other.rs"), "content3").unwrap();

        let db = Arc::new(Database::in_memory(10).unwrap());
        // Enable hidden files indexing since temp dirs often start with a dot
        let mut config = SearchConfig::default();
        config.index_hidden_files = true;
        let config = Arc::new(config);
        // Use empty exclusion filter to avoid any pattern matching issues
        let filter = Arc::new(ExclusionFilter::from_patterns(&[]).unwrap());

        let builder = IndexBuilder::new(db.clone(), config.clone(), filter);
        builder.build(root, None).unwrap();

        let cache = Arc::new(LruCache::new(100));
        let bloom = Arc::new(FileBloomFilter::default());

        let executor = SearchExecutor::new(db, config, cache, bloom);

        let query = Query::new("test".to_string());
        let results = executor.execute(&query).unwrap();

        assert!(!results.is_empty(), "Expected at least one search result");
    }

    #[test]
    fn test_search_with_extension_filter() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        fs::write(root.join("file1.txt"), "content1").unwrap();
        fs::write(root.join("file2.rs"), "content2").unwrap();

        let db = Arc::new(Database::in_memory(10).unwrap());
        // Enable hidden files indexing since temp dirs often start with a dot
        let mut config = SearchConfig::default();
        config.index_hidden_files = true;
        let config = Arc::new(config);
        // Use empty exclusion filter to avoid any pattern matching issues
        let filter = Arc::new(ExclusionFilter::from_patterns(&[]).unwrap());

        let builder = IndexBuilder::new(db.clone(), config.clone(), filter);
        builder.build(root, None).unwrap();

        let cache = Arc::new(LruCache::new(100));
        let bloom = Arc::new(FileBloomFilter::default());

        let executor = SearchExecutor::new(db, config, cache, bloom);

        let query = Query::new("file".to_string()).with_extensions(vec!["rs".to_string()]);
        let results = executor.execute(&query).unwrap();

        assert_eq!(results.len(), 1, "Expected exactly one search result");
        assert_eq!(results[0].file.name, "file2.rs");
    }
}
