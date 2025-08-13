# similarity-ts/rs/py

High-performance code similarity detection tools written in Rust. Detects duplicate functions and similar code patterns across your codebase in multiple programming languages.

## Tool Maturity Status

| Tool                   | Language                  | Status                  | Description                       |
| ---------------------- | ------------------------- | ----------------------- | --------------------------------- |
| **similarity-ts**      | TypeScript/JavaScript     | ✅ **Production Ready** | Most mature and production-tested |
| **similarity-py**      | Python                    | ⚠️ **Beta**             | Not production-tested yet         |
| **similarity-rs**      | Rust                      | ⚠️ **Beta**             | Not production-tested yet         |
| **similarity-elixir**  | Elixir                    | 🧪 **Experimental**     | Early development stage           |
| **similarity-generic** | Go, Java, C/C++, C#, Ruby | 🧪 **Experimental**     | Early development stage           |
| **similarity-md**      | Markdown                  | 🧪 **Experimental**     | Early development stage           |
| **similarity-css**     | CSS/SCSS                  | 🧪 **Experimental**     | Prototype implementation          |

## Features

- **Zero configuration** - works out of the box
- **Multi-language support** - TypeScript/JavaScript, Python, and Rust
- **Fast & Accurate** - AST-based comparison, not just text matching
- **AI-friendly output** - Easy to share with Claude, GPT-4, etc.

## Quick Start

### 1. Install (TypeScript/JavaScript)

```bash
cargo install similarity-ts
```

### 2. Detect duplicates

```bash
# Scan current directory
similarity-ts .

# Scan specific files
similarity-ts src/utils.ts src/helpers.ts

# Show actual code
similarity-ts . --print
```

### 3. Refactor with AI

Copy the output and use this prompt with Claude:

```
Run `similarity-ts .` to detect semantic code similarities. Execute this command, analyze the duplicate code patterns, and create a refactoring plan. Check `similarity-ts -h` for detailed options.
```

Example output:

```
Duplicates in src/utils.ts:
────────────────────────────────────────────────────────────
  src/utils.ts:10-20 calculateTotal <-> src/helpers.ts:5-15 computeSum
  Similarity: 92.50%, Score: 9.2 points
```

The AI will analyze patterns and suggest refactoring strategies.

## Documentation

- [AI Assistant Guide](.claude/commands/check-similarity.md) - Refactoring workflow and best practices

## Available Tools

### Production Ready

- **similarity-ts** - TypeScript/JavaScript similarity detection ✅ **Most mature and production-tested**

### Beta (Not production-tested yet)

- **similarity-py** - Python similarity detection ⚠️ _Not production-tested_
- **similarity-rs** - Rust similarity detection ⚠️ _Not production-tested_

### Experimental

- **similarity-elixir** - Elixir similarity detection 🧪 _Experimental_
- **similarity-generic** - Generic similarity detection for Go, Java, C/C++, C#, Ruby 🧪 _Experimental_
- **similarity-md** - Markdown similarity detection 🧪 _Experimental_
- **similarity-css** - CSS/SCSS similarity detection 🧪 _Experimental_ - Detects duplicate styles and BEM variations

## Installation

### TypeScript/JavaScript

```bash
# Install from crates.io
cargo install similarity-ts

# Use the installed binary
similarity-ts --help
```

### Python

```bash
# Install from crates.io
cargo install similarity-py

# Use the installed binary
similarity-py --help
```

### Rust

```bash
# Install from crates.io
cargo install similarity-rs

# Use the installed binary
similarity-rs --help
```

### Elixir

```bash
# Install from crates.io
cargo install similarity-elixir

# Use the installed binary
similarity-elixir --help
```

### Other Languages (Go, Java, C/C++, C#, Ruby)

```bash
# Install from crates.io
cargo install similarity-generic

# Use the installed binary
similarity-generic --language go main.go
similarity-generic --language java Main.java
```

### From source

```bash
# Clone the repository
git clone https://github.com/mizchi/similarity.git
cd similarity

# Build all tools
cargo build --release

# Or install specific tool
cargo install --path crates/similarity-ts
cargo install --path crates/similarity-py
cargo install --path crates/similarity-rs
```

## Usage

### Common Options (All Languages)

- `--threshold` / `-t` - Similarity threshold (0.0-1.0, default: 0.85)
- `--min-lines` / `-m` - Minimum lines for functions (default: 3-5)
- `--min-tokens` - Minimum AST nodes for functions
- `--print` / `-p` - Print code in output
- `--cross-file` / `-c` - Enable cross-file comparison
- `--no-size-penalty` - Disable size difference penalty

### TypeScript/JavaScript Specific

```bash
# Check for duplicate functions (default)
similarity-ts ./src

# Enable type checking (default enabled)
similarity-ts ./src --no-types  # disable type checking

# Check types only
similarity-ts ./src --no-functions

# Check classes for similarity
similarity-ts ./src --classes

# Check classes only
similarity-ts ./src --classes-only

# Fast mode with bloom filter (default)
similarity-ts ./src --no-fast  # disable
```

### Python Specific

```bash
# Check Python files
similarity-py ./src

# Include test files
similarity-py . --extensions py,test.py
```

### Rust Specific

```bash
# Check Rust files
similarity-rs ./src

# Skip test functions (test_ prefix or #[test])
similarity-rs . --skip-test

# Set minimum tokens (default: 30)
similarity-rs . --min-tokens 50
```

## Output Format

The tool outputs in a VSCode-compatible format for easy navigation:

```
Duplicates in src/utils.ts:
────────────────────────────────────────────────────────────
  src/utils.ts:10 | L10-15 similar-function: calculateSum
  src/utils.ts:20 | L20-25 similar-function: addNumbers
  Similarity: 85.00%, Priority: 8.5 (lines: 10)
```

Click on the file paths in VSCode's terminal to jump directly to the code.

Results are sorted by priority (lines × similarity) to help you focus on the most impactful duplications first.

## AI Integration

### Prompt for Code Deduplication

For AI assistants (like Claude, GPT-4, etc.) to help with code deduplication:

```
`similarity-ts .` でコードの意味的な類似が得られます。あなたはこれを実行し、ソースコードの重複を検知して、リファクタリング計画を立てます。細かいオプションは similarity-ts -h で確認してください。
```

English version:

```
Run `similarity-ts .` to detect semantic code similarities. Execute this command, analyze the duplicate code patterns, and create a refactoring plan. Check `similarity-ts -h` for detailed options.
```

### Example Workflow with AI

1. **Run similarity detection**:

   ```bash
   similarity-ts . --threshold 0.8 --min-lines 10
   ```

2. **Share output with AI**: Copy the similarity report to your AI assistant

3. **AI analyzes patterns**: The AI will identify common patterns and suggest refactoring strategies

4. **Iterative refinement**: Adjust threshold and options based on AI recommendations

### Integration with Development Tools

This tool can be integrated into:

- Pre-commit hooks to prevent duplicate code
- CI/CD pipelines for code quality checks
- IDE extensions for real-time duplicate detection
- AI-powered code review workflows

## How It Works

### Core Algorithm

1. **AST Parsing**: Language-specific parsers convert code to ASTs
   - TypeScript/JavaScript: oxc-parser (fast)
   - Python/Rust: tree-sitter
2. **Tree Extraction**: Extracts function/method nodes with structure
3. **TSED Algorithm**: Tree Structure Edit Distance with size penalties
4. **Similarity Score**: Normalized score between 0 and 1
5. **Impact Calculation**: Considers code size for prioritization

### Overlap Detection (Experimental)

The `--experimental-overlap` flag enables detection of partial code overlaps within and across functions:

```bash
# Basic overlap detection
similarity-ts ./src --experimental-overlap

# With custom parameters
similarity-ts ./src --experimental-overlap \
  --threshold 0.75 \
  --overlap-min-window 8 \
  --overlap-max-window 25 \
  --overlap-size-tolerance 0.25
```

**Parameters:**

- `--experimental-overlap`: Enable overlap detection mode
- `--overlap-min-window`: Minimum AST nodes to consider (default: 8)
- `--overlap-max-window`: Maximum AST nodes to consider (default: 25)
- `--overlap-size-tolerance`: Size variation tolerance (default: 0.25)

**Use Cases:**

- Finding copy-pasted code fragments within larger functions
- Detecting similar algorithmic patterns across different contexts
- Identifying refactoring opportunities for common code blocks

### Language-Specific Features

- **TypeScript**: Type similarity detection (interfaces, type aliases)
- **Python**: Class and method detection, decorator support
- **Rust**: Test function filtering, impl block analysis

## Examples

### TypeScript/JavaScript

```bash
# Find duplicate functions
similarity-ts ./src --threshold 0.7 --print

# Find similar types across files
similarity-ts ./src --no-functions --experimental-types --cross-file --print

# Comprehensive analysis
similarity-ts ./src \
  --threshold 0.8 \
  --min-lines 10 \
  --cross-file \
  --extensions ts,tsx

# Detect partial code overlaps (Experimental)
similarity-ts ./src --experimental-overlap --threshold 0.75 --print
```

### Python

```bash
# Find duplicate functions in Python project
similarity-py ./src --threshold 0.85 --print

# Check with custom settings
similarity-py . \
  --min-lines 5 \
  --extensions py
```

### Rust

```bash
# Find duplicates excluding tests
similarity-rs ./src --skip-test --print

# Strict checking with high token count
similarity-rs . \
  --min-tokens 50 \
  --threshold 0.9 \
  --skip-test
```

## Experimental: Generic Language Support

> ⚠️ **EXPERIMENTAL**: The generic language support is in early development and may have limitations or bugs.

### similarity-generic

The `similarity-generic` tool provides experimental support for additional languages using tree-sitter parsers:

- Go
- Java
- C
- C++
- C#
- Ruby
- Elixir

#### Installation

```bash
# From crates.io (when available)
cargo install similarity-generic

# From source
cargo install --path crates/similarity-generic
```

#### Usage

```bash
# Detect Go duplicates
similarity-generic --language go ./src

# Detect Java duplicates
similarity-generic --language java ./src

# Detect C/C++ duplicates
similarity-generic --language c ./src
similarity-generic --language cpp ./src

# Detect C# duplicates
similarity-generic --language csharp ./src

# Detect Ruby duplicates
similarity-generic --language ruby ./src

# Detect Elixir duplicates
similarity-generic --language elixir ./src

# Common options work the same way
similarity-generic --language go ./src --threshold 0.8 --print
```

#### Supported Languages

| Language | File Extensions           | Status       |
| -------- | ------------------------- | ------------ |
| Go       | .go                       | Experimental |
| Java     | .java                     | Experimental |
| C        | .c, .h                    | Experimental |
| C++      | .cpp, .cc, .cxx, .hpp, .h | Experimental |
| C#       | .cs                       | Experimental |
| Ruby     | .rb                       | Experimental |

#### Custom Language Configuration

You can also provide custom language configurations:

```bash
# Use custom config file
similarity-generic --config ./my-language.json ./src
```

See [examples/configs/custom-language-template.json](crates/similarity-generic/examples/configs/custom-language-template.json) for configuration format.

#### Limitations

- Performance is slower than specialized tools (similarity-ts, similarity-py, similarity-rs)
- Detection accuracy may vary by language
- Some language-specific features may not be fully supported
- Custom configurations require understanding of tree-sitter node types

For production use, prefer the specialized tools when available.

## Performance

- Written in Rust for maximum performance
- Concurrent file processing
- Memory-efficient algorithms
- Language-specific optimizations:
  - **TypeScript/JavaScript**: Fast mode with bloom filters (~4x faster)
  - **Python/Rust**: Tree-sitter based parsing
- Intelligent filtering reduces unnecessary comparisons

## License

MIT
