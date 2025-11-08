pub const CURRENT_SCHEMA_VERSION: i32 = 1;

pub const CREATE_SCHEMA_VERSION_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL
)
"#;

pub const CREATE_FILES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS files (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    extension TEXT,
    size INTEGER NOT NULL,
    created_at INTEGER,
    modified_at INTEGER,
    accessed_at INTEGER,
    is_directory INTEGER NOT NULL DEFAULT 0,
    is_hidden INTEGER NOT NULL DEFAULT 0,
    is_symlink INTEGER NOT NULL DEFAULT 0,
    parent_path TEXT,
    mime_type TEXT,
    file_hash TEXT,
    indexed_at INTEGER NOT NULL,
    last_verified INTEGER NOT NULL
)
"#;

pub const CREATE_FILES_INDEXES: &[&str] = &[
    "CREATE INDEX IF NOT EXISTS idx_files_name ON files(name COLLATE NOCASE)",
    "CREATE INDEX IF NOT EXISTS idx_files_extension ON files(extension)",
    "CREATE INDEX IF NOT EXISTS idx_files_path ON files(path)",
    "CREATE INDEX IF NOT EXISTS idx_files_parent_path ON files(parent_path)",
    "CREATE INDEX IF NOT EXISTS idx_files_modified_at ON files(modified_at)",
    "CREATE INDEX IF NOT EXISTS idx_files_size ON files(size)",
    "CREATE INDEX IF NOT EXISTS idx_files_is_directory ON files(is_directory)",
    "CREATE INDEX IF NOT EXISTS idx_files_file_hash ON files(file_hash)",
];

pub const CREATE_FILES_FTS_TABLE: &str = r#"
CREATE VIRTUAL TABLE IF NOT EXISTS files_fts USING fts5(
    file_id,
    name,
    path,
    content,
    tokenize = 'porter unicode61'
)
"#;

pub const CREATE_FILE_CONTENTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS file_contents (
    file_id INTEGER PRIMARY KEY,
    content_preview TEXT,
    word_count INTEGER,
    line_count INTEGER,
    encoding TEXT,
    FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE
)
"#;

pub const CREATE_EXCLUSION_RULES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS exclusion_rules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pattern TEXT NOT NULL,
    rule_type TEXT NOT NULL,
    created_at INTEGER NOT NULL
)
"#;

pub const CREATE_INDEX_METADATA_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS index_metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at INTEGER NOT NULL
)
"#;

pub const CREATE_SEARCH_HISTORY_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS search_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    query TEXT NOT NULL,
    result_count INTEGER,
    searched_at INTEGER NOT NULL
)
"#;

pub const CREATE_ACCESS_LOG_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS access_log (
    file_id INTEGER NOT NULL,
    accessed_at INTEGER NOT NULL,
    FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE
)
"#;

pub const CREATE_ACCESS_LOG_INDEXES: &[&str] = &[
    "CREATE INDEX IF NOT EXISTS idx_access_log_file_id ON access_log(file_id)",
    "CREATE INDEX IF NOT EXISTS idx_access_log_accessed_at ON access_log(accessed_at)",
];

pub const OPTIMIZE_PRAGMAS: &[&str] = &[
    "PRAGMA journal_mode = WAL",
    "PRAGMA synchronous = NORMAL",
    "PRAGMA cache_size = -64000",
    "PRAGMA temp_store = MEMORY",
    "PRAGMA mmap_size = 268435456",
    "PRAGMA page_size = 4096",
];

pub fn get_all_table_creation_statements() -> Vec<&'static str> {
    vec![
        CREATE_SCHEMA_VERSION_TABLE,
        CREATE_FILES_TABLE,
        CREATE_FILE_CONTENTS_TABLE,
        CREATE_EXCLUSION_RULES_TABLE,
        CREATE_INDEX_METADATA_TABLE,
        CREATE_SEARCH_HISTORY_TABLE,
        CREATE_ACCESS_LOG_TABLE,
        CREATE_FILES_FTS_TABLE,
    ]
}

pub fn get_all_index_creation_statements() -> Vec<&'static str> {
    let mut indexes = Vec::new();
    indexes.extend_from_slice(CREATE_FILES_INDEXES);
    indexes.extend_from_slice(CREATE_ACCESS_LOG_INDEXES);
    indexes
}
