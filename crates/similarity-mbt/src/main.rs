use anyhow::Result;
use clap::Parser;

mod check;
mod moonbit_parser;
mod parallel;

#[derive(Parser)]
#[command(name = "similarity-mbt")]
#[command(about = "MoonBit code similarity analyzer")]
#[command(version)]
struct Cli {
    /// Paths to analyze (files or directories)
    #[arg(default_value = ".")]
    paths: Vec<String>,

    /// Print code in output
    #[arg(short, long)]
    print: bool,

    /// Similarity threshold (0.0-1.0)
    #[arg(short, long, default_value = "0.85")]
    threshold: f64,

    /// File extensions to check
    #[arg(short, long, value_delimiter = ',')]
    extensions: Option<Vec<String>>,

    /// Minimum lines for functions to be considered
    #[arg(short, long, default_value = "3")]
    min_lines: Option<u32>,

    /// Minimum tokens for functions to be considered
    #[arg(long)]
    min_tokens: Option<u32>,

    /// Rename cost for APTED algorithm
    #[arg(short, long, default_value = "0.3")]
    rename_cost: f64,

    /// Disable size penalty for very different sized functions
    #[arg(long)]
    no_size_penalty: bool,

    /// Filter functions by name (substring match)
    #[arg(long)]
    filter_function: Option<String>,

    /// Filter functions by body content (substring match)
    #[arg(long)]
    filter_function_body: Option<String>,

    /// Disable fast mode with bloom filter pre-filtering
    #[arg(long)]
    no_fast: bool,

    /// Exit with code 1 if duplicates are found
    #[arg(long)]
    fail_on_duplicates: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    println!("Analyzing MoonBit code similarity...\n");

    println!("=== Function Similarity ===");
    let duplicate_count = check::check_paths(
        cli.paths,
        cli.threshold,
        cli.rename_cost,
        cli.extensions.as_ref(),
        cli.min_lines.unwrap_or(3),
        cli.min_tokens,
        cli.no_size_penalty,
        cli.print,
        !cli.no_fast,
        cli.filter_function.as_ref(),
        cli.filter_function_body.as_ref(),
    )?;

    if cli.fail_on_duplicates && duplicate_count > 0 {
        std::process::exit(1);
    }

    Ok(())
}
