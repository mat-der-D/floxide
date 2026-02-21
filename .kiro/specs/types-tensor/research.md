# Research & Design Decisions

## Summary
- **Feature**: `types-tensor`
- **Discovery Scope**: New Feature（グリーンフィールド）
- **Key Findings**:
  - 外部依存なし — 純粋な Rust 数学型ライブラリとして実装可能
  - newtype パターンと `Copy` セマンティクスが steering で既に規定済み
  - `SymmTensor` の 6 成分格納順（Voigt 記法）が設計上の主要決定事項

## Research Log

### テンソル型の内部表現
- **Context**: CFD で使用するテンソル型の成分格納順序と内部配列サイズの確認
- **Sources Consulted**: OpenFOAM ソースコード（symmTensor, tensor）、Voigt 記法の標準
- **Findings**:
  - `Tensor`（3×3）: 9 成分、row-major 順（`[xx, xy, xz, yx, yy, yz, zx, zy, zz]`）
  - `SymmTensor`（対称 3×3）: 6 独立成分、上三角 row-major 順（`[xx, xy, xz, yy, yz, zz]`）— OpenFOAM と同じ格納順
  - `SphericalTensor`: 単一 `f64` 値（`ii/3` を表す）— `SphericalTensor(s)` は対角成分が全て `s` の球面テンソルを表す
  - `Vector`: 3 成分（`[x, y, z]`）
  - `Scalar`: `f64` の型エイリアス
- **Implications**: 格納順は全ての演算実装の基盤となるため、早期に固定する必要がある

### Copy セマンティクスの妥当性
- **Context**: テンソル型が `Copy` を実装すべきかの検討
- **Sources Consulted**: steering/rust-practices.md、Rust パフォーマンスガイドライン
- **Findings**:
  - 最大サイズは `Tensor` の 72 バイト（`[f64; 9]`）— スタック上で `Copy` するのに十分小さい
  - CFD の内部ループで頻繁にコピーされるため、`Clone` の明示呼び出しは冗長
  - steering で「`Copy` 型はスタック割り当て」と明記済み
- **Implications**: 全テンソル型に `Copy` を derive する方針で問題なし

### 演算子オーバーロード戦略
- **Context**: Rust の `std::ops` trait を用いた演算子実装パターン
- **Findings**:
  - `Copy` 型のため、`Add<Self>` は値渡し（参照渡し不要）
  - 左スカラー倍（`f64 * Vector`）には `impl Mul<Vector> for f64` が必要
  - 異型間演算の `Output` 型はテンソルランクの昇格規則に従う
  - `AddAssign` / `SubAssign` は `&mut self` で in-place 更新
- **Implications**: 演算子 trait は個別に手動実装。マクロ化は将来の検討事項

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| フラットモジュール | `tensor.rs` に全型を定義 | シンプル、型間の可視性が自然 | ファイルが大きくなる | 初期段階では十分 |
| サブモジュール分割 | `tensor/scalar.rs`, `tensor/vector.rs` 等 | 関心の分離 | 型間の相互参照が煩雑 | 型数が増えた場合に検討 |
| **選択: サブモジュール分割** | 型定義・同型演算・異型間演算・変換をファイル分離 | 並列実装可能、diff の競合回避 | `pub use` による再エクスポートが必要 | タスク並列化に有利 |

## Design Decisions

### Decision: SymmTensor の格納順序
- **Context**: 6 独立成分の格納順は演算実装に直結する
- **Alternatives Considered**:
  1. 上三角 row-major（`[xx, xy, xz, yy, yz, zz]`）— OpenFOAM 方式
  2. Voigt 記法（`[xx, yy, zz, xy, xz, yz]`）— FEM 系で一般的
- **Selected Approach**: 上三角 row-major（OpenFOAM 方式）
- **Rationale**: CFD フレームワークとして OpenFOAM との概念的互換性を重視。Voigt 記法は応力テンソルの FEM 表現に最適化されており、CFD の一般テンソル演算には不自然
- **Trade-offs**: FEM ライブラリとの相互運用時に変換が必要になる可能性があるが、現時点では非目標
- **Follow-up**: `From<SymmTensor> for Tensor` の展開時に格納順マッピングを正確にテストする

### Decision: SphericalTensor の意味
- **Context**: `SphericalTensor` が表す値の定義
- **Alternatives Considered**:
  1. スカラー値 `s` を保持し、対角テンソル `sI` を表す（OpenFOAM 方式）
  2. トレースの 1/3 を保持する
- **Selected Approach**: スカラー値 `s` を保持し、対角テンソル `sI` を表す
- **Rationale**: OpenFOAM の `sphericalTensor` と同じ意味。`sph()` が `SphericalTensor(trace/3)` を返し、`From<SphericalTensor> for Tensor` は `diag(s, s, s)` を生成する
- **Trade-offs**: なし — CFD の標準的な定義

### Decision: モジュール構造
- **Context**: `tensor` モジュールのファイル分割方針
- **Selected Approach**: サブモジュール分割（`tensor/` ディレクトリ）
- **Rationale**: 型定義、同型演算、異型間演算、型変換を独立ファイルに分離し、タスク並列実装を可能にする。`tensor/mod.rs` ではなく `tensor.rs` + `tensor/` ディレクトリの Rust 2018+ パターンを使用
- **Follow-up**: `pub use` による再エクスポートで `dugong_types::tensor::Vector` 等のパスを提供

## Risks & Mitigations
- **浮動小数点精度**: `det()` の計算で桁落ちの可能性 → 既知の入力に対する数値テストで検証
- **SymmTensor 演算の添字ミス**: 6 成分の格納順を間違えやすい → 成分名アクセサとテストで防止
- **演算子 trait の組み合わせ爆発**: 異型間演算が多い → トレーサビリティ表で漏れを防止

## References
- OpenFOAM symmTensor 実装 — 格納順・演算の参照
- Rust `std::ops` ドキュメント — 演算子 trait の実装パターン
- steering/rust-practices.md — newtype パターン、Copy セマンティクス
- steering/cfd-conventions.md — テンソル型の命名規約
