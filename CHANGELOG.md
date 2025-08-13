# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - 2025-01-21

### Added
- **TypeScript/JavaScript**
  - Unified type comparison across type aliases, interfaces, and type literals
  - Type literal extraction from variable declarations, function parameters, and return types
  - Class similarity detection with property and method comparison
  - Self-comparison exclusion logic using line range overlap detection
  - Fingerprint-based optimization for fast type filtering
  - `--unified-types` flag (enabled by default) for comprehensive type comparison
  - `--type-literals-only` flag to check only type literals
  - `--classes` flag for class similarity detection
  - `--include-inheritance` and `--include-implements` flags for filtering inherited classes
  - `--suggest` flag to show refactoring suggestions
- **Rust**
  - Struct and enum similarity detection
  - Type alias comparison
  - Generic type parameter support
- **CSS/SCSS (Experimental)**
  - CSS and SCSS rule similarity detection
  - Selector normalization and comparison
  - Property value normalization
  - SCSS nesting support
  - BEM methodology pattern detection
- **PHP**
  - Function and class similarity detection
  - Namespace-aware comparison
- **Python**
  - Function and class similarity detection
  - Decorator support
- **Elixir**
  - Function similarity detection
  - Module-based comparison
- **Generic Language Support**
  - Tree-sitter based parsing for multiple languages
  - Configurable language detection

### Changed
- Type literals are now included by default in type checking (no longer need `--include-type-literals`)
- Unified type comparison is enabled by default
- Improved CLI help messages and option descriptions
- Better error handling for parse errors
- Performance improvements through parallel processing

### Removed
- `--no-type-literals` option (type literals are always included now)
- `--include-types` option (deprecated, both interfaces and type aliases are included by default)

## [0.1.1] - 2025-01-19

### Added
- `--filter-function <name>` option to filter results by function name (substring match)
- `--filter-function-body <text>` option to filter results by function body content (substring match)

### Changed
- Improved documentation organization and removed outdated files

### Removed
- Removed unnecessary regex dependency, improving build size and compile time

## [0.1.0] - 2025-01-19

### Added
- Initial release with core functionality
- Function similarity detection using AST-based comparison
- Type similarity detection (experimental) for interfaces, type aliases, and type literals
- Cross-file and within-file duplicate detection
- Configurable similarity thresholds
- VSCode-compatible output format
- Fast mode with bloom filter pre-filtering
- Support for TypeScript and JavaScript files (.ts, .tsx, .js, .jsx, .mjs, .cjs, .mts, .cts)
- `--min-tokens` option for filtering by AST node count
- `--print` option to display code snippets
- Parallel file processing for performance