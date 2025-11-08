use crate::core::error::Result;
use crate::core::types::{ExclusionRule, ExclusionRuleType};
use globset::{Glob, GlobSet, GlobSetBuilder};
use regex::RegexSet;
use std::path::Path;

pub struct ExclusionFilter {
    glob_set: Option<GlobSet>,
    regex_set: Option<RegexSet>,
    path_patterns: Vec<String>,
}

impl ExclusionFilter {
    pub fn new(rules: Vec<ExclusionRule>) -> Result<Self> {
        let mut glob_builder = GlobSetBuilder::new();
        let mut regex_patterns = Vec::new();
        let mut path_patterns = Vec::new();

        for rule in rules {
            match rule.rule_type {
                ExclusionRuleType::Glob => {
                    let glob = Glob::new(&rule.pattern)?;
                    glob_builder.add(glob);
                }
                ExclusionRuleType::Regex => {
                    regex_patterns.push(rule.pattern);
                }
                ExclusionRuleType::Path => {
                    path_patterns.push(rule.pattern);
                }
            }
        }

        let glob_set = if glob_builder.build().is_ok() {
            Some(glob_builder.build()?)
        } else {
            None
        };

        let regex_set = if !regex_patterns.is_empty() {
            Some(RegexSet::new(regex_patterns)?)
        } else {
            None
        };

        Ok(Self {
            glob_set,
            regex_set,
            path_patterns,
        })
    }

    pub fn from_patterns(patterns: &[String]) -> Result<Self> {
        let rules = patterns
            .iter()
            .map(|p| ExclusionRule {
                pattern: p.clone(),
                rule_type: ExclusionRuleType::Glob,
            })
            .collect();

        Self::new(rules)
    }

    pub fn is_excluded<P: AsRef<Path>>(&self, path: P) -> bool {
        let path = path.as_ref();
        let path_str = path.to_string_lossy();

        if let Some(ref glob_set) = self.glob_set {
            if glob_set.is_match(path) {
                return true;
            }
        }

        if let Some(ref regex_set) = self.regex_set {
            if regex_set.is_match(&path_str) {
                return true;
            }
        }

        for pattern in &self.path_patterns {
            if path_str.contains(pattern) {
                return true;
            }
        }

        false
    }

    pub fn should_index<P: AsRef<Path>>(&self, path: P) -> bool {
        !self.is_excluded(path)
    }
}

impl Default for ExclusionFilter {
    fn default() -> Self {
        Self::from_patterns(&[
            ".git".to_string(),
            "node_modules".to_string(),
            "target".to_string(),
            ".DS_Store".to_string(),
        ])
        .unwrap()
    }
}

pub fn build_gitignore_filter<P: AsRef<Path>>(root: P) -> Result<ignore::gitignore::Gitignore> {
    let mut builder = ignore::gitignore::GitignoreBuilder::new(root.as_ref());

    let gitignore_path = root.as_ref().join(".gitignore");
    if gitignore_path.exists() {
        builder.add(gitignore_path);
    }

    builder.build()
        .map_err(|e| crate::core::error::SearchError::Configuration(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_exclusion_filter_glob() {
        let rules = vec![ExclusionRule {
            pattern: "*.tmp".to_string(),
            rule_type: ExclusionRuleType::Glob,
        }];

        let filter = ExclusionFilter::new(rules).unwrap();
        assert!(filter.is_excluded(PathBuf::from("test.tmp")));
        assert!(!filter.is_excluded(PathBuf::from("test.txt")));
    }

    #[test]
    fn test_exclusion_filter_path() {
        let rules = vec![ExclusionRule {
            pattern: "node_modules".to_string(),
            rule_type: ExclusionRuleType::Path,
        }];

        let filter = ExclusionFilter::new(rules).unwrap();
        assert!(filter.is_excluded(PathBuf::from("/project/node_modules/package")));
        assert!(!filter.is_excluded(PathBuf::from("/project/src/main.rs")));
    }

    #[test]
    fn test_default_exclusion_filter() {
        let filter = ExclusionFilter::default();
        assert!(filter.is_excluded(PathBuf::from(".git")));
        assert!(filter.is_excluded(PathBuf::from("node_modules")));
        assert!(filter.is_excluded(PathBuf::from("target")));
    }
}
