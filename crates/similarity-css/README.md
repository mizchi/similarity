# similarity-css

> ⚠️ **EXPERIMENTAL**: This is a prototype implementation for CSS/SCSS similarity detection. The API and functionality may change significantly. Use at your own risk.

A CSS/SCSS similarity detection tool that identifies duplicate styles, redundant rules, and BEM component variations.

## Features

- **CSS and SCSS parsing** using tree-sitter
- **Nested SCSS syntax flattening** with BEM notation support (`&__element`, `&--modifier`)
- **Multiple similarity detection types**:
  - Exact duplicates
  - Style duplicates (same styles, different selectors)
  - BEM component variations
  - Selector conflicts
- **Shorthand property expansion** for accurate comparison
- **CSS specificity calculation**
- **Multiple output formats**: standard, VSCode, JSON

## Installation

This tool is part of the similarity workspace. Build it with:

```bash
cargo build --release -p similarity-css
```

## Usage

```bash
# Analyze CSS files
similarity-css path/to/css/

# Analyze SCSS files
similarity-css --scss path/to/scss/

# Set custom threshold (0.0-1.0)
similarity-css --threshold 0.7 path/to/css/

# Different output formats
similarity-css --output json path/to/css/
similarity-css --output vscode path/to/css/
```

## Examples

### Analyzing BEM components

```bash
similarity-css --scss examples/scss-bem/
```

This will detect:
- Duplicate button styles (`.btn` vs `.button`)
- Similar form input styles
- BEM modifier variations

### Output Example

```
=== CSS Similarity Analysis Results ===

## Similar Styles Found: 74

1. .btn and .button (similarity: 60.00%)
   Files: button.scss and button.scss
   Lines: 2-14 and 138-149

## BEM Component Variations Found: 37

1. BEM variation: .btn--primary
   Similar to: .btn--secondary
   Similarity: 51.00%
```

## Implementation Notes

- Uses TSED algorithm for AST comparison (currently weighted at 0%)
- Simple text-based SCSS flattener for handling complex nested rules
- Handles multiple selectors and media queries
- Supports single-line CSS rules

## Limitations

- SCSS variable resolution is not implemented
- Mixin expansion is not supported
- Import statements are not followed
- Cross-file BEM component detection is limited

## Future Improvements

- [ ] SCSS variable and mixin support
- [ ] Import resolution
- [ ] CSS-in-JS support
- [ ] Performance optimizations for large codebases
- [ ] Integration with build tools

## License

See the main repository's LICENSE file.