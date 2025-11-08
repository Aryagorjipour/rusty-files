use rusty_files::prelude::*;
use std::path::PathBuf;

fn main() -> Result<()> {
    env_logger::init();

    let index_path = PathBuf::from("./examples_index.db");
    let engine = SearchEngine::new(&index_path)?;

    let current_dir = std::env::current_dir()?;
    println!("Indexing directory: {}", current_dir.display());

    let count = engine.index_directory(&current_dir, Some(Box::new(|progress| {
        if progress.current % 100 == 0 {
            println!(
                "Progress: {}/{} files ({:.1}%)",
                progress.current, progress.total, progress.percentage
            );
        }
    })))?;

    println!("\nSuccessfully indexed {} files\n", count);

    let query = "*.rs";
    println!("Searching for: {}", query);

    let results = engine.search(query)?;

    println!("Found {} results:\n", results.len());

    for (i, result) in results.iter().take(10).enumerate() {
        println!(
            "{}. {} ({})",
            i + 1,
            result.file.name,
            result.file.path.display()
        );
    }

    if results.len() > 10 {
        println!("\n... and {} more results", results.len() - 10);
    }

    let stats = engine.get_stats()?;
    println!("\nIndex Statistics:");
    println!("  Total files: {}", stats.total_files);
    println!("  Total directories: {}", stats.total_directories);
    println!("  Index size: {} bytes", stats.index_size);

    std::fs::remove_file(index_path).ok();

    Ok(())
}
