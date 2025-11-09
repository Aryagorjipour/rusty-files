use actix_web::{web, HttpResponse, Result};
use std::time::Instant;
use std::sync::atomic::Ordering;
use tracing::{info, error};
use chrono::Utc;

use crate::{Query, MatchMode, SearchScope, SizeFilter};
use crate::server::models::*;
use crate::server::state::AppState;

// ============ Search Endpoint ============

pub async fn search(
    state: web::Data<AppState>,
    req: web::Json<SearchRequest>,
) -> Result<HttpResponse> {
    let start = Instant::now();

    info!("Search request: {:?}", req.query);

    // Build query from request
    let query = build_query(&req)?;

    // Execute search
    let engine = state.engine.read();
    let results = engine
        .search_with_query(&query)
        .map_err(|e| {
            error!("Search failed: {}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?;

    let took_ms = start.elapsed().as_millis() as u64;

    // Record metrics
    state.metrics.record_search(took_ms);

    // Convert to API response
    let total = results.len();
    let has_more = total > req.limit;
    let results: Vec<FileResult> = results
        .into_iter()
        .skip(req.offset)
        .take(req.limit)
        .map(convert_result)
        .collect();

    Ok(HttpResponse::Ok().json(SearchResponse {
        results,
        total,
        took_ms,
        has_more,
    }))
}

// ============ Index Endpoint ============

pub async fn index(
    state: web::Data<AppState>,
    req: web::Json<IndexRequest>,
) -> Result<HttpResponse> {
    let start = Instant::now();

    info!("Index request: {:?}", req.path);

    // Validate path
    if !req.path.exists() {
        return Ok(HttpResponse::BadRequest().json(ErrorResponse {
            error: "invalid_path".to_string(),
            message: "Path does not exist".to_string(),
            code: 400,
            details: None,
        }));
    }

    let engine = state.engine.read();

    let count = engine
        .index_directory(&req.path, None)
        .map_err(|e| {
            error!("Indexing failed: {}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?;

    let took_ms = start.elapsed().as_millis() as u64;

    Ok(HttpResponse::Ok().json(IndexResponse {
        indexed_count: count,
        skipped_count: 0,
        error_count: 0,
        took_ms,
        status: IndexStatus::Completed,
    }))
}

// ============ Update Endpoint ============

pub async fn update(
    state: web::Data<AppState>,
    req: web::Json<UpdateRequest>,
) -> Result<HttpResponse> {
    let start = Instant::now();

    info!("Update request: {:?}", req.path);

    let engine = state.engine.read();

    let stats = engine
        .update_index(&req.path, None)
        .map_err(|e| {
            error!("Update failed: {}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?;

    let took_ms = start.elapsed().as_millis() as u64;

    Ok(HttpResponse::Ok().json(UpdateResponse {
        added: stats.added,
        updated: stats.updated,
        removed: stats.removed,
        took_ms,
    }))
}

// ============ Watch Endpoint ============

pub async fn start_watch(
    state: web::Data<AppState>,
    req: web::Json<WatchRequest>,
) -> Result<HttpResponse> {
    info!("Watch request: {:?}", req.path);

    let watch_id = uuid::Uuid::new_v4().to_string();

    // Start watching
    let mut engine = state.engine.write();
    engine
        .start_watching(&req.path)
        .map_err(|e| {
            error!("Watch failed: {}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?;

    // Store watch handle
    use crate::server::state::WatchHandle;
    state.watchers.insert(
        watch_id.clone(),
        WatchHandle {
            path: req.path.clone(),
            recursive: req.recursive,
            created_at: Utc::now(),
        },
    );

    Ok(HttpResponse::Ok().json(WatchResponse {
        watch_id,
        path: req.path.clone(),
        status: "active".to_string(),
    }))
}

pub async fn stop_watch(
    state: web::Data<AppState>,
    watch_id: web::Path<String>,
) -> Result<HttpResponse> {
    info!("Stop watch request: {}", watch_id);

    if let Some((_, handle)) = state.watchers.remove(watch_id.as_str()) {
        let mut engine = state.engine.write();
        engine
            .stop_watching()
            .map_err(|e| {
                error!("Stop watch failed: {}", e);
                actix_web::error::ErrorInternalServerError(e)
            })?;

        Ok(HttpResponse::Ok().json(serde_json::json!({
            "message": "Watch stopped",
            "path": handle.path
        })))
    } else {
        Ok(HttpResponse::NotFound().json(ErrorResponse {
            error: "not_found".to_string(),
            message: "Watch ID not found".to_string(),
            code: 404,
            details: None,
        }))
    }
}

// ============ Stats Endpoint ============

pub async fn get_stats(state: web::Data<AppState>) -> Result<HttpResponse> {
    let engine = state.engine.read();
    let db_stats = engine.get_stats().map_err(|e| {
        error!("Failed to get stats: {}", e);
        actix_web::error::ErrorInternalServerError(e)
    })?;

    Ok(HttpResponse::Ok().json(StatsResponse {
        total_files: db_stats.total_files,
        total_directories: db_stats.total_directories,
        total_size: db_stats.total_size,
        index_size_mb: db_stats.index_size as f64 / 1_000_000.0,
        last_update: Some(db_stats.last_update),
        uptime_seconds: state.uptime_seconds(),
        performance: PerformanceStats {
            total_searches: state.metrics.total_searches.load(Ordering::Relaxed),
            avg_search_time_ms: state.metrics.avg_search_time_ms(),
            cache_hit_rate: state.metrics.cache_hit_rate(),
            memory_usage_mb: get_memory_usage_mb(),
        },
    }))
}

// ============ Health Endpoint ============

pub async fn health_check(state: web::Data<AppState>) -> Result<HttpResponse> {
    let mut checks = Vec::new();

    // Database check
    let db_check_start = Instant::now();
    let engine = state.engine.read();
    let db_healthy = engine.get_stats().is_ok();
    checks.push(HealthCheck {
        name: "database".to_string(),
        status: if db_healthy {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unhealthy
        },
        message: None,
        response_time_ms: Some(db_check_start.elapsed().as_millis() as u64),
    });

    // Memory check
    let memory_mb = get_memory_usage_mb();
    let memory_healthy = memory_mb < 1000.0; // Less than 1GB
    checks.push(HealthCheck {
        name: "memory".to_string(),
        status: if memory_healthy {
            HealthStatus::Healthy
        } else {
            HealthStatus::Degraded
        },
        message: Some(format!("{:.2} MB", memory_mb)),
        response_time_ms: None,
    });

    let overall_status = if checks
        .iter()
        .all(|c| matches!(c.status, HealthStatus::Healthy))
    {
        HealthStatus::Healthy
    } else if checks
        .iter()
        .any(|c| matches!(c.status, HealthStatus::Unhealthy))
    {
        HealthStatus::Unhealthy
    } else {
        HealthStatus::Degraded
    };

    Ok(HttpResponse::Ok().json(HealthResponse {
        status: overall_status,
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: state.uptime_seconds(),
        checks,
    }))
}

// ============ Helper Functions ============

fn build_query(req: &SearchRequest) -> Result<Query> {
    let mut query = Query::new(req.query.clone());

    // Set match mode
    query = match req.mode {
        SearchMode::Exact => query.with_match_mode(MatchMode::Exact),
        SearchMode::Fuzzy => query.with_match_mode(MatchMode::Fuzzy),
        SearchMode::Regex => query.with_match_mode(MatchMode::Regex),
        SearchMode::Glob => query.with_match_mode(MatchMode::Glob),
    };

    // Apply filters
    if let Some(ref extensions) = req.filters.extensions {
        query = query.with_extensions(extensions.clone());
    }

    if let Some(size_min) = req.filters.size_min {
        query = query.with_size_filter(SizeFilter::GreaterThan(size_min));
    }

    if let Some(ref scope) = req.filters.scope {
        query = query.with_scope(match scope {
            crate::server::models::SearchScope::Name => SearchScope::Name,
            crate::server::models::SearchScope::Path => SearchScope::Path,
            crate::server::models::SearchScope::Content => SearchScope::Content,
            crate::server::models::SearchScope::All => SearchScope::All,
        });
    }

    // Set limit
    query = query.with_max_results(req.limit);

    Ok(query)
}

fn convert_result(result: crate::SearchResult) -> FileResult {
    FileResult {
        path: result.file.path.clone(),
        name: result.file.name.clone(),
        size: result.file.size,
        modified: result.file.modified_at.unwrap_or_else(|| Utc::now()),
        file_type: if result.file.is_directory {
            FileType::Directory
        } else if result.file.is_symlink {
            FileType::Symlink
        } else {
            FileType::File
        },
        score: result.score as f32,
        content_preview: result.snippet,
    }
}

fn get_memory_usage_mb() -> f64 {
    #[cfg(target_os = "linux")]
    {
        // Read from /proc/self/status
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    if let Some(kb_str) = line.split_whitespace().nth(1) {
                        if let Ok(kb) = kb_str.parse::<f64>() {
                            return kb / 1024.0; // Convert to MB
                        }
                    }
                }
            }
        }
    }

    0.0 // Fallback
}
