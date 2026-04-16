---
name: check-similarity-py
description: Detect duplicate Python code using AST-based similarity analysis. Use when working with .py files and looking for code duplication or refactoring opportunities.
argument-hint: "[path] [--threshold 0.85] [--print]"
allowed-tools: Bash(similarity-py *) Bash(cargo run -p similarity-py *) Read Grep Glob
paths: "**/*.py"
---

# Python Code Similarity Detection

## What to do

Run `similarity-py` on the target project to detect duplicate functions and classes, then analyze results and propose refactoring.

If `similarity-py` is not installed, build it:

```bash
cargo install --path crates/similarity-py
```

## Step 1: Run similarity analysis

```bash
similarity-py $ARGUMENTS
```

If no arguments given:

```bash
similarity-py . --threshold 0.85 --min-lines 5
```

## Step 2: Analyze results

### High-priority
- **100% similarity**: Identical functions with renamed variables -> extract shared function
- **95-100%**: Same algorithm, different names -> parameterize
- **Duplicate class methods**: Same logic across classes -> extract mixin or base class

### Medium-priority
- **85-95%**: Similar data processing with minor differences -> shared utility
- **Decorated variants**: Same function with different decorators -> single function with configurable decoration

### Acceptable
- **Short `__init__` methods** that set attributes
- **Property getters/setters** with trivial structure

## Step 3: Propose refactoring

For each high-priority pair, show before/after code.

## Key Options

| Option | Description |
|--------|-------------|
| `--threshold <0-1>` | Similarity threshold (default: 0.85) |
| `--min-lines <n>` | Skip functions shorter than n lines (default: 3) |
| `--print` | Show actual code snippets |
| `--filter-function <name>` | Filter by function name |
| `--fail-on-duplicates` | Exit code 1 if duplicates found |
| `--experimental-overlap` | Enable partial overlap detection |

## Common Python refactoring patterns

- **Data processing functions** with different field access -> generic with key parameter
- **API endpoint handlers** -> shared decorator or base handler
- **Validation methods** -> schema-based (pydantic, attrs)
- **Duplicate class methods** across classes -> mixin class or standalone function
- **Test setup** duplication -> fixtures or shared base test class
