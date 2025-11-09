use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub server: ServerSettings,
    pub database: DatabaseSettings,
    pub security: SecuritySettings,
    pub performance: PerformanceSettings,
    pub logging: LoggingSettings,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerSettings {
    pub host: String,
    pub port: u16,
    pub workers: usize,
    pub keep_alive: u64,
    pub client_timeout: u64,
    pub enable_cors: bool,
    pub cors_origins: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseSettings {
    pub path: PathBuf,
    pub pool_size: u32,
    pub max_connections: u32,
    pub connection_timeout: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SecuritySettings {
    pub enable_auth: bool,
    pub jwt_secret: String,
    pub jwt_expiry: u64,
    pub api_key: Option<String>,
    pub rate_limit_per_minute: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PerformanceSettings {
    pub max_search_results: usize,
    pub search_timeout_ms: u64,
    pub index_batch_size: usize,
    pub cache_size: usize,
    pub enable_compression: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingSettings {
    pub level: String,
    pub format: String, // "json" or "pretty"
    pub file: Option<PathBuf>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            server: ServerSettings {
                host: "127.0.0.1".to_string(),
                port: 8080,
                workers: num_cpus::get(),
                keep_alive: 75,
                client_timeout: 30,
                enable_cors: true,
                cors_origins: vec!["http://localhost:*".to_string()],
            },
            database: DatabaseSettings {
                path: PathBuf::from("./filesearch.db"),
                pool_size: 10,
                max_connections: 100,
                connection_timeout: 30,
            },
            security: SecuritySettings {
                enable_auth: false,
                jwt_secret: uuid::Uuid::new_v4().to_string(),
                jwt_expiry: 3600,
                api_key: None,
                rate_limit_per_minute: 100,
            },
            performance: PerformanceSettings {
                max_search_results: 1000,
                search_timeout_ms: 5000,
                index_batch_size: 1000,
                cache_size: 10000,
                enable_compression: true,
            },
            logging: LoggingSettings {
                level: "info".to_string(),
                format: "pretty".to_string(),
                file: None,
            },
        }
    }
}

impl ServerConfig {
    pub fn from_file(path: &PathBuf) -> anyhow::Result<Self> {
        let settings = config::Config::builder()
            .add_source(config::File::from(path.as_ref()))
            .add_source(config::Environment::with_prefix("FILESEARCH"))
            .build()?;

        Ok(settings.try_deserialize()?)
    }

    pub fn load() -> anyhow::Result<Self> {
        // Try to load from default locations
        let config_paths = vec![
            PathBuf::from("config/default.toml"),
            PathBuf::from("config.toml"),
            PathBuf::from("./default.toml"),
        ];

        for path in config_paths {
            if path.exists() {
                return Self::from_file(&path);
            }
        }

        // If no config file found, use defaults
        Ok(Self::default())
    }
}
