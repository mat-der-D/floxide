# Research & Design Decisions

---
**Purpose**: ディスカバリフェーズで得た知見・アーキテクチャ調査結果・設計根拠を記録する。

---

## Summary
- **Feature**: `types-field-value`
- **Discovery Scope**: Extension（既存 `dugong-types` クレートへの trait 追加）
- **Key Findings**:
  - 既存テンソル型（`Vector`, `Tensor`, `SymmTensor`, `SphericalTensor`）はすべて `Copy + Add + Sub + Mul<f64> + Neg` を実装済み。`FieldValue` のスーパートレイトバウンドは追加実装なしに満たされる。
  - `f64` も `std::ops` 経由で同バウンドを完全に満たすため、実装コードはゼロ。スーパートレイトのみで充足。
  - 新規外部依存は不要。`std::ops` のみを使用する完全にノーコストな拡張。

## Research Log

### 既存テンソル型の trait バウンド確認
- **Context**: `FieldValue` のスーパートレイトバウンド `Copy + Add<Output=Self> + Sub<Output=Self> + Mul<f64, Output=Self> + Neg<Output=Self>` が全テンソル型で満たされるかを確認する必要があった。
- **Sources Consulted**: `crates/types/src/tensor/ops.rs`、`crates/types/src/tensor/types.rs`
- **Findings**:
  - `Vector`: `#[derive(Copy)]` + `ops.rs` で `Add`, `Sub`, `Neg`, `Mul<f64>` 実装済み ✓
  - `Tensor`: `#[derive(Copy)]` + `ops.rs` で `Add`, `Sub`, `Neg`, `Mul<f64>` 実装済み ✓
  - `SymmTensor`: `#[derive(Copy)]` + `ops.rs` で `Add`, `Sub`, `Neg`, `Mul<f64>` 実装済み ✓
  - `SphericalTensor`: `#[derive(Copy)]` + `ops.rs` で `Add`, `Sub`, `Neg`, `Mul<f64>` 実装済み ✓
  - `f64`: `Copy` + `std::ops` で同バウンドをすべて満たす ✓
- **Implications**: `FieldValue` 実装時に追加 `impl` は不要。`zero()` と `mag()` の2メソッドのみ提供すれば実装が成立する。

### モジュール構成パターンの確認
- **Context**: `rust-practices.md` の「`mod.rs` 禁止」ルールに従うモジュール構成を決定する必要があった。
- **Sources Consulted**: `.kiro/steering/rust-practices.md`、`crates/types/src/tensor.rs`
- **Findings**:
  - 既存の `tensor` モジュールは `tensor.rs` + `tensor/` ディレクトリ構成（`mod.rs` 不使用）。
  - `traits` モジュールも同パターンに従う: `crates/types/src/traits.rs` がエントリポイント、`crates/types/src/traits/` 配下にサブモジュールファイルを置く。
- **Implications**: `traits/mod.rs` は使用禁止。`traits.rs` がサブモジュール宣言と再エクスポートの起点となる。

### `SymmTensor` の Frobenius ノルム計算
- **Context**: 要件 2.4 に「対角外成分が 2 回現れることを考慮した Frobenius ノルム」と明記されているため、数学的正確さを確認した。
- **Findings**:
  - `SymmTensor` は上三角成分 `[xx, xy, xz, yy, yz, zz]` の 6 成分で格納。
  - フルテンソルとしては 9 成分（`xy = yx`、`xz = zx`、`yz = zy`）。
  - Frobenius ノルム = `√(xx² + yy² + zz² + 2·xy² + 2·xz² + 2·yz²)`
  - これは `Tensor` の Frobenius ノルム `√(Σᵢⱼ aᵢⱼ²)` と対称テンソルとして一致する。
- **Implications**: `SymmTensor::mag()` の実装では対角外成分に係数 2 を乗じた上で平方根を取る。

### `SphericalTensor` の Frobenius ノルム計算
- **Context**: 要件 2.5 に `mag() = √3 · |s|` と明記されているが、数学的導出を確認した。
- **Findings**:
  - `SphericalTensor(s)` は `s·I`（3×3 スカラー倍単位テンソル）を表す。
  - Frobenius ノルム = `√(s² + 0 + 0 + 0 + s² + 0 + 0 + 0 + s²)` = `√(3s²)` = `√3 · |s|`
- **Implications**: 要件の式は数学的に正確。`SphericalTensor::mag()` は `(3.0_f64).sqrt() * self.value().abs()` で実装できる。

### `HasDiv` での `SymmTensor` の扱い
- **Context**: 要件 4.4 に `SymmTensor::DivOutput = Vector` が明記されている。`SymmTensor` が `HasDiv` を実装するが `HasGrad` は実装しない点を確認。
- **Findings**:
  - `SymmTensor` は対称応力テンソルを表すため、その発散（div）は運動量方程式の拡散項として `Vector` を生成する。CFD の物理的観点から妥当。
  - `SymmTensor` の grad は数学的に定義されないわけではないが、本フレームワークでは Spec 対象外。
- **Implications**: `HasDiv for SymmTensor` のみ実装し、`HasGrad` は実装しない。

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| 単一ファイル (`traits.rs`) | 3 trait を 1 ファイルに集約 | ファイル数が少ない | ファイルが肥大化する可能性がある | 要件 5.1 でファイル分離が明示指定されているため不採用 |
| ファイル分離 (`field_value.rs`, `has_grad.rs`, `has_div.rs`) | 各 trait を独立ファイルに分離 | 単一責務・変更局所性・要件準拠 | 特になし | **採用**: 要件 5.1・`rust-practices.md` のモジュール規約に完全合致 |

## Design Decisions

### Decision: `FieldValue` を object-safe にしない設計
- **Context**: `dyn FieldValue` での使用は現時点では不要。将来的にも `dyn` 化する場合は `HasGrad`/`HasDiv` と組み合わせるが、これらは associated type を持つため object-safe ではない。
- **Alternatives Considered**:
  1. object-safe 化 — `zero()` を `fn zero(&self) -> Self` とする
  2. 静的ディスパッチのみ — `fn zero() -> Self` のまま（クレートのコンパイル時型選択方針に一致）
- **Selected Approach**: 静的ディスパッチのみ（選択肢 2）
- **Rationale**: `rust-practices.md` に「`dyn Trait` は実行時選択の境界のみ」と明記。フィールド演算は静的ディスパッチで行う。
- **Trade-offs**: `dyn FieldValue` は使用不可。ただしこれは意図した制約。
- **Follow-up**: フィールドクレートでの使用時に型推論が正しく動作することを確認。

### Decision: `compile_fail` テストによる型安全性の検証
- **Context**: 要件 3.5・4.6 で「`HasGrad`/`HasDiv` を実装しない型を使用した場合のコンパイルエラー」を `compile_fail` テストで検証することが求められている。
- **Alternatives Considered**:
  1. `compile_fail` テスト（`#[doc = "```compile_fail"]` または `trybuild` crate）
  2. 手動確認のみ
- **Selected Approach**: `#[cfg(test)]` モジュール内での `compile_fail` doctests または標準テストとして記述
- **Rationale**: 型安全性のコンパイル時保証は Rust らしい検証手段。標準的なパターン。
- **Trade-off**: `trybuild` は依存追加が必要なため、標準的な `compile_fail` doctest を優先する。

## Risks & Mitigations
- `f64` への `FieldValue` 実装が `std` の `Mul<f64> for f64` と競合しないか — 競合しない。`FieldValue` は新規 trait であり既存 impl には影響なし。
- `compile_fail` doctest の CI 安定性 — stable Rust コンパイラのエラーメッセージに依存しないため安定。
- `SymmTensor::mag()` の係数 2 の誤記リスク — 数値テストで `zero().mag() < 1e-14` を確認することで検出可能。

## References
- `crates/types/src/tensor/ops.rs` — 既存テンソル型の `std::ops` 実装確認
- `crates/types/src/tensor/types.rs` — 既存テンソル型の定義確認
- `.kiro/steering/rust-practices.md` — モジュール構成・trait 設計原則
- `.kiro/steering/testing.md` — テスト構成・数値比較の精度方針
