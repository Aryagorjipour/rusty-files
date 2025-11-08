use crate::core::types::{FileEntry, SizeFilter};

pub fn apply_size_filter(entry: &FileEntry, filter: &SizeFilter) -> bool {
    match filter {
        SizeFilter::Exact(size) => entry.size == *size,
        SizeFilter::Range(min, max) => entry.size >= *min && entry.size <= *max,
        SizeFilter::GreaterThan(size) => entry.size > *size,
        SizeFilter::LessThan(size) => entry.size < *size,
    }
}

pub fn parse_size(input: &str) -> Option<u64> {
    let input = input.trim().to_lowercase();

    let (number_str, multiplier) = if input.ends_with("kb") || input.ends_with("k") {
        (input.trim_end_matches("kb").trim_end_matches('k'), 1024u64)
    } else if input.ends_with("mb") || input.ends_with('m') {
        (input.trim_end_matches("mb").trim_end_matches('m'), 1024u64 * 1024)
    } else if input.ends_with("gb") || input.ends_with('g') {
        (input.trim_end_matches("gb").trim_end_matches('g'), 1024u64 * 1024 * 1024)
    } else if input.ends_with("tb") || input.ends_with('t') {
        (input.trim_end_matches("tb").trim_end_matches('t'), 1024u64 * 1024 * 1024 * 1024)
    } else if input.ends_with('b') {
        (input.trim_end_matches('b'), 1u64)
    } else {
        (input.as_str(), 1u64)
    };

    number_str.trim().parse::<u64>().ok().map(|n| n * multiplier)
}

pub fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if size >= TB {
        format!("{:.2} TB", size as f64 / TB as f64)
    } else if size >= GB {
        format!("{:.2} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.2} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.2} KB", size as f64 / KB as f64)
    } else {
        format!("{} B", size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_parse_size() {
        assert_eq!(parse_size("100"), Some(100));
        assert_eq!(parse_size("1KB"), Some(1024));
        assert_eq!(parse_size("1MB"), Some(1024 * 1024));
        assert_eq!(parse_size("1GB"), Some(1024 * 1024 * 1024));
        assert_eq!(parse_size("1.5MB"), None);
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(100), "100 B");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_apply_size_filter() {
        let entry = FileEntry {
            id: None,
            path: PathBuf::from("/test/file.txt"),
            name: "file.txt".to_string(),
            extension: Some("txt".to_string()),
            size: 1024,
            created_at: None,
            modified_at: None,
            accessed_at: None,
            is_directory: false,
            is_hidden: false,
            is_symlink: false,
            parent_path: None,
            mime_type: None,
            file_hash: None,
            indexed_at: chrono::Utc::now(),
            last_verified: chrono::Utc::now(),
        };

        assert!(apply_size_filter(&entry, &SizeFilter::Exact(1024)));
        assert!(apply_size_filter(&entry, &SizeFilter::GreaterThan(1000)));
        assert!(apply_size_filter(&entry, &SizeFilter::LessThan(2000)));
        assert!(apply_size_filter(&entry, &SizeFilter::Range(1000, 2000)));
    }
}
