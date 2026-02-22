# 技術スタック

## アーキテクチャ

レイヤードアーキテクチャのモジュラー設計。Cargo ワークスペースで複数クレートに分離し、依存方向は上位（基盤型）から下位（アプリケーション）への一方向のみ。循環依存は禁止。

## コア技術

- **言語**: Rust (Edition 2024)
- **ビルドシステム**: Cargo ワークスペース
- **並列化**: rsmpi (MPI) + rayon (スレッド並列)
- **実行時型選択**: `inventory` crate + `dyn Trait`
- **シリアライズ**: serde

## 主要ライブラリ

- **`inventory`**: ファクトリの分散登録（リンク時解決）による実行時型選択
- **`rsmpi`**: MPI バインディング。`Threading::Funneled` でメインスレッドのみ MPI 通信
- **`rayon`**: ノード内スレッド並列（予定）
- **`serde`**: I/O・設定ファイルのシリアライズ/デシリアライズ

## 開発標準

### 型安全性
- `typenum` 型レベル整数による次元検査（`Dim<V, M, L, T>`）
- typestate パターンによる状態遷移の型レベル保証（`Fresh`/`Stale`）
- trait bounds で演算の合法性をコンパイル時検証
- `Quantity` トレイト: 次元付き型（`Dim<V, M, L, T>`）から内部値型（`type Value = V`）を公開し、`fvm` 演算子が `T: Quantity where T::Value: FieldValue` として次元付きフィールドを受け取る際の層間接続インターフェース

### コード品質
- `cargo clippy` による静的解析
- `cargo fmt` によるフォーマット統一

### テスト
- `cargo test` による単体テスト
- 各クレート単位でのテスト

## 開発環境

### 必須ツール
- Rust toolchain (Edition 2024 対応)
- MPI 実装（OpenMPI 等）— 並列ビルド時

### 共通コマンド
```bash
# ビルド: cargo build
# テスト: cargo test
# リリースビルド: cargo build --release
# チェック: cargo clippy
```

## 主要な技術的決定

1. **dlopen 不採用**: `unsafe` 回避と LTO 最適化のため全コードを静的リンク
2. **`inventory` による実行時選択**: OpenFOAM の `runTimeSelectionTable` に相当。文字列からオブジェクト生成
3. **objectRegistry の廃止**: OpenFOAM の文字列ベースのレジストリを Rust の型システム（依存性注入・typestate）で代替
4. **ライフタイムによるメッシュ参照**: `Arc` ではなく `&'mesh FvMesh` でフィールドがメッシュを借用
5. **`fvm`/`fvc` のコンテキストオブジェクト方式**: スキーム設定を保持する `ImplicitOps`/`ExplicitOps` に演算子メソッドを定義

---
_標準とパターンを記録。全依存を列挙しない_
