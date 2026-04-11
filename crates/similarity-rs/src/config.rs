use clap::Parser;
use similarity_core::ConfigLoader;

#[derive(Debug, Parser)]
#[command(name = "similarity-rs")]
#[command(about = "Rust code similarity analyzer")]
#[command(version)]
pub struct Cli {
    /// Paths to analyze (files or directories)
    #[arg(default_value = ".")]
    pub paths: Vec<String>,

    /// Print code in output
    #[arg(short, long)]
    pub print: bool,

    /// Similarity threshold (0.0-1.0)
    #[arg(short, long)]
    pub threshold: Option<f64>,

    /// File extensions to check
    #[arg(short, long, value_delimiter = ',')]
    pub extensions: Option<Vec<String>>,

    /// Minimum lines for functions to be considered
    #[arg(short, long)]
    pub min_lines: Option<u32>,

    /// Minimum tokens for functions to be considered
    #[arg(long)]
    pub min_tokens: Option<u32>,

    /// Rename cost for APTED algorithm
    #[arg(short, long)]
    pub rename_cost: Option<f64>,

    /// Disable size penalty for very different sized functions
    #[arg(long)]
    pub no_size_penalty: bool,

    /// Filter functions by name (substring match)
    #[arg(long)]
    pub filter_function: Option<String>,

    /// Filter functions by body content (substring match)
    #[arg(long)]
    pub filter_function_body: Option<String>,

    /// Disable fast mode with bloom filter pre-filtering
    #[arg(long)]
    pub no_fast: bool,

    /// Exclude directories matching the given patterns (can be specified multiple times)
    #[arg(long)]
    pub exclude: Vec<String>,

    /// Skip test functions (functions starting with 'test_' or annotated with #[test])
    #[arg(long)]
    pub skip_test: bool,

    /// Enable experimental overlap detection mode
    #[arg(long = "experimental-overlap")]
    pub overlap: bool,

    /// Minimum window size for overlap detection (number of nodes)
    #[arg(long)]
    pub overlap_min_window: Option<u32>,

    /// Maximum window size for overlap detection (number of nodes)
    #[arg(long)]
    pub overlap_max_window: Option<u32>,

    /// Size tolerance for overlap detection (0.0-1.0)
    #[arg(long)]
    pub overlap_size_tolerance: Option<f64>,

    /// Exit with code 1 if duplicates are found
    #[arg(long)]
    pub fail_on_duplicates: bool,

    /// Enable type similarity checking for structs and enums (experimental)
    #[arg(long = "experimental-types")]
    pub types: bool,

    /// Disable function similarity checking
    #[arg(long = "no-functions")]
    pub no_functions: bool,

    /// Use new generalized structure comparison framework (experimental)
    #[arg(long)]
    pub use_structure_comparison: bool,
}

#[derive(Debug, Default, serde::Deserialize)]
pub struct Config {
    pub threshold: Option<f64>,
    pub extensions: Option<Vec<String>>,
    pub min_lines: Option<u32>,
    pub min_tokens: Option<u32>,
    pub rename_cost: Option<f64>,
    pub no_size_penalty: Option<bool>,
    pub filter_function: Option<String>,
    pub filter_function_body: Option<String>,
    pub no_fast: Option<bool>,
    pub exclude: Option<Vec<String>>,
    pub skip_test: Option<bool>,
    pub overlap: Option<bool>,
    pub overlap_min_window: Option<u32>,
    pub overlap_max_window: Option<u32>,
    pub overlap_size_tolerance: Option<f64>,
    pub fail_on_duplicates: Option<bool>,
    pub types: Option<bool>,
    pub no_functions: Option<bool>,
    pub use_structure_comparison: Option<bool>,
}

impl ConfigLoader for Config {}

pub struct ResolvedConfig {
    pub threshold: f64,
    pub extensions: Option<Vec<String>>,
    pub min_lines: u32,
    pub min_tokens: Option<u32>,
    pub rename_cost: f64,
    pub no_size_penalty: bool,
    pub filter_function: Option<String>,
    pub filter_function_body: Option<String>,
    pub no_fast: bool,
    pub exclude: Vec<String>,
    pub skip_test: bool,
    pub overlap: bool,
    pub overlap_min_window: u32,
    pub overlap_max_window: u32,
    pub overlap_size_tolerance: f64,
    pub fail_on_duplicates: bool,
    pub types: bool,
    pub no_functions: bool,
    pub use_structure_comparison: bool,
}

fn resolve_value<T>(cli: Option<T>, config: Option<T>, default: T) -> T {
    cli.or(config).unwrap_or(default)
}

fn resolve_option<T>(cli: Option<T>, config: Option<T>, default: Option<T>) -> Option<T> {
    cli.or(config).or(default)
}

fn resolve_flag(cli: bool, config: Option<bool>) -> bool {
    cli || config.unwrap_or(false)
}

impl ResolvedConfig {
    pub fn from(cli: Cli, config: Config) -> Self {
        let mut exclude = config.exclude.unwrap_or_default();
        exclude.extend(cli.exclude);

        Self {
            threshold: resolve_value(cli.threshold, config.threshold, 0.85),
            extensions: cli.extensions.or(config.extensions),
            min_lines: resolve_value(cli.min_lines, config.min_lines, 3),
            min_tokens: resolve_option(cli.min_tokens, config.min_tokens, Some(30)),
            rename_cost: resolve_value(cli.rename_cost, config.rename_cost, 0.3),
            no_size_penalty: resolve_flag(cli.no_size_penalty, config.no_size_penalty),
            filter_function: cli.filter_function.or(config.filter_function),
            filter_function_body: cli.filter_function_body.or(config.filter_function_body),
            no_fast: resolve_flag(cli.no_fast, config.no_fast),
            exclude,
            skip_test: resolve_flag(cli.skip_test, config.skip_test),
            overlap: resolve_flag(cli.overlap, config.overlap),
            overlap_min_window: resolve_value(cli.overlap_min_window, config.overlap_min_window, 8),
            overlap_max_window: resolve_value(
                cli.overlap_max_window,
                config.overlap_max_window,
                25,
            ),
            overlap_size_tolerance: resolve_value(
                cli.overlap_size_tolerance,
                config.overlap_size_tolerance,
                0.25,
            ),
            fail_on_duplicates: resolve_flag(cli.fail_on_duplicates, config.fail_on_duplicates),
            types: resolve_flag(cli.types, config.types),
            no_functions: resolve_flag(cli.no_functions, config.no_functions),
            use_structure_comparison: resolve_flag(
                cli.use_structure_comparison,
                config.use_structure_comparison,
            ),
        }
    }
}
