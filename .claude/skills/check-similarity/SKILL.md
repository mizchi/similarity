---
name: check-similarity
description: Detect duplicate code using AST-based similarity analysis. Auto-selects the right tool based on file types in the project (similarity-ts for TypeScript/JavaScript, similarity-py for Python, similarity-mbt for MoonBit, similarity-rs for Rust, etc).
argument-hint: "[path] [--threshold 0.85] [--print]"
allowed-tools: Bash(similarity-* *) Bash(cargo run -p similarity-* *) Read Grep Glob
---

# Code Similarity Detection (Multi-Language)

## What to do

Detect which languages are present in the target path and run the appropriate similarity tool. Then analyze results and propose refactoring.

## Step 1: Detect language and select tool

Check the target path for file extensions:

| Extension | Tool | Install |
|-----------|------|---------|
| `.ts`, `.tsx`, `.js`, `.jsx` | `similarity-ts` | `cargo install similarity-ts` |
| `.py` | `similarity-py` | `cargo install similarity-py` |
| `.mbt` | `similarity-mbt` | `cargo install similarity-mbt` |
| `.rs` | `similarity-rs` | `cargo install similarity-rs` |
| `.php` | `similarity-php` | `cargo install similarity-php` |
| `.ex`, `.exs` | `similarity-elixir` | `cargo install similarity-elixir` |
| `.css`, `.scss` | `similarity-css` | `cargo install similarity-css` |

If the tool is not installed, build from this repo:

```bash
cargo install --path crates/similarity-<lang>
```

## Step 2: Run analysis

```bash
similarity-<lang> $ARGUMENTS
```

If no arguments given:

```bash
similarity-<lang> . --threshold 0.85 --min-lines 5
```

For TypeScript, also consider type similarity:

```bash
similarity-ts . --threshold 0.85 --min-tokens 25 --experimental-types
```

## Step 3: Analyze and report

Categorize results by priority:

1. **100% similarity**: Exact duplicates - must refactor
2. **95-100%**: Near-identical - high-value refactoring targets
3. **85-95%**: Same pattern - consider extracting shared logic
4. **< 85%**: Similar structure - investigate if related

Report the findings grouped by file, sorted by similarity (highest first).
For each high-priority pair, propose a concrete refactoring with before/after code.

## Common Options (all tools)

| Option | Description |
|--------|-------------|
| `--threshold <0-1>` | Similarity threshold (default: 0.85) |
| `--min-lines <n>` | Skip short functions (default: 3) |
| `--print` | Show code snippets |
| `--fail-on-duplicates` | Exit code 1 if duplicates found |
| `--filter-function <name>` | Filter by function name |

## CI Integration

```bash
# Fail CI if near-exact duplicates exist
similarity-<lang> . --threshold 0.95 --min-lines 5 --fail-on-duplicates
```
