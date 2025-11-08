use crate::core::error::{Result, SearchError};
use crate::core::types::{ContentPreview, ExclusionRule, ExclusionRuleType, FileEntry, IndexStats};
use crate::storage::migrations::MigrationManager;
use chrono::{DateTime, TimeZone, Utc};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::{Path, PathBuf};

pub type DbPool = Pool<SqliteConnectionManager>;

pub struct Database {
    pool: DbPool,
}

impl Database {
    pub fn new<P: AsRef<Path>>(path: P, pool_size: u32) -> Result<Self> {
        let manager = SqliteConnectionManager::file(path.as_ref());
        let pool = Pool::builder()
            .max_size(pool_size)
            .build(manager)?;

        {
            let conn = pool.get()?;
            MigrationManager::initialize_schema(&conn)?;
        }

        Ok(Self { pool })
    }

    pub fn in_memory(pool_size: u32) -> Result<Self> {
        let manager = SqliteConnectionManager::memory();
        let pool = Pool::builder()
            .max_size(pool_size)
            .build(manager)?;

        {
            let conn = pool.get()?;
            MigrationManager::initialize_schema(&conn)?;
        }

        Ok(Self { pool })
    }

    pub fn insert_file(&self, file: &FileEntry) -> Result<i64> {
        let conn = self.pool.get()?;

        let created_at = file.created_at.map(|dt| dt.timestamp());
        let modified_at = file.modified_at.map(|dt| dt.timestamp());
        let accessed_at = file.accessed_at.map(|dt| dt.timestamp());
        let indexed_at = file.indexed_at.timestamp();
        let last_verified = file.last_verified.timestamp();

        conn.execute(
            r#"
            INSERT INTO files (
                path, name, extension, size, created_at, modified_at, accessed_at,
                is_directory, is_hidden, is_symlink, parent_path, mime_type, file_hash,
                indexed_at, last_verified
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
            ON CONFLICT(path) DO UPDATE SET
                name = excluded.name,
                extension = excluded.extension,
                size = excluded.size,
                modified_at = excluded.modified_at,
                accessed_at = excluded.accessed_at,
                is_directory = excluded.is_directory,
                is_hidden = excluded.is_hidden,
                is_symlink = excluded.is_symlink,
                mime_type = excluded.mime_type,
                file_hash = excluded.file_hash,
                last_verified = excluded.last_verified
            "#,
            params![
                file.path.to_string_lossy().to_string(),
                file.name,
                file.extension,
                file.size as i64,
                created_at,
                modified_at,
                accessed_at,
                file.is_directory as i32,
                file.is_hidden as i32,
                file.is_symlink as i32,
                file.parent_path.as_ref().map(|p| p.to_string_lossy().to_string()),
                file.mime_type,
                file.file_hash,
                indexed_at,
                last_verified,
            ],
        )?;

        Ok(conn.last_insert_rowid())
    }

    pub fn insert_files_batch(&self, files: &[FileEntry]) -> Result<()> {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;

        for file in files {
            let created_at = file.created_at.map(|dt| dt.timestamp());
            let modified_at = file.modified_at.map(|dt| dt.timestamp());
            let accessed_at = file.accessed_at.map(|dt| dt.timestamp());
            let indexed_at = file.indexed_at.timestamp();
            let last_verified = file.last_verified.timestamp();

            tx.execute(
                r#"
                INSERT INTO files (
                    path, name, extension, size, created_at, modified_at, accessed_at,
                    is_directory, is_hidden, is_symlink, parent_path, mime_type, file_hash,
                    indexed_at, last_verified
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
                ON CONFLICT(path) DO UPDATE SET
                    name = excluded.name,
                    extension = excluded.extension,
                    size = excluded.size,
                    modified_at = excluded.modified_at,
                    accessed_at = excluded.accessed_at,
                    is_directory = excluded.is_directory,
                    is_hidden = excluded.is_hidden,
                    is_symlink = excluded.is_symlink,
                    mime_type = excluded.mime_type,
                    file_hash = excluded.file_hash,
                    last_verified = excluded.last_verified
                "#,
                params![
                    file.path.to_string_lossy().to_string(),
                    file.name,
                    file.extension,
                    file.size as i64,
                    created_at,
                    modified_at,
                    accessed_at,
                    file.is_directory as i32,
                    file.is_hidden as i32,
                    file.is_symlink as i32,
                    file.parent_path.as_ref().map(|p| p.to_string_lossy().to_string()),
                    file.mime_type,
                    file.file_hash,
                    indexed_at,
                    last_verified,
                ],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    pub fn find_by_path(&self, path: &Path) -> Result<Option<FileEntry>> {
        let conn = self.pool.get()?;

        let result = conn
            .query_row(
                r#"
                SELECT id, path, name, extension, size, created_at, modified_at, accessed_at,
                       is_directory, is_hidden, is_symlink, parent_path, mime_type, file_hash,
                       indexed_at, last_verified
                FROM files WHERE path = ?1
                "#,
                params![path.to_string_lossy().to_string()],
                |row| Self::row_to_file_entry(row),
            )
            .optional()?;

        Ok(result)
    }

    pub fn find_by_id(&self, id: i64) -> Result<Option<FileEntry>> {
        let conn = self.pool.get()?;

        let result = conn
            .query_row(
                r#"
                SELECT id, path, name, extension, size, created_at, modified_at, accessed_at,
                       is_directory, is_hidden, is_symlink, parent_path, mime_type, file_hash,
                       indexed_at, last_verified
                FROM files WHERE id = ?1
                "#,
                params![id],
                |row| Self::row_to_file_entry(row),
            )
            .optional()?;

        Ok(result)
    }

    pub fn delete_by_path(&self, path: &Path) -> Result<()> {
        let conn = self.pool.get()?;
        conn.execute(
            "DELETE FROM files WHERE path = ?1",
            params![path.to_string_lossy().to_string()],
        )?;
        Ok(())
    }

    pub fn search_by_name(&self, pattern: &str, limit: usize) -> Result<Vec<FileEntry>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT id, path, name, extension, size, created_at, modified_at, accessed_at,
                   is_directory, is_hidden, is_symlink, parent_path, mime_type, file_hash,
                   indexed_at, last_verified
            FROM files WHERE name LIKE ?1 LIMIT ?2
            "#,
        )?;

        let files = stmt
            .query_map(params![format!("%{}%", pattern), limit], |row| {
                Self::row_to_file_entry(row)
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(files)
    }

    pub fn search_by_extension(&self, extension: &str, limit: usize) -> Result<Vec<FileEntry>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT id, path, name, extension, size, created_at, modified_at, accessed_at,
                   is_directory, is_hidden, is_symlink, parent_path, mime_type, file_hash,
                   indexed_at, last_verified
            FROM files WHERE extension = ?1 LIMIT ?2
            "#,
        )?;

        let files = stmt
            .query_map(params![extension, limit], |row| {
                Self::row_to_file_entry(row)
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(files)
    }

    pub fn get_all_files(&self, limit: usize, offset: usize) -> Result<Vec<FileEntry>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT id, path, name, extension, size, created_at, modified_at, accessed_at,
                   is_directory, is_hidden, is_symlink, parent_path, mime_type, file_hash,
                   indexed_at, last_verified
            FROM files LIMIT ?1 OFFSET ?2
            "#,
        )?;

        let files = stmt
            .query_map(params![limit, offset], |row| Self::row_to_file_entry(row))?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(files)
    }

    pub fn insert_content(&self, file_id: i64, preview: &ContentPreview) -> Result<()> {
        let conn = self.pool.get()?;

        conn.execute(
            r#"
            INSERT INTO file_contents (file_id, content_preview, word_count, line_count, encoding)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(file_id) DO UPDATE SET
                content_preview = excluded.content_preview,
                word_count = excluded.word_count,
                line_count = excluded.line_count,
                encoding = excluded.encoding
            "#,
            params![
                file_id,
                preview.preview,
                preview.word_count as i64,
                preview.line_count as i64,
                preview.encoding
            ],
        )?;

        Ok(())
    }

    pub fn insert_fts_entry(&self, file_id: i64, name: &str, path: &str, content: &str) -> Result<()> {
        let conn = self.pool.get()?;

        conn.execute(
            "INSERT INTO files_fts (file_id, name, path, content) VALUES (?1, ?2, ?3, ?4)",
            params![file_id, name, path, content],
        )?;

        Ok(())
    }

    pub fn search_content(&self, query: &str, limit: usize) -> Result<Vec<i64>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT file_id FROM files_fts WHERE files_fts MATCH ?1 LIMIT ?2"
        )?;

        let file_ids = stmt
            .query_map(params![query, limit], |row| row.get(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(file_ids)
    }

    pub fn add_exclusion_rule(&self, rule: &ExclusionRule) -> Result<i64> {
        let conn = self.pool.get()?;

        let rule_type = match rule.rule_type {
            ExclusionRuleType::Glob => "glob",
            ExclusionRuleType::Regex => "regex",
            ExclusionRuleType::Path => "path",
        };

        conn.execute(
            "INSERT INTO exclusion_rules (pattern, rule_type, created_at) VALUES (?1, ?2, ?3)",
            params![rule.pattern, rule_type, Utc::now().timestamp()],
        )?;

        Ok(conn.last_insert_rowid())
    }

    pub fn get_exclusion_rules(&self) -> Result<Vec<ExclusionRule>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT pattern, rule_type FROM exclusion_rules")?;

        let rules = stmt
            .query_map([], |row| {
                let pattern: String = row.get(0)?;
                let rule_type_str: String = row.get(1)?;
                let rule_type = match rule_type_str.as_str() {
                    "glob" => ExclusionRuleType::Glob,
                    "regex" => ExclusionRuleType::Regex,
                    "path" => ExclusionRuleType::Path,
                    _ => ExclusionRuleType::Glob,
                };

                Ok(ExclusionRule { pattern, rule_type })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(rules)
    }

    pub fn log_access(&self, file_id: i64) -> Result<()> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT INTO access_log (file_id, accessed_at) VALUES (?1, ?2)",
            params![file_id, Utc::now().timestamp()],
        )?;
        Ok(())
    }

    pub fn get_stats(&self) -> Result<IndexStats> {
        let conn = self.pool.get()?;

        let total_files: i64 = conn.query_row(
            "SELECT COUNT(*) FROM files WHERE is_directory = 0",
            [],
            |row| row.get(0),
        )?;

        let total_directories: i64 = conn.query_row(
            "SELECT COUNT(*) FROM files WHERE is_directory = 1",
            [],
            |row| row.get(0),
        )?;

        let total_size: i64 = conn.query_row(
            "SELECT COALESCE(SUM(size), 0) FROM files WHERE is_directory = 0",
            [],
            |row| row.get(0),
        )?;

        let indexed_files: i64 = conn.query_row(
            "SELECT COUNT(*) FROM file_contents",
            [],
            |row| row.get(0),
        )?;

        let last_update_ts: Option<i64> = conn
            .query_row(
                "SELECT MAX(indexed_at) FROM files",
                [],
                |row| row.get(0),
            )
            .optional()?
            .flatten();

        let last_update = last_update_ts
            .and_then(|ts| Utc.timestamp_opt(ts, 0).single())
            .unwrap_or_else(Utc::now);

        let index_size = std::fs::metadata(
            conn.path().ok_or_else(|| {
                SearchError::Configuration("Cannot get database path".to_string())
            })?,
        )
        .map(|m| m.len())
        .unwrap_or(0);

        Ok(IndexStats {
            total_files: total_files as usize,
            total_directories: total_directories as usize,
            total_size: total_size as u64,
            indexed_files: indexed_files as usize,
            last_update,
            index_size,
        })
    }

    pub fn clear_all(&self) -> Result<()> {
        let conn = self.pool.get()?;
        let tx = conn.unchecked_transaction()?;

        tx.execute("DELETE FROM files", [])?;
        tx.execute("DELETE FROM file_contents", [])?;
        tx.execute("DELETE FROM files_fts", [])?;
        tx.execute("DELETE FROM access_log", [])?;
        tx.execute("DELETE FROM search_history", [])?;

        tx.commit()?;
        Ok(())
    }

    pub fn vacuum(&self) -> Result<()> {
        let conn = self.pool.get()?;
        conn.execute("VACUUM", [])?;
        Ok(())
    }

    fn row_to_file_entry(row: &rusqlite::Row) -> rusqlite::Result<FileEntry> {
        let id: i64 = row.get(0)?;
        let path: String = row.get(1)?;
        let name: String = row.get(2)?;
        let extension: Option<String> = row.get(3)?;
        let size: i64 = row.get(4)?;
        let created_at: Option<i64> = row.get(5)?;
        let modified_at: Option<i64> = row.get(6)?;
        let accessed_at: Option<i64> = row.get(7)?;
        let is_directory: i32 = row.get(8)?;
        let is_hidden: i32 = row.get(9)?;
        let is_symlink: i32 = row.get(10)?;
        let parent_path: Option<String> = row.get(11)?;
        let mime_type: Option<String> = row.get(12)?;
        let file_hash: Option<String> = row.get(13)?;
        let indexed_at: i64 = row.get(14)?;
        let last_verified: i64 = row.get(15)?;

        Ok(FileEntry {
            id: Some(id),
            path: PathBuf::from(path),
            name,
            extension,
            size: size as u64,
            created_at: created_at.and_then(|ts| Utc.timestamp_opt(ts, 0).single()),
            modified_at: modified_at.and_then(|ts| Utc.timestamp_opt(ts, 0).single()),
            accessed_at: accessed_at.and_then(|ts| Utc.timestamp_opt(ts, 0).single()),
            is_directory: is_directory != 0,
            is_hidden: is_hidden != 0,
            is_symlink: is_symlink != 0,
            parent_path: parent_path.map(PathBuf::from),
            mime_type,
            file_hash,
            indexed_at: Utc.timestamp_opt(indexed_at, 0).single().unwrap_or_else(Utc::now),
            last_verified: Utc.timestamp_opt(last_verified, 0).single().unwrap_or_else(Utc::now),
        })
    }
}
