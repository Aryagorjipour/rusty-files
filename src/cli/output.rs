use rusty_files::core::types::{IndexStats, SearchResult};
use rusty_files::filters::{format_date, format_relative_date, format_size};
use rusty_files::indexer::{UpdateStats, VerificationStats};
use colored::*;

pub struct OutputFormatter {
    use_colors: bool,
    verbose: bool,
}

impl OutputFormatter {
    pub fn new(use_colors: bool, verbose: bool) -> Self {
        Self { use_colors, verbose }
    }

    pub fn print_search_results(&self, results: &[SearchResult], query: &str) {
        if results.is_empty() {
            self.print_info(&format!("No results found for query: {}", query));
            return;
        }

        self.print_header(&format!("Found {} results for: {}", results.len(), query));
        println!();

        for (idx, result) in results.iter().enumerate() {
            self.print_search_result(idx + 1, result);
        }

        println!();
        self.print_summary(&format!("Total: {} results", results.len()));
    }

    pub fn print_search_result(&self, index: usize, result: &SearchResult) {
        let file = &result.file;

        let index_str = format!("[{}]", index);
        let name = &file.name;
        let path = file.path.display().to_string();

        if self.use_colors {
            print!("{} ", index_str.bright_black());
            print!("{} ", name.bright_white().bold());
            println!("{}", path.bright_black());
        } else {
            println!("[{}] {} ({})", index, name, path);
        }

        if self.verbose {
            let mut details = Vec::new();

            if let Some(ref ext) = file.extension {
                details.push(format!("ext: {}", ext));
            }

            details.push(format!("size: {}", format_size(file.size)));

            if let Some(modified) = file.modified_at {
                details.push(format!("modified: {}", format_relative_date(modified)));
            }

            if result.score > 0.0 {
                details.push(format!("score: {:.2}", result.score));
            }

            let details_str = details.join(" | ");
            if self.use_colors {
                println!("  {}", details_str.bright_black());
            } else {
                println!("  {}", details_str);
            }
        }

        if let Some(ref snippet) = result.snippet {
            if self.use_colors {
                println!("  {}", snippet.as_str().bright_yellow());
            } else {
                println!("  {}", snippet);
            }
        }

        println!();
    }

    pub fn print_index_stats(&self, stats: &IndexStats) {
        self.print_header("Index Statistics");
        println!();

        self.print_stat("Total Files", &stats.total_files.to_string());
        self.print_stat("Total Directories", &stats.total_directories.to_string());
        self.print_stat("Total Size", &format_size(stats.total_size));
        self.print_stat(
            "Indexed Files (Content)",
            &stats.indexed_files.to_string(),
        );
        self.print_stat("Last Update", &format_date(stats.last_update));
        self.print_stat("Index Size", &format_size(stats.index_size));

        println!();
    }

    pub fn print_update_stats(&self, stats: &UpdateStats) {
        self.print_header("Index Update Summary");
        println!();

        self.print_stat("Files Added", &stats.added.to_string());
        self.print_stat("Files Updated", &stats.updated.to_string());
        self.print_stat("Files Removed", &stats.removed.to_string());
        self.print_stat("Total Changes", &stats.total().to_string());

        println!();
    }

    pub fn print_verification_stats(&self, stats: &VerificationStats) {
        self.print_header("Index Verification Results");
        println!();

        self.print_stat("Total Indexed", &stats.total_indexed.to_string());
        self.print_stat("Valid", &stats.valid.to_string());
        self.print_stat("Outdated", &stats.outdated.to_string());
        self.print_stat("Missing", &stats.missing.to_string());
        self.print_stat(
            "Health",
            &format!("{:.1}%", stats.health_percentage()),
        );

        println!();
    }

    fn print_stat(&self, label: &str, value: &str) {
        if self.use_colors {
            println!("  {}: {}", label.cyan(), value.white());
        } else {
            println!("  {}: {}", label, value);
        }
    }

    pub fn print_header(&self, text: &str) {
        if self.use_colors {
            println!("{}", text.bright_green().bold());
        } else {
            println!("{}", text);
            println!("{}", "=".repeat(text.len()));
        }
    }

    pub fn print_info(&self, text: &str) {
        if self.use_colors {
            println!("{}", text.bright_blue());
        } else {
            println!("{}", text);
        }
    }

    pub fn print_success(&self, text: &str) {
        if self.use_colors {
            println!("{} {}", "✓".green(), text.green());
        } else {
            println!("[SUCCESS] {}", text);
        }
    }

    pub fn print_error(&self, text: &str) {
        if self.use_colors {
            eprintln!("{} {}", "✗".red(), text.red());
        } else {
            eprintln!("[ERROR] {}", text);
        }
    }

    pub fn print_warning(&self, text: &str) {
        if self.use_colors {
            println!("{} {}", "⚠".yellow(), text.yellow());
        } else {
            println!("[WARNING] {}", text);
        }
    }

    pub fn print_summary(&self, text: &str) {
        if self.use_colors {
            println!("{}", text.bright_white().bold());
        } else {
            println!("{}", text);
        }
    }

    pub fn print_progress(&self, message: &str) {
        if self.use_colors {
            print!("\r{}", message.bright_black());
        } else {
            print!("\r{}", message);
        }
        use std::io::Write;
        std::io::stdout().flush().ok();
    }
}

impl Default for OutputFormatter {
    fn default() -> Self {
        Self::new(true, false)
    }
}

pub fn print_table(headers: &[&str], rows: &[Vec<String>], use_colors: bool) {
    let mut col_widths = vec![0; headers.len()];

    for (i, header) in headers.iter().enumerate() {
        col_widths[i] = header.len();
    }

    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < col_widths.len() {
                col_widths[i] = col_widths[i].max(cell.len());
            }
        }
    }

    let separator = col_widths
        .iter()
        .map(|w| "-".repeat(w + 2))
        .collect::<Vec<_>>()
        .join("+");

    if use_colors {
        for (i, header) in headers.iter().enumerate() {
            print!("| {:<width$} ", header.cyan(), width = col_widths[i]);
        }
    } else {
        for (i, header) in headers.iter().enumerate() {
            print!("| {:<width$} ", header, width = col_widths[i]);
        }
    }
    println!("|");

    println!("+{}+", separator);

    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < col_widths.len() {
                print!("| {:<width$} ", cell, width = col_widths[i]);
            }
        }
        println!("|");
    }
}
