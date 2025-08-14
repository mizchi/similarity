# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.2] - 2025-01-22

### Added
- **Cross-Language Structure Comparison Framework**
  - Language-agnostic structure comparison engine for unified similarity detection
  - TypeScript adapter for interfaces, type aliases, and classes
  - Rust adapter with derive attribute support (treated as regular members)
  - CSS adapter for rule and property comparison
  - Fingerprint-based pre-filtering for improved performance
  - Size penalty mechanism to reduce false positives
  - `--use-structure-comparison` flag to enable the new comparison mode (TypeScript and CSS)

### Changed
- Rust struct/enum comparison now includes derive attributes as part of the structure
- CSS parser improved to correctly extract rules and declarations

### Fixed
- CSS rule extraction in similarity-css now works correctly with tree-sitter-css

## [0.4.1] - 2025-01-21

### Fixed
- Fixed `--exclude` option to properly handle relative directory paths (e.g., `tests/fixtures`)
- Improved exclude pattern matching with automatic wildcard expansion
- Fixed clippy warnings and code formatting issues
- Confirmed `.gitignore` is properly respected by default

### Changed
- `--exclude` patterns now automatically expand to match both files and directories
- Example: `--exclude="tests"` now matches `tests/`, `**/tests/`, and `tests/**`

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