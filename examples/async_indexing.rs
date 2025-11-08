use rusty_files::prelude::*;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::task;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let index_path = PathBuf::from("./async_index.db");
    let engine = Arc::new(SearchEngine::new(&index_path)?);

    let current_dir = std::env::current_dir()?;
    println!("Starting async indexing for: {}\n", current_dir.display());

    let engine_clone = Arc::clone(&engine);
    let current_dir_clone = current_dir.clone();

    let indexing_task = task::spawn_blocking(move || {
        engine_clone.index_directory(&current_dir_clone, Some(Box::new(|progress| {
            if progress.current % 50 == 0 {
                println!(
                    "[Indexing] {}/{} files ({:.1}%)",
                    progress.current, progress.total, progress.percentage
                );
            }
        })))
    });

    println!("Indexing task started...\n");

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let count = indexing_task.await.unwrap()?;
    println!("\n✓ Indexing complete: {} files indexed\n", count);

    let queries = vec!["*.rs", "Cargo", "test"];

    for query in queries {
        let engine_clone = Arc::clone(&engine);
        let query_owned = query.to_string();

        let search_task = task::spawn_blocking(move || {
            (query_owned.clone(), engine_clone.search(&query_owned))
        });

        let (q, results) = search_task.await.unwrap();
        let results = results?;

        println!("Query '{}': {} results", q, results.len());
    }

    println!("\nCleaning up...");
    std::fs::remove_file(index_path).ok();

    Ok(())
}
