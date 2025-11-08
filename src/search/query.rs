use crate::core::error::{Result, SearchError};
use crate::core::types::{DateFilter, MatchMode, SearchScope, SizeFilter};
use crate::filters::{parse_relative_date, parse_size};

#[derive(Debug, Clone)]
pub struct Query {
    pub pattern: String,
    pub match_mode: MatchMode,
    pub scope: SearchScope,
    pub size_filter: Option<SizeFilter>,
    pub date_filter: Option<DateFilter>,
    pub extensions: Vec<String>,
    pub max_results: Option<usize>,
}

impl Query {
    pub fn new(pattern: String) -> Self {
        Self {
            pattern,
            match_mode: MatchMode::CaseInsensitive,
            scope: SearchScope::Name,
            size_filter: None,
            date_filter: None,
            extensions: Vec::new(),
            max_results: None,
        }
    }

    pub fn with_match_mode(mut self, mode: MatchMode) -> Self {
        self.match_mode = mode;
        self
    }

    pub fn with_scope(mut self, scope: SearchScope) -> Self {
        self.scope = scope;
        self
    }

    pub fn with_size_filter(mut self, filter: SizeFilter) -> Self {
        self.size_filter = Some(filter);
        self
    }

    pub fn with_date_filter(mut self, filter: DateFilter) -> Self {
        self.date_filter = Some(filter);
        self
    }

    pub fn with_extensions(mut self, extensions: Vec<String>) -> Self {
        self.extensions = extensions;
        self
    }

    pub fn with_max_results(mut self, max: usize) -> Self {
        self.max_results = Some(max);
        self
    }
}

pub struct QueryParser;

impl QueryParser {
    pub fn parse(input: &str) -> Result<Query> {
        let mut query = Query::new(String::new());
        let parts: Vec<&str> = input.split_whitespace().collect();

        let mut pattern_parts = Vec::new();
        let mut i = 0;

        while i < parts.len() {
            let part = parts[i];

            if part.contains(':') {
                let (key, value) = part.split_once(':').unwrap();
                match key.to_lowercase().as_str() {
                    "ext" | "extension" => {
                        query.extensions = value.split(',').map(|s| s.to_string()).collect();
                    }
                    "size" => {
                        query.size_filter = Self::parse_size_filter(value)?;
                    }
                    "modified" | "date" => {
                        query.date_filter = Self::parse_date_filter(value)?;
                    }
                    "mode" => {
                        query.match_mode = Self::parse_match_mode(value)?;
                    }
                    "scope" => {
                        query.scope = Self::parse_scope(value)?;
                    }
                    "limit" | "max" => {
                        if let Ok(max) = value.parse::<usize>() {
                            query.max_results = Some(max);
                        }
                    }
                    _ => {
                        pattern_parts.push(part);
                    }
                }
            } else {
                pattern_parts.push(part);
            }

            i += 1;
        }

        query.pattern = pattern_parts.join(" ");

        if query.pattern.is_empty() {
            return Err(SearchError::InvalidQuery(
                "Query pattern cannot be empty".to_string(),
            ));
        }

        Ok(query)
    }

    fn parse_size_filter(value: &str) -> Result<Option<SizeFilter>> {
        if value.starts_with('>') {
            let size_str = value.trim_start_matches('>');
            if let Some(size) = parse_size(size_str) {
                return Ok(Some(SizeFilter::GreaterThan(size)));
            }
        } else if value.starts_with('<') {
            let size_str = value.trim_start_matches('<');
            if let Some(size) = parse_size(size_str) {
                return Ok(Some(SizeFilter::LessThan(size)));
            }
        } else if value.contains("..") {
            let parts: Vec<&str> = value.split("..").collect();
            if parts.len() == 2 {
                if let (Some(min), Some(max)) = (parse_size(parts[0]), parse_size(parts[1])) {
                    return Ok(Some(SizeFilter::Range(min, max)));
                }
            }
        } else if let Some(size) = parse_size(value) {
            return Ok(Some(SizeFilter::Exact(size)));
        }

        Err(SearchError::InvalidQuery(format!(
            "Invalid size filter: {}",
            value
        )))
    }

    fn parse_date_filter(value: &str) -> Result<Option<DateFilter>> {
        if value.starts_with('>') || value.starts_with("after:") {
            let date_str = value.trim_start_matches('>').trim_start_matches("after:");
            if let Some(date) = parse_relative_date(date_str) {
                return Ok(Some(DateFilter::After(date)));
            }
        } else if value.starts_with('<') || value.starts_with("before:") {
            let date_str = value.trim_start_matches('<').trim_start_matches("before:");
            if let Some(date) = parse_relative_date(date_str) {
                return Ok(Some(DateFilter::Before(date)));
            }
        } else if value.contains("..") {
            let parts: Vec<&str> = value.split("..").collect();
            if parts.len() == 2 {
                if let (Some(start), Some(end)) =
                    (parse_relative_date(parts[0]), parse_relative_date(parts[1]))
                {
                    return Ok(Some(DateFilter::Between(start, end)));
                }
            }
        } else if let Some(date) = parse_relative_date(value) {
            return Ok(Some(DateFilter::On(date)));
        }

        Err(SearchError::InvalidQuery(format!(
            "Invalid date filter: {}",
            value
        )))
    }

    fn parse_match_mode(value: &str) -> Result<MatchMode> {
        match value.to_lowercase().as_str() {
            "exact" => Ok(MatchMode::Exact),
            "case" | "casesensitive" => Ok(MatchMode::Exact),
            "insensitive" | "caseinsensitive" => Ok(MatchMode::CaseInsensitive),
            "fuzzy" => Ok(MatchMode::Fuzzy),
            "regex" => Ok(MatchMode::Regex),
            "glob" => Ok(MatchMode::Glob),
            _ => Err(SearchError::InvalidQuery(format!(
                "Invalid match mode: {}",
                value
            ))),
        }
    }

    fn parse_scope(value: &str) -> Result<SearchScope> {
        match value.to_lowercase().as_str() {
            "name" => Ok(SearchScope::Name),
            "path" => Ok(SearchScope::Path),
            "content" => Ok(SearchScope::Content),
            "all" => Ok(SearchScope::All),
            _ => Err(SearchError::InvalidQuery(format!(
                "Invalid search scope: {}",
                value
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_query() {
        let query = QueryParser::parse("test.txt").unwrap();
        assert_eq!(query.pattern, "test.txt");
        assert_eq!(query.match_mode, MatchMode::CaseInsensitive);
    }

    #[test]
    fn test_parse_query_with_extension() {
        let query = QueryParser::parse("test ext:rs").unwrap();
        assert_eq!(query.pattern, "test");
        assert_eq!(query.extensions, vec!["rs"]);
    }

    #[test]
    fn test_parse_query_with_size() {
        let query = QueryParser::parse("test size:>1MB").unwrap();
        assert_eq!(query.pattern, "test");
        assert!(query.size_filter.is_some());
    }

    #[test]
    fn test_parse_query_with_date() {
        let query = QueryParser::parse("test modified:today").unwrap();
        assert_eq!(query.pattern, "test");
        assert!(query.date_filter.is_some());
    }

    #[test]
    fn test_parse_query_with_mode() {
        let query = QueryParser::parse("test mode:fuzzy").unwrap();
        assert_eq!(query.pattern, "test");
        assert_eq!(query.match_mode, MatchMode::Fuzzy);
    }

    #[test]
    fn test_parse_complex_query() {
        let query = QueryParser::parse("test ext:rs,txt size:>100KB modified:today mode:fuzzy").unwrap();
        assert_eq!(query.pattern, "test");
        assert_eq!(query.extensions.len(), 2);
        assert!(query.size_filter.is_some());
        assert!(query.date_filter.is_some());
        assert_eq!(query.match_mode, MatchMode::Fuzzy);
    }
}
