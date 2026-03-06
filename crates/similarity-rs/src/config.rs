use clap::Parser;
use similarity_core::ConfigLoader;

#[derive(Parser)]
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

/// Settings loaded from `similarity.toml`.
/// All fields are `Option<T>` — missing keys simply stay `None`.
///
/// Example `similarity.toml`:
/// ```toml
/// threshold = 0.90
/// min_lines = 5
/// min_tokens = 50
/// skip_test = true
/// exclude = ["target/", "tests/fixtures/"]
/// ```
#[derive(serde::Deserialize, Default)]
pub struct Config {
    pub threshold: Option<f64>,
    pub min_lines: Option<u32>,
    pub min_tokens: Option<u32>,
    pub rename_cost: Option<f64>,
    pub no_size_penalty: Option<bool>,
    pub skip_test: Option<bool>,
    pub no_fast: Option<bool>,
    pub fail_on_duplicates: Option<bool>,
    pub exclude: Option<Vec<String>>,
    pub extensions: Option<Vec<String>>,
    pub filter_function: Option<String>,
    pub filter_function_body: Option<String>,
    pub overlap: Option<bool>,
    pub overlap_min_window: Option<u32>,
    pub overlap_max_window: Option<u32>,
    pub overlap_size_tolerance: Option<f64>,
    pub types: Option<bool>,
    pub no_functions: Option<bool>,
    pub use_structure_comparison: Option<bool>,
}

impl ConfigLoader for Config {}

/// Resolved settings after merging CLI args over config file values.
/// All fields are concrete — defaults have been applied.
pub struct ResolvedConfig {
    pub threshold: f64,
    pub min_lines: u32,
    pub min_tokens: Option<u32>,
    pub rename_cost: f64,
    pub no_size_penalty: bool,
    pub skip_test: bool,
    pub no_fast: bool,
    pub fail_on_duplicates: bool,
    pub overlap: bool,
    pub types_enabled: bool,
    pub no_functions: bool,
    pub use_structure_comparison: bool,
    pub overlap_min_window: u32,
    pub overlap_max_window: u32,
    pub overlap_size_tolerance: f64,
    pub exclude: Vec<String>,
    pub extensions: Option<Vec<String>>,
    pub filter_function: Option<String>,
    pub filter_function_body: Option<String>,
}

/// CLI value wins over config value; falls back to `default` if neither is set.
fn resolve<T>(cli: Option<T>, cfg: Option<T>, default: T) -> T {
    cli.or(cfg).unwrap_or(default)
}

/// CLI bool flag wins; config can also enable it when CLI didn't pass it.
fn resolve_flag(cli: bool, cfg: Option<bool>) -> bool {
    cli || cfg.unwrap_or(false)
}

impl ResolvedConfig {
    /// Merge CLI args over config file values. CLI always wins.
    /// For bool presence flags (already `true` if passed on CLI), config can also enable them.
    /// Exclude lists are merged: config entries first, CLI entries appended (CLI has final say).
    pub fn from(cli: Cli, cfg: Config) -> Self {
        // Config exclude entries first, CLI entries appended — so CLI patterns take precedence.
        let mut exclude = cfg.exclude.unwrap_or_default();
        exclude.extend(cli.exclude);

        Self {
            threshold:                resolve(cli.threshold, cfg.threshold, 0.85),
            min_lines:                resolve(cli.min_lines, cfg.min_lines, 3),
            min_tokens:               cli.min_tokens.or(cfg.min_tokens),
            rename_cost:              resolve(cli.rename_cost, cfg.rename_cost, 0.3),
            no_size_penalty:          resolve_flag(cli.no_size_penalty, cfg.no_size_penalty),
            skip_test:                resolve_flag(cli.skip_test, cfg.skip_test),
            no_fast:                  resolve_flag(cli.no_fast, cfg.no_fast),
            fail_on_duplicates:       resolve_flag(cli.fail_on_duplicates, cfg.fail_on_duplicates),
            overlap:                  resolve_flag(cli.overlap, cfg.overlap),
            types_enabled:            resolve_flag(cli.types, cfg.types),
            no_functions:             resolve_flag(cli.no_functions, cfg.no_functions),
            use_structure_comparison: resolve_flag(cli.use_structure_comparison, cfg.use_structure_comparison),
            overlap_min_window:       resolve(cli.overlap_min_window, cfg.overlap_min_window, 8),
            overlap_max_window:       resolve(cli.overlap_max_window, cfg.overlap_max_window, 25),
            overlap_size_tolerance:   resolve(cli.overlap_size_tolerance, cfg.overlap_size_tolerance, 0.25),
            exclude,
            extensions:               cli.extensions.or(cfg.extensions),
            filter_function:          cli.filter_function.or(cfg.filter_function),
            filter_function_body:     cli.filter_function_body.or(cfg.filter_function_body),
        }
    }
}
