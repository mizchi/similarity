# Rust コード類似度検出の推奨設定

## 実プロジェクトでの検証結果

### 検出された主な重複パターン

1. **構造的に同一なコード（正当な検出）**
   - `extract_struct_definition` と `extract_enum_definition`: 98.44%
   - 実際にリファクタリング可能な重複コード

2. **偽陽性のパターン**
   - テスト関数: 構造が似ているため95-99%の類似度
   - 短い関数: サイズペナルティがあっても誤検出されやすい

### 推奨パラメータ設定

#### 1. 一般的な重複検出
```bash
similarity-rs --threshold 0.8 --min-lines 10 --min-tokens 50
```
- 80%以上の類似度
- 10行以上の関数
- 50トークン以上（ASTノード数）

#### 2. 厳密な重複検出
```bash
similarity-rs --threshold 0.9 --min-lines 15 --min-tokens 100
```
- 90%以上の類似度
- 15行以上の関数
- 100トークン以上

#### 3. テストコードを除外
```bash
similarity-rs --threshold 0.8 --min-lines 10 --skip-test
```
- `#[test]` 属性の付いた関数を除外
- `test_` で始まる関数を除外

### パラメータの影響

| パラメータ | 効果 | 推奨値 |
|----------|------|--------|
| `threshold` | 類似度の閾値 | 0.8-0.9 |
| `min-lines` | 最小行数 | 10-15 |
| `min-tokens` | 最小トークン数 | 50-100 |
| `size-penalty` | 短い関数へのペナルティ | true（デフォルト） |
| `rename-cost` | 変数名の違いへの寛容度 | 0.3（デフォルト） |

### 実際の使用例

#### CI/CDでの使用
```yaml
- name: Check code duplication
  run: |
    cargo install similarity-rs
    similarity-rs src \
      --threshold 0.85 \
      --min-lines 12 \
      --min-tokens 60 \
      --skip-test
```

#### リファクタリング候補の検出
```bash
# 高い類似度の長い関数を検出
similarity-rs src \
  --threshold 0.95 \
  --min-lines 20 \
  --min-tokens 150
```

### 注意事項

1. **テストコードの扱い**
   - テスト関数は構造が似ているため偽陽性が多い
   - `--skip-test` オプションの使用を推奨

2. **最小トークン数の重要性**
   - `min-tokens` を設定しないと短い関数で偽陽性が増える
   - 50トークン以上を推奨

3. **言語特性の考慮**
   - Rustのマクロ展開後のコードは検出されない
   - ジェネリクスの具体化は別関数として扱われる

### まとめ

`compare_values` パラメータの修正により、Rust コードの類似度検出が大幅に改善されました。適切なパラメータ設定により、実用的な重複検出が可能になっています。