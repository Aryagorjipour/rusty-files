use crate::cli::output::OutputFormatter;
use crate::core::{Result, SearchEngine};
use crate::search::QueryParser;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

pub struct CommandExecutor {
    engine: Arc<Mutex<SearchEngine>>,
    formatter: OutputFormatter,
}

impl CommandExecutor {
    pub fn new(engine: SearchEngine, use_colors: bool, verbose: bool) -> Self {
        Self {
            engine: Arc::new(Mutex::new(engine)),
            formatter: OutputFormatter::new(use_colors, verbose),
        }
    }

    pub fn index(&self, path: PathBuf, show_progress: bool) -> Result<()> {
        let engine = self.engine.lock().unwrap();

        self.formatter.print_header(&format!(
            "Indexing directory: {}",
            path.display()
        ));

        let progress_bar = if show_progress {
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.green} [{elapsed_precise}] {msg}")
                    .unwrap(),
            );
            Some(pb)
        } else {
            None
        };

        let pb_clone = progress_bar.clone();
        let callback = move |progress: crate::core::types::Progress| {
            if let Some(ref pb) = pb_clone {
                pb.set_message(format!(
                    "{}/{} files ({}%)",
                    progress.current, progress.total, progress.percentage as u64
                ));
            }
        };

        let count = engine.index_directory(&path, Some(Box::new(callback)))?;

        if let Some(pb) = progress_bar {
            pb.finish_with_message("Indexing complete");
        }

        self.formatter.print_success(&format!(
            "Successfully indexed {} files",
            count
        ));

        Ok(())
    }

    pub fn update(&self, path: PathBuf, show_progress: bool) -> Result<()> {
        let engine = self.engine.lock().unwrap();

        self.formatter.print_header(&format!(
            "Updating index for: {}",
            path.display()
        ));

        let progress_bar = if show_progress {
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.green} [{elapsed_precise}] {msg}")
                    .unwrap(),
            );
            Some(pb)
        } else {
            None
        };

        let pb_clone = progress_bar.clone();
        let callback = move |progress: crate::core::types::Progress| {
            if let Some(ref pb) = pb_clone {
                pb.set_message(format!("{}", progress.message));
            }
        };

        let stats = engine.update_index(&path, Some(Box::new(callback)))?;

        if let Some(pb) = progress_bar {
            pb.finish_with_message("Update complete");
        }

        self.formatter.print_update_stats(&stats);
        self.formatter.print_success("Index updated successfully");

        Ok(())
    }

    pub fn search(&self, query: String) -> Result<()> {
        let engine = self.engine.lock().unwrap();

        let parsed_query = QueryParser::parse(&query)?;
        let results = engine.search_with_query(&parsed_query)?;

        self.formatter.print_search_results(&results, &query);

        Ok(())
    }

    pub fn stats(&self) -> Result<()> {
        let engine = self.engine.lock().unwrap();
        let stats = engine.get_stats()?;

        self.formatter.print_index_stats(&stats);

        Ok(())
    }

    pub fn verify(&self, path: PathBuf) -> Result<()> {
        let engine = self.engine.lock().unwrap();

        self.formatter.print_header(&format!(
            "Verifying index for: {}",
            path.display()
        ));

        let stats = engine.verify_index(&path)?;

        self.formatter.print_verification_stats(&stats);

        if stats.health_percentage() < 80.0 {
            self.formatter.print_warning(
                "Index health is below 80%. Consider running 'update' command.",
            );
        } else {
            self.formatter.print_success("Index is in good health");
        }

        Ok(())
    }

    pub fn watch(&self, path: PathBuf) -> Result<()> {
        let mut engine = self.engine.lock().unwrap();

        self.formatter.print_header(&format!(
            "Starting file system watch on: {}",
            path.display()
        ));

        engine.start_watching(&path)?;

        self.formatter.print_success("Watch started. Press Ctrl+C to stop.");

        std::thread::park();

        Ok(())
    }

    pub fn clear(&self, confirm: bool) -> Result<()> {
        if !confirm {
            self.formatter.print_warning(
                "This will delete all indexed data. Use --confirm to proceed.",
            );
            return Ok(());
        }

        let engine = self.engine.lock().unwrap();

        self.formatter.print_header("Clearing index...");

        engine.clear_index()?;

        self.formatter.print_success("Index cleared successfully");

        Ok(())
    }

    pub fn vacuum(&self) -> Result<()> {
        let engine = self.engine.lock().unwrap();

        self.formatter.print_header("Optimizing database...");

        engine.vacuum()?;

        self.formatter.print_success("Database optimized successfully");

        Ok(())
    }

    pub fn export(&self, output_path: PathBuf, query: Option<String>) -> Result<()> {
        let engine = self.engine.lock().unwrap();

        self.formatter.print_header(&format!(
            "Exporting results to: {}",
            output_path.display()
        ));

        let results = if let Some(q) = query {
            engine.search(&q)?
        } else {
            vec![]
        };

        let output_str = if output_path.extension().and_then(|s| s.to_str()) == Some("json") {
            serde_json::to_string_pretty(&results)
                .map_err(|e| crate::core::error::SearchError::Configuration(e.to_string()))?
        } else {
            results
                .iter()
                .map(|r| r.file.path.display().to_string())
                .collect::<Vec<_>>()
                .join("\n")
        };

        std::fs::write(&output_path, output_str)?;

        self.formatter.print_success(&format!(
            "Exported {} results",
            results.len()
        ));

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_index_command() {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path().join("data");
        fs::create_dir(&data_dir).unwrap();
        fs::write(data_dir.join("test.txt"), "content").unwrap();

        let index_path = temp_dir.path().join("index.db");
        let engine = SearchEngine::new(&index_path).unwrap();
        let executor = CommandExecutor::new(engine, false, false);

        let result = executor.index(data_dir, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_search_command() {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path().join("data");
        fs::create_dir(&data_dir).unwrap();
        fs::write(data_dir.join("test.txt"), "content").unwrap();

        let index_path = temp_dir.path().join("index.db");
        let engine = SearchEngine::new(&index_path).unwrap();
        let executor = CommandExecutor::new(engine, false, false);

        executor.index(data_dir, false).unwrap();

        let result = executor.search("test".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn test_stats_command() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = temp_dir.path().join("index.db");
        let engine = SearchEngine::new(&index_path).unwrap();
        let executor = CommandExecutor::new(engine, false, false);

        let result = executor.stats();
        assert!(result.is_ok());
    }
}
