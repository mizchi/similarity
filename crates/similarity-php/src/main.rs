use anyhow::Result;
use clap::Parser;

mod check;
mod parallel;
mod php_parser;

#[derive(Parser)]
#[command(name = "similarity-php")]
#[command(about = "PHP code similarity analyzer")]
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

    /// Enable experimental overlap detection mode
    #[arg(long = "experimental-overlap")]
    overlap: bool,

    /// Minimum window size for overlap detection (number of nodes)
    #[arg(long, default_value = "8")]
    overlap_min_window: u32,

    /// Maximum window size for overlap detection (number of nodes)
    #[arg(long, default_value = "25")]
    overlap_max_window: u32,

    /// Size tolerance for overlap detection (0.0-1.0)
    #[arg(long, default_value = "0.25")]
    overlap_size_tolerance: f64,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let functions_enabled = true; // PHP always has functions enabled
    let overlap_enabled = cli.overlap;

    println!("Analyzing PHP code similarity...\n");

    let separator = "-".repeat(60);

    // Run functions analysis
    if !overlap_enabled || functions_enabled {
        println!("=== Function Similarity ===");
        check::check_paths(
            cli.paths.clone(),
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
    }

    // Run overlap detection if enabled
    if overlap_enabled {
        if functions_enabled {
            println!("\n{separator}");
        }
        println!("=== Overlap Detection (Experimental) ===");
        check::check_overlap(
            cli.paths,
            cli.threshold,
            cli.overlap_min_window,
            cli.overlap_max_window,
            cli.overlap_size_tolerance,
            cli.print,
        )?;
    }

    Ok(())
}