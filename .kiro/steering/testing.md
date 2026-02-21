# テスト標準

## 方針

- **振る舞いをテストする**（実装詳細ではなく）
- 高速で再現性のあるテストを優先。不安定なテストは即座に修正するか削除
- クリティカルパスを重点的にカバーし、100% カバレッジの追求はしない
- 数値計算の正しさ（精度・収束）の検証を最重要視

## テスト構成

### 配置パターン
- **単体テスト**: 各クレートの `src/` 内に `#[cfg(test)] mod tests` として配置
- **結合テスト**: 各クレートの `tests/` ディレクトリに配置
- **クレート横断テスト**: `apps/simple-solver` の `tests/` で統合検証

### 命名規約
- テスト関数: `test_<対象>_<条件>_<期待結果>`（例: `test_dim_add_same_dimension_succeeds`）
- テストモジュール: テスト対象のモジュール名に対応

## テスト種別

### 単体テスト（各クレート内）
- 型演算の正しさ（次元の加減乗除、テンソル演算）
- trait 実装の網羅（`FieldValue`, `HasGrad`, `HasDiv`）
- typestate 遷移の検証（`Fresh`/`Stale`）
- コンパイルエラーとなるべきケースは `compile_fail` テストで

### 数値検証テスト
- 既知の解析解との比較（製造解法 / Method of Manufactured Solutions）
- 格子収束性の確認（Richardson 外挿、収束次数の検証）
- 保存則（質量・運動量）の検証

### 結合テスト
- クレート間の依存関係が正しく機能すること
- `evaluate_boundaries` → 離散化演算のフロー全体
- MPI 並列テストは `mpirun -np N cargo test` で実行

## テスト構造（AAA パターン）

```rust
#[test]
fn test_pressure_velocity_dimension_product() {
    // Arrange
    let p = Pressure::new(101325.0);
    let v = Velocity::new([1.0, 0.0, 0.0]);

    // Act
    let result = p * v;

    // Assert
    assert_eq!(result.dimension(), expected_dimension);
}
```

## 数値比較

- 浮動小数点の比較には許容誤差（tolerance）を明示
- 相対誤差と絶対誤差の使い分け:
  - 値が十分大きい場合: 相対誤差（例: `assert!((result - expected).abs() / expected.abs() < 1e-10)`）
  - ゼロ近傍: 絶対誤差（例: `assert!((result - expected).abs() < 1e-14)`）
- テスト用のアサーションヘルパーを `types` クレートに用意することを検討

## コマンド

```bash
# 全テスト実行
cargo test

# 特定クレートのテスト
cargo test -p dugong-types

# MPI 並列テスト（将来）
mpirun -np 4 cargo test -p dugong-fields -- --test-threads=1
```

---
_パターンと方針に焦点。ツール固有の設定は別途管理_
