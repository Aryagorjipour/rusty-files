use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub id: Option<i64>,
    pub path: PathBuf,
    pub name: String,
    pub extension: Option<String>,
    pub size: u64,
    pub created_at: Option<DateTime<Utc>>,
    pub modified_at: Option<DateTime<Utc>>,
    pub accessed_at: Option<DateTime<Utc>>,
    pub is_directory: bool,
    pub is_hidden: bool,
    pub is_symlink: bool,
    pub parent_path: Option<PathBuf>,
    pub mime_type: Option<String>,
    pub file_hash: Option<String>,
    pub indexed_at: DateTime<Utc>,
    pub last_verified: DateTime<Utc>,
}

impl FileEntry {
    pub fn new(path: PathBuf) -> Self {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_string());

        let parent_path = path.parent().map(|p| p.to_path_buf());

        let now = Utc::now();

        Self {
            id: None,
            path,
            name,
            extension,
            size: 0,
            created_at: None,
            modified_at: None,
            accessed_at: None,
            is_directory: false,
            is_hidden: false,
            is_symlink: false,
            parent_path,
            mime_type: None,
            file_hash: None,
            indexed_at: now,
            last_verified: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub file: FileEntry,
    pub score: f64,
    pub snippet: Option<String>,
    pub matches: Vec<MatchLocation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchLocation {
    pub line: usize,
    pub column: usize,
    pub length: usize,
    pub context: String,
}

#[derive(Debug, Clone)]
pub struct Progress {
    pub current: usize,
    pub total: usize,
    pub message: String,
    pub percentage: f64,
}

impl Progress {
    pub fn new(current: usize, total: usize, message: String) -> Self {
        let percentage = if total > 0 {
            (current as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        Self {
            current,
            total,
            message,
            percentage,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchMode {
    Exact,
    CaseInsensitive,
    Fuzzy,
    Regex,
    Glob,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchScope {
    Name,
    Path,
    Content,
    All,
}

#[derive(Debug, Clone)]
pub enum SizeFilter {
    Exact(u64),
    Range(u64, u64),
    GreaterThan(u64),
    LessThan(u64),
}

#[derive(Debug, Clone)]
pub enum DateFilter {
    After(DateTime<Utc>),
    Before(DateTime<Utc>),
    Between(DateTime<Utc>, DateTime<Utc>),
    On(DateTime<Utc>),
}

#[derive(Debug, Clone)]
pub struct IndexStats {
    pub total_files: usize,
    pub total_directories: usize,
    pub total_size: u64,
    pub indexed_files: usize,
    pub last_update: DateTime<Utc>,
    pub index_size: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExclusionRuleType {
    Glob,
    Regex,
    Path,
}

#[derive(Debug, Clone)]
pub struct ExclusionRule {
    pub pattern: String,
    pub rule_type: ExclusionRuleType,
}

#[derive(Debug, Clone)]
pub struct ContentPreview {
    pub preview: String,
    pub word_count: usize,
    pub line_count: usize,
    pub encoding: String,
}

pub type ProgressCallback = Box<dyn Fn(Progress) + Send + Sync>;
