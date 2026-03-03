# Rust ベストプラクティス

## 基本方針

- **Rust らしく書く**: C++ や他言語のイディオムをそのまま持ち込まない
- **型システムを最大活用**: 不正な状態を表現不可能にする設計を優先
- **`unsafe` は原則禁止**: `unsafe` が必要な場面は MPI バインディング等の最下層に限定し、安全な抽象で包む

## 所有権と借用

### パターン
- **不変参照を基本とする**: `&self` メソッドを優先。`&mut self` は状態変更が明確な場合のみ
- **ライフタイムパラメータで関連を表現**: `VolumeField<'mesh, T, State>` のように、データの生存期間の関連を型で表現
- **`Arc` / `Rc` よりもライフタイム**: 共有所有権が不要な場合（例: メッシュは不変）、参照カウントではなく借用を使用
- **`Clone` を安易に使わない**: 大きなデータ（フィールド値の `Vec`）のコピーは意図的に行う

### アンチパターン
- `Arc<Mutex<T>>` で全てを解決しようとしない
- ライフタイムを避けるためだけに `'static` にしない
- `.clone()` でコンパイルエラーを消さない

## 型設計

### newtype パターン
```rust
struct Vector([f64; 3]);        // 生配列ではなく newtype
struct SymmTensor([f64; 6]);    // 型の区別で誤用を防止
```

### typestate パターン
```rust
struct Fresh;
struct Stale;
struct VolumeField<'a, T, S> { /* ... */ _state: PhantomData<S> }

// 状態遷移を型で表現 — 不正な遷移はコンパイルエラー
impl VolumeField<'_, T, Stale> {
    fn evaluate_boundaries(self, world: &Communicator) -> VolumeField<'_, T, Fresh> { /* ... */ }
}
```

### 型レベル整数（typenum）
```rust
// typenum::Integer による型レベル次元検査（stable Rust）
use typenum::{Integer, P1, N1, N2, Z0};
struct Dim<V, M: Integer, L: Integer, T: Integer> {
    value: V,
    _phantom: PhantomData<(M, L, T)>,
}
```

## trait 設計

### 原則
- **小さく焦点を絞った trait**: `FieldValue`, `HasGrad`, `HasDiv` のように単一責務
- **associated type でランク変化を表現**: ジェネリックパラメータではなく associated type で出力型を固定
- **`dyn Trait` は実行時選択の境界のみ**: `dyn PhysicalBC<T>`, `dyn LinearSolver` 等
- **オブジェクト安全性を意識**: `dyn` 化する trait はジェネリックメソッドを持たない

### trait bounds
```rust
// 必要最小限の bounds を要求
fn gradient<T: FieldValue + HasGrad>(field: &VolumeField<T, Fresh>) -> Vec<T::GradOutput>
```

## エラーハンドリング

- `Result<T, E>` で回復可能なエラーを伝搬（`?` 演算子）
- `panic!` は内部不変条件の違反にのみ使用
- `unwrap()` は原則アプリケーション層（`apps/`）でのみ許容
- ライブラリ層で `unwrap()` を使用する場合は、論理的に `None`/`Err` が発生し得ないことを示すコメントを必ず添える（例: `OnceLock::get_or_init()` 直後の `get().unwrap()` など、内部不変条件により到達不可能なケース）
- 詳細は `error-handling.md` を参照

## パフォーマンス意識

### コンパイル時
- const generics とジェネリクスで単相化（monomorphization）を活用
- ただし `FvMatrix` のように不要な単相化はコンパイル時間・バイナリサイズの観点で避ける

### 実行時
- `Copy` 型（テンソル等の固定長値）はスタック割り当て
- `Vec<T>` のフィールドデータはヒープ割り当てだが連続メモリ（キャッシュフレンドリー）
- LTO (`lto = true`, `codegen-units = 1`) によるクレート境界を超えたインライン展開

## コーディングスタイル

- **`cargo fmt`**: フォーマットは自動化。議論しない
- **`cargo clippy`**: lint に従う。例外は `#[allow(...)]` で明示的に許可し、理由をコメント
- **ドキュメントコメント**: 公開 API には `///` を付ける。内部実装の自明なコードには不要
- **モジュール構成**: `mod.rs` ではなくファイル名でモジュールを定義（Rust 2018+ スタイル）。
  サブモジュールを持つモジュール `foo` は `foo/mod.rs` ではなく `foo.rs` + `foo/` ディレクトリで構成する。
  ```
  src/
    lib.rs          ← `pub mod tensor;`
    tensor.rs       ← tensor モジュールの定義（サブモジュール宣言・再エクスポート）
    tensor/         ← tensor のサブモジュール群
      types.rs
      ops.rs
      ...
  ```
  `foo/mod.rs` 方式は使用禁止。`foo.rs` がモジュールのエントリポイントとなり、
  同名の `foo/` ディレクトリ内にサブモジュールファイルを配置する。

---
_パターンと原則に焦点。言語リファレンスの再掲ではない_
