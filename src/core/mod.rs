pub mod config;
pub mod engine;
pub mod error;
pub mod types;

pub use config::{SearchConfig, SearchConfigBuilder};
pub use engine::SearchEngine;
pub use error::{Result, SearchError};
pub use types::*;
