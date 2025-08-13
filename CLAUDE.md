## Project Goal

Calculate code similarity between functions and types across multiple programming languages.

## Project Structure

### Crates Organization Policy

- **crates/core (similarity-ts-core)**: 言語非依存のコア機能
  - AST比較アルゴリズム (APTED, TSED)
  - 共通パーサーインターフェース (`LanguageParser` trait)
  - CLI共通ユーティリティ (`cli_parallel`, `cli_output`, `cli_file_utils`)
  
- **crates/similarity-ts**: TypeScript/JavaScript専用CLI
  - oxc_parser を使用した高速パース
  - 型システムの類似性検出 (type_comparator, type_extractor)
  - JSX/TSX サポート
  
- **crates/similarity-py**: Python専用CLI
  - tree-sitter-python を使用
  - Python固有の構文サポート
  - クラス・メソッドの検出
  
- **__deprecated/**: TypeScriptプロトタイプ実装（参考用のみ）

### Multi-Language Support Policy

1. **言語ごとに独立したCLIパッケージを提供**
   - `similarity-ts`: TypeScript/JavaScript専用
   - `similarity-py`: Python専用
   - 将来: `similarity-rs` (Rust), `similarity-go` (Go) など

2. **言語固有機能は各パッケージに実装**
   - TypeScript: 型システム、インターフェース、ジェネリクス、JSX
   - Python: デコレータ、内包表記、インデント構造
   - 各言語の特性に最適化した検出パターン

3. **共通機能はcoreに集約**
   - AST比較アルゴリズム
   - 並列処理フレームワーク
   - ファイル操作ユーティリティ
   - 将来のクロス言語比較の基盤

## Development Stack

### Rust (Main)
- cargo (workspace構成)
- clap (CLI framework)
- oxc_parser (TypeScript/JavaScript parser - 高速)
- tree-sitter (Python, その他の言語 - 汎用的だが約10倍遅い)
- rayon (並列処理)

## Coding Rules

### Rust
- Follow standard Rust conventions
- Use clippy for linting
- Run tests with `cargo test`
- push する前には .github/workflows/rust.yaml 相当の確認のテストを実行して確認

## Directory Patterns

```
crates/              # Rust implementation (main)
  core/              # Language-agnostic core logic
  similarity-ts/     # TypeScript/JavaScript CLI
  similarity-py/     # Python CLI
examples/            # Example files
  mixed_language_project/  # Multi-language examples
__deprecated/        # Deprecated TypeScript prototype
```

## Features

### Common Features (All Languages)
- Function similarity detection using AST-based comparison
- Configurable similarity thresholds
- Cross-file analysis support
- VSCode-compatible output format
- Parallel processing for performance

### TypeScript/JavaScript Specific
- Type similarity detection (interfaces, type aliases, type literals)
- Class similarity detection (properties, methods, inheritance)
- JSX/TSX support
- ES6+ syntax support (arrow functions, classes, etc.)
- Fast parsing with oxc_parser

### Python Specific
- Class and method detection
- Decorator support
- Python 3.x syntax support
- Indentation-based structure analysis

## Future Language Expansion

新しい言語を追加する場合:

1. `crates/similarity-{lang}` ディレクトリを作成
2. `LanguageParser` trait を実装
3. 言語固有の機能を実装
4. 統合テストを追加

## Important Implementation Notes

### 類似度計算には必ず calculate_tsed を使用する

**重要**: AST の類似度を計算する際は、必ず `similarity_core::tsed::calculate_tsed` 関数を使用すること。

**間違った実装例**:
```rust
// ❌ 直接計算してはいけない
let dist = compute_edit_distance(&tree1, &tree2, &options.apted_options);
let size1 = tree1.get_subtree_size();
let size2 = tree2.get_subtree_size();
let max_size = size1.max(size2) as f64;
let similarity = if max_size > 0.0 { 1.0 - (dist / max_size) } else { 1.0 };
```

**正しい実装例**:
```rust
// ✅ calculate_tsed を使用する
let similarity = similarity_core::tsed::calculate_tsed(&tree1, &tree2, options);
```

**理由**:
- `calculate_tsed` は `size_penalty` などの重要なオプションを適用する
- サイズが大きく異なる関数間の false positive を防ぐ
- 小さい関数に対する適切なペナルティを適用する

この間違いは特に新しい言語サポートを追加する際に発生しやすいため、必ず確認すること。

```