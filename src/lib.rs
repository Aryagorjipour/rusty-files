pub mod core;
pub mod filters;
pub mod indexer;
pub mod search;
pub mod storage;
pub mod utils;
pub mod watcher;

pub use core::{
    DateFilter, ExclusionRule, ExclusionRuleType, FileEntry, IndexStats, MatchLocation, MatchMode,
    Progress, Result, SearchConfig, SearchConfigBuilder, SearchEngine, SearchError, SearchResult,
    SearchScope, SizeFilter,
};

pub use search::{Query, QueryParser};

pub use indexer::{UpdateStats, VerificationStats};

pub use filters::ExclusionFilter;

pub mod prelude {
    pub use crate::core::{Result, SearchConfig, SearchEngine};
    pub use crate::search::{Query, QueryParser};
}
