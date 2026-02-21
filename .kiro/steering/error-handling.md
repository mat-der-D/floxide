# エラーハンドリング標準

## 方針

- **早期失敗**: 不正な入力は可能な限り早く（理想的にはコンパイル時に）検出
- **型システム活用**: 不正な状態を表現不可能にする（typestate, newtype, enum）
- **`Result` 中心**: 回復可能なエラーは `Result<T, E>` で伝搬。`panic!` は回復不能な論理エラーのみ

## エラーの分類

### コンパイル時検出（型システムで防止）
- 次元の不整合（`Dim<V, M, L, T>` の const generics）
- 境界条件未評価のフィールド使用（`Fresh`/`Stale` typestate）
- 陰的/陽的演算子の取り違え（`ImplicitOps`/`ExplicitOps` の型分離）
- テンソルランクの不正な昇降（`HasGrad`/`HasDiv` の associated type）

### 実行時エラー（`Result` で伝搬）
- メッシュ読み込み失敗（ファイル不在、フォーマット不正）
- 設定ファイルのパースエラー
- ソルバーの非収束
- MPI 通信エラー

### パニック（回復不能）
- 内部不変条件の違反（`unreachable!()`, `assert!`）
- メモリ割り当て失敗
- 初期化順序の論理エラー

## エラー型の設計パターン

### クレートごとのエラー enum

```rust
// 各クレートが自身のエラー型を定義
#[derive(Debug, thiserror::Error)]
enum MeshError {
    #[error("メッシュファイルの読み込みに失敗: {path}")]
    FileNotFound { path: PathBuf },
    #[error("不正なメッシュトポロジ: {reason}")]
    InvalidTopology { reason: String },
}
```

### エラー伝搬の原則

1. **クレート境界**: 下位クレートのエラーは上位で `From` 変換するか、コンテキストを付加
2. **ライブラリ → アプリケーション**: ライブラリは `Result` を返す。`unwrap()` はアプリケーション層でのみ
3. **`?` 演算子**: エラー伝搬の標準手段。チェーンが深くなりすぎる場合は `context()` で情報付加

## ソルバー収束のハンドリング

```rust
// ソルバーは収束結果を構造体で返す
struct SolveResult {
    converged: bool,
    iterations: usize,
    final_residual: f64,
    initial_residual: f64,
}
```

- 非収束は `Err` ではなく `SolveResult` の `converged: false` で表現
- 呼び出し側が非収束時の挙動（警告して継続 / 中断）を決定
- OpenFOAM の `solverPerformance` に相当

## MPI エラーの扱い

- MPI エラーは通常プログラムを abort する（MPI の標準動作）
- `rsmpi` の panic を捕捉しようとしない — MPI の部分的失敗からの回復は現実的でない
- 初期化時の `Threading::Funneled` 非対応等の検出のみ `Result` で処理

---
_パターンと方針に焦点。具体的なエラーコードの列挙ではない_
