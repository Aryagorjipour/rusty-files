use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use chrono::{DateTime, Utc};

// ============ Search Models ============

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: String,

    #[serde(default)]
    pub mode: SearchMode,

    #[serde(default)]
    pub filters: SearchFilters,

    #[serde(default = "default_limit")]
    pub limit: usize,

    #[serde(default)]
    pub offset: usize,
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SearchMode {
    #[default]
    Exact,
    Fuzzy,
    Regex,
    Glob,
}

#[derive(Debug, Deserialize, Default)]
pub struct SearchFilters {
    pub extensions: Option<Vec<String>>,
    pub size_min: Option<u64>,
    pub size_max: Option<u64>,
    pub modified_after: Option<DateTime<Utc>>,
    pub modified_before: Option<DateTime<Utc>>,
    pub scope: Option<SearchScope>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SearchScope {
    Name,
    Path,
    Content,
    All,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub results: Vec<FileResult>,
    pub total: usize,
    pub took_ms: u64,
    pub has_more: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct FileResult {
    pub path: PathBuf,
    pub name: String,
    pub size: u64,
    pub modified: DateTime<Utc>,
    pub file_type: FileType,
    pub score: f32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_preview: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum FileType {
    File,
    Directory,
    Symlink,
}

// ============ Index Models ============

#[derive(Debug, Deserialize)]
pub struct IndexRequest {
    pub path: PathBuf,

    #[serde(default)]
    pub recursive: bool,

    #[serde(default)]
    pub follow_symlinks: bool,

    #[serde(default)]
    pub exclusions: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct IndexResponse {
    pub indexed_count: usize,
    pub skipped_count: usize,
    pub error_count: usize,
    pub took_ms: u64,
    pub status: IndexStatus,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum IndexStatus {
    Completed,
    Partial,
    Failed,
}

#[derive(Debug, Serialize)]
pub struct IndexProgress {
    pub current: usize,
    pub total: usize,
    pub current_path: PathBuf,
    pub percentage: f32,
}

// ============ Update Models ============

#[derive(Debug, Deserialize)]
pub struct UpdateRequest {
    pub path: PathBuf,
}

#[derive(Debug, Serialize)]
pub struct UpdateResponse {
    pub added: usize,
    pub updated: usize,
    pub removed: usize,
    pub took_ms: u64,
}

// ============ Watch Models ============

#[derive(Debug, Deserialize)]
pub struct WatchRequest {
    pub path: PathBuf,

    #[serde(default)]
    pub recursive: bool,
}

#[derive(Debug, Serialize)]
pub struct WatchResponse {
    pub watch_id: String,
    pub path: PathBuf,
    pub status: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct FileChangeEvent {
    pub event_type: FileEventType,
    pub path: PathBuf,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum FileEventType {
    Created,
    Modified,
    Deleted,
    Renamed,
}

// ============ Stats Models ============

#[derive(Debug, Serialize)]
pub struct StatsResponse {
    pub total_files: usize,
    pub total_directories: usize,
    pub total_size: u64,
    pub index_size_mb: f64,
    pub last_update: Option<DateTime<Utc>>,
    pub uptime_seconds: u64,
    pub performance: PerformanceStats,
}

#[derive(Debug, Serialize)]
pub struct PerformanceStats {
    pub total_searches: u64,
    pub avg_search_time_ms: f64,
    pub cache_hit_rate: f32,
    pub memory_usage_mb: f64,
}

// ============ Health Models ============

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: HealthStatus,
    pub version: String,
    pub uptime_seconds: u64,
    pub checks: Vec<HealthCheck>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Debug, Serialize)]
pub struct HealthCheck {
    pub name: String,
    pub status: HealthStatus,
    pub message: Option<String>,
    pub response_time_ms: Option<u64>,
}

// ============ Error Models ============

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub code: u16,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

// ============ Utilities ============

fn default_limit() -> usize {
    100
}
