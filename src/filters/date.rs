use crate::core::types::{DateFilter, FileEntry};
use chrono::{DateTime, Duration, Utc};

pub fn apply_date_filter(entry: &FileEntry, filter: &DateFilter) -> bool {
    let modified = entry.modified_at.unwrap_or_else(Utc::now);

    match filter {
        DateFilter::After(date) => modified > *date,
        DateFilter::Before(date) => modified < *date,
        DateFilter::Between(start, end) => modified >= *start && modified <= *end,
        DateFilter::On(date) => {
            let start_of_day = date.date_naive().and_hms_opt(0, 0, 0).unwrap();
            let end_of_day = date.date_naive().and_hms_opt(23, 59, 59).unwrap();

            let start = DateTime::<Utc>::from_naive_utc_and_offset(start_of_day, Utc);
            let end = DateTime::<Utc>::from_naive_utc_and_offset(end_of_day, Utc);

            modified >= start && modified <= end
        }
    }
}

pub fn parse_relative_date(input: &str) -> Option<DateTime<Utc>> {
    let input = input.trim().to_lowercase();
    let now = Utc::now();

    if input == "today" {
        Some(now - Duration::days(0))
    } else if input == "yesterday" {
        Some(now - Duration::days(1))
    } else if input == "week" || input == "this week" {
        Some(now - Duration::weeks(1))
    } else if input == "month" || input == "this month" {
        Some(now - Duration::days(30))
    } else if input == "year" || input == "this year" {
        Some(now - Duration::days(365))
    } else if input.ends_with("days") || input.ends_with('d') {
        let num_str = input.trim_end_matches("days").trim_end_matches('d').trim();
        num_str.parse::<i64>().ok().map(|n| now - Duration::days(n))
    } else if input.ends_with("weeks") || input.ends_with('w') {
        let num_str = input.trim_end_matches("weeks").trim_end_matches('w').trim();
        num_str.parse::<i64>().ok().map(|n| now - Duration::weeks(n))
    } else if input.ends_with("months") {
        let num_str = input.trim_end_matches("months").trim();
        num_str.parse::<i64>().ok().map(|n| now - Duration::days(n * 30))
    } else if input.ends_with("years") {
        let num_str = input.trim_end_matches("years").trim();
        num_str.parse::<i64>().ok().map(|n| now - Duration::days(n * 365))
    } else {
        None
    }
}

pub fn format_date(date: DateTime<Utc>) -> String {
    date.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

pub fn format_relative_date(date: DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(date);

    if duration.num_seconds() < 60 {
        "just now".to_string()
    } else if duration.num_minutes() < 60 {
        let mins = duration.num_minutes();
        if mins == 1 {
            "1 minute ago".to_string()
        } else {
            format!("{} minutes ago", mins)
        }
    } else if duration.num_hours() < 24 {
        let hours = duration.num_hours();
        if hours == 1 {
            "1 hour ago".to_string()
        } else {
            format!("{} hours ago", hours)
        }
    } else if duration.num_days() < 7 {
        let days = duration.num_days();
        if days == 1 {
            "1 day ago".to_string()
        } else {
            format!("{} days ago", days)
        }
    } else if duration.num_weeks() < 4 {
        let weeks = duration.num_weeks();
        if weeks == 1 {
            "1 week ago".to_string()
        } else {
            format!("{} weeks ago", weeks)
        }
    } else {
        format_date(date)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_relative_date() {
        assert!(parse_relative_date("today").is_some());
        assert!(parse_relative_date("yesterday").is_some());
        assert!(parse_relative_date("week").is_some());
        assert!(parse_relative_date("7days").is_some());
        assert!(parse_relative_date("2weeks").is_some());
    }

    #[test]
    fn test_format_relative_date() {
        let now = Utc::now();
        let one_hour_ago = now - Duration::hours(1);
        let formatted = format_relative_date(one_hour_ago);
        assert!(formatted.contains("hour"));
    }
}
