use clap::{Parser, Subcommand};
use rusty_files::prelude::*;
use rusty_files::SearchEngine;
use std::path::PathBuf;

mod commands;
mod interactive;
mod output;

use commands::CommandExecutor;
use interactive::InteractiveMode;

#[derive(Parser)]
#[command(
    name = "filesearch",
    about = "A high-performance file search engine",
    version,
    author
)]
struct Cli {
    #[arg(short, long, global = true, help = "Path to index database")]
    index: Option<PathBuf>,

    #[arg(short, long, global = true, help = "Enable verbose output")]
    verbose: bool,

    #[arg(long, global = true, help = "Disable colored output")]
    no_color: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Build index for a directory")]
    Index {
        #[arg(help = "Directory to index")]
        path: PathBuf,

        #[arg(short, long, help = "Show progress")]
        progress: bool,
    },

    #[command(about = "Update existing index")]
    Update {
        #[arg(help = "Directory to update")]
        path: PathBuf,

        #[arg(short, long, help = "Show progress")]
        progress: bool,
    },

    #[command(about = "Search for files")]
    Search {
        #[arg(help = "Search query")]
        query: String,
    },

    #[command(about = "Show index statistics")]
    Stats,

    #[command(about = "Verify index integrity")]
    Verify {
        #[arg(help = "Directory to verify")]
        path: PathBuf,
    },

    #[command(about = "Watch directory for changes")]
    Watch {
        #[arg(help = "Directory to watch")]
        path: PathBuf,
    },

    #[command(about = "Clear index")]
    Clear {
        #[arg(long, help = "Confirm deletion")]
        confirm: bool,
    },

    #[command(about = "Optimize database")]
    Vacuum,

    #[command(about = "Export search results")]
    Export {
        #[arg(short, long, help = "Output file path")]
        output: PathBuf,

        #[arg(short, long, help = "Search query to export")]
        query: Option<String>,
    },

    #[command(about = "Start interactive search mode")]
    Interactive,
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();

    let index_path = cli
        .index
        .unwrap_or_else(|| PathBuf::from("./filesearch.db"));

    let engine = match SearchEngine::new(&index_path) {
        Ok(e) => e,
        Err(err) => {
            eprintln!("Failed to initialize search engine: {}", err);
            std::process::exit(1);
        }
    };

    let executor = CommandExecutor::new(engine, !cli.no_color, cli.verbose);

    let result = match cli.command {
        Commands::Index { path, progress } => executor.index(path, progress),
        Commands::Update { path, progress } => executor.update(path, progress),
        Commands::Search { query } => executor.search(query),
        Commands::Stats => executor.stats(),
        Commands::Verify { path } => executor.verify(path),
        Commands::Watch { path } => executor.watch(path),
        Commands::Clear { confirm } => executor.clear(confirm),
        Commands::Vacuum => executor.vacuum(),
        Commands::Export { output, query } => executor.export(output, query),
        Commands::Interactive => {
            let engine = match SearchEngine::new(&index_path) {
                Ok(e) => e,
                Err(err) => {
                    eprintln!("Failed to initialize search engine: {}", err);
                    std::process::exit(1);
                }
            };
            let mut interactive = InteractiveMode::new(engine);
            interactive.run()
        }
    };

    if let Err(err) = result {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}
