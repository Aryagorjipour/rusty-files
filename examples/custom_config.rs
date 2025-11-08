use rusty_files::prelude::*;
use rusty_files::SearchEngine;
use std::path::PathBuf;

fn main() -> Result<()> {
    env_logger::init();

    let index_path = PathBuf::from("./custom_index.db");

    let engine = SearchEngine::builder()
        .index_path(&index_path)
        .thread_count(4)
        .enable_content_search(false)
        .enable_fuzzy_search(true)
        .cache_size(500)
        .max_search_results(50)
        .exclusion_patterns(vec![
            ".git".to_string(),
            "node_modules".to_string(),
            "target".to_string(),
            "*.log".to_string(),
        ])
        .build()?;

    println!("Search Engine Configuration:");
    println!("  Thread count: {}", engine.get_config().thread_count);
    println!(
        "  Content search: {}",
        engine.get_config().enable_content_search
    );
    println!(
        "  Fuzzy search: {}",
        engine.get_config().enable_fuzzy_search
    );
    println!("  Cache size: {}", engine.get_config().cache_size);
    println!("  Exclusion patterns: {:?}\n", engine.get_config().exclusion_patterns);

    let current_dir = std::env::current_dir()?;
    println!("Indexing: {}\n", current_dir.display());

    let count = engine.index_directory(&current_dir, None)?;
    println!("Indexed {} files\n", count);

    let queries = vec![
        "Cargo mode:exact",
        "*.rs ext:rs",
        "main mode:fuzzy",
    ];

    for query in queries {
        println!("Query: {}", query);
        let results = engine.search(query)?;
        println!("  Results: {}\n", results.len());
    }

    std::fs::remove_file(index_path).ok();

    Ok(())
}
