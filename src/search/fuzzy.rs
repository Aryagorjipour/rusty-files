use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher as FuzzyMatcherTrait;

pub struct FuzzyMatcher {
    matcher: SkimMatcherV2,
    threshold: i64,
}

impl FuzzyMatcher {
    pub fn new(threshold: f64) -> Self {
        Self {
            matcher: SkimMatcherV2::default(),
            threshold: (threshold * 100.0) as i64,
        }
    }

    pub fn fuzzy_match(&self, choice: &str, pattern: &str) -> Option<i64> {
        self.matcher.fuzzy_match(choice, pattern)
    }

    pub fn fuzzy_match_with_threshold(&self, choice: &str, pattern: &str) -> Option<i64> {
        if let Some(score) = self.matcher.fuzzy_match(choice, pattern) {
            if score >= self.threshold {
                return Some(score);
            }
        }
        None
    }

    pub fn fuzzy_indices(&self, choice: &str, pattern: &str) -> Option<(i64, Vec<usize>)> {
        self.matcher.fuzzy_indices(choice, pattern)
    }

    pub fn score_normalized(&self, choice: &str, pattern: &str) -> f64 {
        if let Some(score) = self.fuzzy_match(choice, pattern) {
            let max_score = pattern.len() as i64 * 16;
            (score as f64 / max_score as f64).min(1.0)
        } else {
            0.0
        }
    }
}

impl Default for FuzzyMatcher {
    fn default() -> Self {
        Self::new(0.5)
    }
}

pub fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.chars().count();
    let len2 = s2.chars().count();

    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    for i in 0..=len1 {
        matrix[i][0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }

    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();

    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if s1_chars[i - 1] == s2_chars[j - 1] { 0 } else { 1 };
            matrix[i][j] = (matrix[i - 1][j] + 1)
                .min(matrix[i][j - 1] + 1)
                .min(matrix[i - 1][j - 1] + cost);
        }
    }

    matrix[len1][len2]
}

pub fn similarity_score(s1: &str, s2: &str) -> f64 {
    let distance = levenshtein_distance(s1, s2);
    let max_len = s1.len().max(s2.len());

    if max_len == 0 {
        return 1.0;
    }

    1.0 - (distance as f64 / max_len as f64)
}

pub fn starts_with_score(text: &str, pattern: &str) -> f64 {
    let text_lower = text.to_lowercase();
    let pattern_lower = pattern.to_lowercase();

    if text_lower.starts_with(&pattern_lower) {
        1.0
    } else if text_lower.contains(&pattern_lower) {
        0.5
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_matcher() {
        let matcher = FuzzyMatcher::default();

        assert!(matcher.fuzzy_match("hello", "hlo").is_some());
        assert!(matcher.fuzzy_match("hello", "xyz").is_none());
    }

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(levenshtein_distance("hello", "hello"), 0);
        assert_eq!(levenshtein_distance("", "test"), 4);
    }

    #[test]
    fn test_similarity_score() {
        let score = similarity_score("hello", "hello");
        assert!((score - 1.0).abs() < 0.01);

        let score = similarity_score("hello", "hallo");
        assert!(score > 0.7);

        let score = similarity_score("hello", "world");
        assert!(score < 0.5);
    }

    #[test]
    fn test_starts_with_score() {
        assert_eq!(starts_with_score("hello world", "hello"), 1.0);
        assert_eq!(starts_with_score("hello world", "world"), 0.5);
        assert_eq!(starts_with_score("hello world", "xyz"), 0.0);
    }

    #[test]
    fn test_score_normalized() {
        let matcher = FuzzyMatcher::default();
        let score = matcher.score_normalized("hello", "hlo");
        assert!(score > 0.0 && score <= 1.0);
    }
}
