use crate::core::error::Result;
use crate::core::types::MatchMode;
use globset::{Glob, GlobMatcher};
use regex::Regex;
use std::sync::Arc;

pub trait Matcher: Send + Sync {
    fn is_match(&self, text: &str) -> bool;
    fn find_matches(&self, text: &str) -> Vec<(usize, usize)>;
}

pub struct ExactMatcher {
    pattern: String,
    case_sensitive: bool,
}

impl ExactMatcher {
    pub fn new(pattern: String, case_sensitive: bool) -> Self {
        Self {
            pattern,
            case_sensitive,
        }
    }
}

impl Matcher for ExactMatcher {
    fn is_match(&self, text: &str) -> bool {
        if self.case_sensitive {
            text.contains(&self.pattern)
        } else {
            text.to_lowercase().contains(&self.pattern.to_lowercase())
        }
    }

    fn find_matches(&self, text: &str) -> Vec<(usize, usize)> {
        let mut matches = Vec::new();
        let pattern = if self.case_sensitive {
            self.pattern.clone()
        } else {
            self.pattern.to_lowercase()
        };
        let search_text = if self.case_sensitive {
            text.to_string()
        } else {
            text.to_lowercase()
        };

        let mut start = 0;
        while let Some(pos) = search_text[start..].find(&pattern) {
            let absolute_pos = start + pos;
            matches.push((absolute_pos, pattern.len()));
            start = absolute_pos + 1;
        }

        matches
    }
}

pub struct RegexMatcher {
    regex: Regex,
}

impl RegexMatcher {
    pub fn new(pattern: &str) -> Result<Self> {
        Ok(Self {
            regex: Regex::new(pattern)?,
        })
    }

    pub fn new_case_insensitive(pattern: &str) -> Result<Self> {
        let pattern = format!("(?i){}", pattern);
        Ok(Self {
            regex: Regex::new(&pattern)?,
        })
    }
}

impl Matcher for RegexMatcher {
    fn is_match(&self, text: &str) -> bool {
        self.regex.is_match(text)
    }

    fn find_matches(&self, text: &str) -> Vec<(usize, usize)> {
        self.regex
            .find_iter(text)
            .map(|m| (m.start(), m.end() - m.start()))
            .collect()
    }
}

pub struct GlobPatternMatcher {
    matcher: GlobMatcher,
}

impl GlobPatternMatcher {
    pub fn new(pattern: &str) -> Result<Self> {
        let glob = Glob::new(pattern)?;
        Ok(Self {
            matcher: glob.compile_matcher(),
        })
    }
}

impl Matcher for GlobPatternMatcher {
    fn is_match(&self, text: &str) -> bool {
        self.matcher.is_match(text)
    }

    fn find_matches(&self, text: &str) -> Vec<(usize, usize)> {
        if self.is_match(text) {
            vec![(0, text.len())]
        } else {
            vec![]
        }
    }
}

pub struct CompositeMatcher {
    matchers: Vec<Arc<dyn Matcher>>,
    require_all: bool,
}

impl CompositeMatcher {
    pub fn new(matchers: Vec<Arc<dyn Matcher>>, require_all: bool) -> Self {
        Self {
            matchers,
            require_all,
        }
    }

    pub fn and(matchers: Vec<Arc<dyn Matcher>>) -> Self {
        Self::new(matchers, true)
    }

    pub fn or(matchers: Vec<Arc<dyn Matcher>>) -> Self {
        Self::new(matchers, false)
    }
}

impl Matcher for CompositeMatcher {
    fn is_match(&self, text: &str) -> bool {
        if self.require_all {
            self.matchers.iter().all(|m| m.is_match(text))
        } else {
            self.matchers.iter().any(|m| m.is_match(text))
        }
    }

    fn find_matches(&self, text: &str) -> Vec<(usize, usize)> {
        let mut all_matches = Vec::new();
        for matcher in &self.matchers {
            all_matches.extend(matcher.find_matches(text));
        }
        all_matches.sort_by_key(|(pos, _)| *pos);
        all_matches.dedup();
        all_matches
    }
}

pub fn create_matcher(pattern: &str, mode: MatchMode) -> Result<Arc<dyn Matcher>> {
    match mode {
        MatchMode::Exact => Ok(Arc::new(ExactMatcher::new(pattern.to_string(), true))),
        MatchMode::CaseInsensitive => {
            Ok(Arc::new(ExactMatcher::new(pattern.to_string(), false)))
        }
        MatchMode::Regex => Ok(Arc::new(RegexMatcher::new(pattern)?)),
        MatchMode::Glob => Ok(Arc::new(GlobPatternMatcher::new(pattern)?)),
        MatchMode::Fuzzy => Ok(Arc::new(ExactMatcher::new(pattern.to_string(), false))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_matcher() {
        let matcher = ExactMatcher::new("test".to_string(), true);
        assert!(matcher.is_match("this is a test"));
        assert!(!matcher.is_match("this is a TEST"));

        let matcher = ExactMatcher::new("test".to_string(), false);
        assert!(matcher.is_match("this is a TEST"));
    }

    #[test]
    fn test_regex_matcher() {
        let matcher = RegexMatcher::new(r"\d+").unwrap();
        assert!(matcher.is_match("test123"));
        assert!(!matcher.is_match("test"));

        let matches = matcher.find_matches("test123abc456");
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_glob_matcher() {
        let matcher = GlobPatternMatcher::new("*.txt").unwrap();
        assert!(matcher.is_match("file.txt"));
        assert!(!matcher.is_match("file.rs"));
    }

    #[test]
    fn test_composite_matcher_and() {
        let m1 = Arc::new(ExactMatcher::new("hello".to_string(), false));
        let m2 = Arc::new(ExactMatcher::new("world".to_string(), false));

        let composite = CompositeMatcher::and(vec![m1, m2]);
        assert!(composite.is_match("hello world"));
        assert!(!composite.is_match("hello there"));
    }

    #[test]
    fn test_composite_matcher_or() {
        let m1 = Arc::new(ExactMatcher::new("hello".to_string(), false));
        let m2 = Arc::new(ExactMatcher::new("world".to_string(), false));

        let composite = CompositeMatcher::or(vec![m1, m2]);
        assert!(composite.is_match("hello"));
        assert!(composite.is_match("world"));
        assert!(!composite.is_match("test"));
    }
}
