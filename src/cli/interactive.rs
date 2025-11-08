use crate::output::OutputFormatter;
use rusty_files::core::{Result, SearchEngine};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use std::io::{self, Write};
use std::sync::Arc;
use std::sync::Mutex;

pub struct InteractiveMode {
    engine: Arc<Mutex<SearchEngine>>,
    formatter: OutputFormatter,
    history: Vec<String>,
    history_index: usize,
}

impl InteractiveMode {
    pub fn new(engine: SearchEngine) -> Self {
        Self {
            engine: Arc::new(Mutex::new(engine)),
            formatter: OutputFormatter::new(true, false),
            history: Vec::new(),
            history_index: 0,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        self.print_welcome();

        loop {
            print!("\n> ");
            io::stdout().flush()?;

            let input = self.read_line()?;
            let input = input.trim();

            if input.is_empty() {
                continue;
            }

            if self.handle_command(input)? {
                break;
            }

            self.history.push(input.to_string());
            self.history_index = self.history.len();
        }

        Ok(())
    }

    fn print_welcome(&self) {
        self.formatter.print_header("Rusty Files - Interactive Search");
        println!();
        self.formatter.print_info("Type a search query or use commands:");
        println!("  :help    - Show help");
        println!("  :stats   - Show index statistics");
        println!("  :quit    - Exit interactive mode");
        println!();
    }

    fn handle_command(&self, input: &str) -> Result<bool> {
        if input.starts_with(':') {
            match input {
                ":quit" | ":q" | ":exit" => return Ok(true),
                ":help" | ":h" => {
                    self.print_help();
                }
                ":stats" => {
                    self.print_stats()?;
                }
                ":clear" => {
                    self.clear_screen()?;
                }
                ":history" => {
                    self.print_history();
                }
                _ => {
                    self.formatter.print_error(&format!("Unknown command: {}", input));
                    self.formatter.print_info("Type :help for available commands");
                }
            }
            Ok(false)
        } else {
            self.execute_search(input)?;
            Ok(false)
        }
    }

    fn execute_search(&self, query: &str) -> Result<()> {
        let engine = self.engine.lock().unwrap();
        let results = engine.search(query)?;

        self.formatter.print_search_results(&results, query);

        Ok(())
    }

    fn print_help(&self) {
        self.formatter.print_header("Interactive Mode Help");
        println!();
        println!("Search Queries:");
        println!("  pattern                    - Simple search");
        println!("  pattern ext:rs             - Search with extension filter");
        println!("  pattern size:>1MB          - Search with size filter");
        println!("  pattern modified:today     - Search with date filter");
        println!("  pattern mode:fuzzy         - Use fuzzy matching");
        println!();
        println!("Commands:");
        println!("  :help, :h                  - Show this help");
        println!("  :stats                     - Show index statistics");
        println!("  :clear                     - Clear screen");
        println!("  :history                   - Show search history");
        println!("  :quit, :q, :exit           - Exit interactive mode");
        println!();
    }

    fn print_stats(&self) -> Result<()> {
        let engine = self.engine.lock().unwrap();
        let stats = engine.get_stats()?;

        self.formatter.print_index_stats(&stats);

        Ok(())
    }

    fn print_history(&self) {
        if self.history.is_empty() {
            self.formatter.print_info("No search history");
            return;
        }

        self.formatter.print_header("Search History");
        println!();

        for (i, query) in self.history.iter().enumerate() {
            println!("  {}: {}", i + 1, query);
        }

        println!();
    }

    fn clear_screen(&self) -> Result<()> {
        execute!(io::stdout(), Clear(ClearType::All))?;
        Ok(())
    }

    fn read_line(&self) -> Result<String> {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Ok(input)
    }

    pub fn run_with_raw_mode(&mut self) -> Result<()> {
        enable_raw_mode()?;

        let result = self.run_raw_mode_loop();

        disable_raw_mode()?;

        result
    }

    fn run_raw_mode_loop(&mut self) -> Result<()> {
        self.print_welcome();

        let mut input = String::new();

        loop {
            print!("\r> {}", input);
            io::stdout().flush()?;

            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Enter => {
                        println!();
                        if !input.is_empty() {
                            if self.handle_command(&input)? {
                                break;
                            }
                            self.history.push(input.clone());
                            self.history_index = self.history.len();
                            input.clear();
                        }
                    }
                    KeyCode::Char(c) => {
                        input.push(c);
                    }
                    KeyCode::Backspace => {
                        input.pop();
                    }
                    KeyCode::Esc => {
                        break;
                    }
                    KeyCode::Up => {
                        if self.history_index > 0 {
                            self.history_index -= 1;
                            input = self.history[self.history_index].clone();
                        }
                    }
                    KeyCode::Down => {
                        if self.history_index < self.history.len() - 1 {
                            self.history_index += 1;
                            input = self.history[self.history_index].clone();
                        } else {
                            self.history_index = self.history.len();
                            input.clear();
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_interactive_mode_creation() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = temp_dir.path().join("index.db");
        let engine = SearchEngine::new(&index_path).unwrap();
        let _interactive = InteractiveMode::new(engine);
    }
}
