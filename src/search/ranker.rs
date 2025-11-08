use crate::core::types::{FileEntry, SearchResult};
use crate::search::fuzzy::{similarity_score, starts_with_score, FuzzyMatcher};
use crate::utils::path::get_path_depth;
use std::cmp::Ordering;

pub struct ResultRanker {
    fuzzy_matcher: FuzzyMatcher,
}

impl ResultRanker {
    pub fn new(fuzzy_threshold: f64) -> Self {
        Self {
            fuzzy_matcher: FuzzyMatcher::new(fuzzy_threshold),
        }
    }

    pub fn rank(&self, results: Vec<SearchResult>, query: &str) -> Vec<SearchResult> {
        let mut ranked_results = results;

        for result in &mut ranked_results {
            result.score = self.calculate_score(&result.file, query);
        }

        ranked_results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(Ordering::Equal)
                .then_with(|| a.file.name.cmp(&b.file.name))
        });

        ranked_results
    }

    pub fn calculate_score(&self, file: &FileEntry, query: &str) -> f64 {
        let name_match_score = self.name_match_score(&file.name, query);
        let path_depth_penalty = self.path_depth_penalty(file);
        let recency_score = self.recency_score(file);

        let weights = ScoreWeights {
            name_match: 0.5,
            path_depth: 0.2,
            recency: 0.3,
        };

        weights.name_match * name_match_score
            + weights.path_depth * path_depth_penalty
            + weights.recency * recency_score
    }

    fn name_match_score(&self, name: &str, query: &str) -> f64 {
        let exact_match = if name.eq_ignore_ascii_case(query) {
            1.0
        } else {
            0.0
        };

        if exact_match > 0.0 {
            return exact_match;
        }

        let starts_with = starts_with_score(name, query);
        if starts_with > 0.0 {
            return 0.9 * starts_with;
        }

        let fuzzy_score = self.fuzzy_matcher.score_normalized(name, query);
        if fuzzy_score > 0.0 {
            return 0.7 * fuzzy_score;
        }

        let similarity = similarity_score(name, query);
        0.5 * similarity
    }

    fn path_depth_penalty(&self, file: &FileEntry) -> f64 {
        let depth = get_path_depth(&file.path);
        let max_depth = 20.0;
        let normalized_depth = (depth as f64 / max_depth).min(1.0);

        1.0 - (normalized_depth * 0.5)
    }

    fn recency_score(&self, file: &FileEntry) -> f64 {
        use chrono::Utc;

        if let Some(modified) = file.modified_at {
            let now = Utc::now();
            let age = now.signed_duration_since(modified);
            let days = age.num_days() as f64;

            if days < 1.0 {
                1.0
            } else if days < 7.0 {
                0.9
            } else if days < 30.0 {
                0.7
            } else if days < 90.0 {
                0.5
            } else if days < 365.0 {
                0.3
            } else {
                0.1
            }
        } else {
            0.5
        }
    }

    pub fn boost_by_extension(&self, mut results: Vec<SearchResult>, preferred_extensions: &[String]) -> Vec<SearchResult> {
        for result in &mut results {
            if let Some(ref ext) = result.file.extension {
                if preferred_extensions.iter().any(|e| e.eq_ignore_ascii_case(ext)) {
                    result.score *= 1.2;
                }
            }
        }

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(Ordering::Equal)
        });

        results
    }

    pub fn boost_by_size(&self, mut results: Vec<SearchResult>, prefer_smaller: bool) -> Vec<SearchResult> {
        if results.is_empty() {
            return results;
        }

        let sizes: Vec<u64> = results.iter().map(|r| r.file.size).collect();
        let max_size = *sizes.iter().max().unwrap_or(&1) as f64;

        for result in &mut results {
            let size_ratio = result.file.size as f64 / max_size;
            let size_score = if prefer_smaller {
                1.0 - size_ratio
            } else {
                size_ratio
            };

            result.score *= 1.0 + (size_score * 0.1);
        }

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(Ordering::Equal)
        });

        results
    }
}

impl Default for ResultRanker {
    fn default() -> Self {
        Self::new(0.7)
    }
}

struct ScoreWeights {
    name_match: f64,
    path_depth: f64,
    recency: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use chrono::Utc;

    fn create_test_file(name: &str, path: &str) -> FileEntry {
        FileEntry {
            id: Some(1),
            path: PathBuf::from(path),
            name: name.to_string(),
            extension: Some("txt".to_string()),
            size: 1024,
            created_at: Some(Utc::now()),
            modified_at: Some(Utc::now()),
            accessed_at: None,
            is_directory: false,
            is_hidden: false,
            is_symlink: false,
            parent_path: None,
            mime_type: None,
            file_hash: None,
            indexed_at: Utc::now(),
            last_verified: Utc::now(),
        }
    }

    #[test]
    fn test_name_match_score() {
        let ranker = ResultRanker::default();
        let file = create_test_file("test.txt", "/path/test.txt");

        let score = ranker.calculate_score(&file, "test");
        assert!(score > 0.0);
    }

    #[test]
    fn test_ranking_order() {
        let ranker = ResultRanker::default();

        let results = vec![
            SearchResult {
                file: create_test_file("other.txt", "/deep/path/other.txt"),
                score: 0.0,
                snippet: None,
                matches: vec![],
            },
            SearchResult {
                file: create_test_file("test.txt", "/test.txt"),
                score: 0.0,
                snippet: None,
                matches: vec![],
            },
        ];

        let ranked = ranker.rank(results, "test");
        assert_eq!(ranked[0].file.name, "test.txt");
    }

    #[test]
    fn test_boost_by_extension() {
        let ranker = ResultRanker::default();

        let mut results = vec![
            SearchResult {
                file: create_test_file("file1.rs", "/file1.rs"),
                score: 0.5,
                snippet: None,
                matches: vec![],
            },
            SearchResult {
                file: create_test_file("file2.txt", "/file2.txt"),
                score: 0.5,
                snippet: None,
                matches: vec![],
            },
        ];

        results[0].file.extension = Some("rs".to_string());
        results[1].file.extension = Some("txt".to_string());

        let boosted = ranker.boost_by_extension(results, &["rs".to_string()]);
        assert_eq!(boosted[0].file.extension, Some("rs".to_string()));
    }
}
