# リサーチ & 設計決定記録: mesh-primitive

---

## サマリー

- **フィーチャー:** `mesh-primitive`
- **ディスカバリスコープ:** 新規機能（グリーンフィールド実装）
- **主な知見:**
  - `std::sync::OnceLock` (Rust 1.70+ stable) が外部クレート不要で最適な選択肢
  - Rust Edition 2024 を使用しているため `OnceLock` は完全に利用可能
  - ジオメトリアルゴリズムは OpenFOAM の `primitiveMeshGeometry.C` に相当するテトラへドロン分解方式を採用
  - 既存 `crates/mesh/Cargo.toml` に `dugong-types` のみが依存として記載されており、外部クレート追加は最小限にする
  - `crates/mesh/src/lib.rs` は実装が空（`// TODO: Implement mesh structure`）であり、本スペックが初期実装対象

---

## リサーチログ

### OnceLock vs OnceCell の選定

- **背景:** `PrimitiveMesh` の遅延計算フィールドには「一度だけ初期化し、以降はキャッシュを返す」パターンが必要。`&self` からの初期化（`&mut self` なし）と `Send + Sync` の両立が必須要件。
- **参照資料:**
  - Rust 標準ライブラリ (`std::sync::OnceLock`) — Rust 1.70.0 (2023-06-01) で stable
  - `once_cell` クレート — Rust 1.70 以前の `OnceCell`/`OnceLock` の先行実装
  - プロジェクト `Cargo.toml` (workspace) — Edition 2024、`once_cell` の記載なし
- **調査結果:**
  - `std::sync::OnceLock<T>` は `T: Send + Sync` のとき `OnceLock<T>` が `Send + Sync` を自動実装
  - `Vec<[f64; 3]>` および `Vec<Vec<usize>>` はいずれも `Send + Sync` を満たす
  - `once_cell::sync::OnceCell` は `OnceLock` の標準化前の外部実装で API はほぼ同等
  - プロジェクトに `once_cell` の依存記載がなく、標準ライブラリで完結することを優先する
- **結論:** `std::sync::OnceLock` を採用。外部クレート依存を追加しない。

### 遅延計算フィールドにおける依存関係の分析

- **背景:** セルジオメトリ（cell_volumes, cell_centers）の計算にはまず面ジオメトリ（face_centers, face_areas）が必要であるが、`OnceLock` のフィールド間依存を安全に扱う方法を検討した。
- **調査結果:**
  - `OnceLock::get_or_init` はクロージャ内で `self` の他の `OnceLock` フィールドを呼び出すことができる
  - `calc_cell_geometry()` の内部で `self.face_centers()` と `self.face_areas()` を呼ぶことは安全（再帰的にキャッシュが初期化されるだけで、デッドロックは起きない）
  - ただし、`cell_centers` と `cell_volumes` の計算ではどちらもピラミッド体積の計算が必要であるため、両者を一つのプライベートメソッドで同時に計算してタプルで返すのが効率的
- **結論:** `calc_cell_geometry(&self) -> (Vec<[f64; 3]>, Vec<f64>)` のようなプライベートメソッドで `cell_centers` と `cell_volumes` をまとめて計算するが、公開 API は個別のアクセサとする。`OnceLock` の初期化ではそれぞれ独立したフィールドに格納する（一方の初期化が他方をトリガーする設計は `get_or_init` 内の再帰呼び出しで実現）。

### OpenFOAM ジオメトリアルゴリズムの分析

- **背景:** 要件 2.5–2.6 および 3.5–3.7 で「OpenFOAM と同等の手法」が明示されているため、アルゴリズムを特定する必要があった。
- **参照資料:** `docs-dev/spec-ideas/mesh_architecture.md`, OpenFOAM `primitiveMeshGeometry.C`
- **調査結果（面ジオメトリ）:**
  - 任意多角形の面中心と面積ベクトルを安定して計算するために、面を参照点（頂点平均）からの三角形列に分解する
  - 参照点を頂点平均とすることで、凸多角形・非凸多角形いずれでも安定した計算が可能
  - 面積ベクトル = 各三角形の面積ベクトルの合計（クロス積の和）
  - 面中心 = 各三角形の面積スカラーで加重した三角形重心の加重平均
  - 向き: `owner` → `neighbor` を正とする（境界面は外向き法線）
- **調査結果（セルジオメトリ）:**
  - 各面の面中心と面積ベクトルをセルの参照点（まず面中心の平均）に対するピラミッドに分解
  - ピラミッド体積 = `face_area_vec · (face_center - cell_ref) / 3`
  - 向きの符号: face が対象セルの `owner` なら `+`, `neighbor` なら `-`（外向き法線の定義による）
  - セル中心 = ピラミッド重心（`(3 * cell_ref + face_center) / 4`）の体積加重平均
- **結論:** 設計書記載のアルゴリズムはこの手法に準拠している。数値精度要件（相対誤差 1e-10）を満たすことを立方体テストで検証する。

### 既存コードベースの調査

- **背景:** `crates/mesh/` の実装状況を確認し、新規ファイルの配置計画を立てる必要があった。
- **調査結果:**
  - `crates/mesh/src/lib.rs` は 5 行のみ（コメント + TODO）で実装なし
  - `crates/mesh/Cargo.toml` の依存は `dugong-types = { path = "../types" }` のみ
  - `dugong-types` には `Dim`, `Velocity`, `Density` 等の型が定義されているが、`PrimitiveMesh` の実装で直接使用する型（`[f64; 3]`, `Vec<usize>` 等）は Rust 標準型で十分
  - Rust Edition 2024 の `mod.rs` 禁止方針（`rust-practices.md`）に従い、`primitive_mesh.rs`, `error.rs`, `geometry.rs` のファイル分割を採用
- **結論:** 新規ファイルを 3 本追加し、`lib.rs` を更新することで実装を開始できる。既存コードとの衝突なし。

### thiserror クレートの採用可否

- **背景:** `MeshError` に `std::error::Error` を実装するため、`thiserror` クレートの導入を検討した。
- **調査結果:**
  - 現在の `Cargo.toml`（ワークスペースおよび `crates/mesh/`）に `thiserror` の記載なし
  - `thiserror` は proc-macro クレートで、`Display` と `std::error::Error` の定型実装を自動化する
  - `MeshError` の各バリアントは単純なフィールド表示であり、手動実装も容易
- **結論:** 実装フェーズで開発者が `thiserror` 導入の可否を判断する。設計書では「`thiserror` を使うか手動実装とするか」の選択肢を明記する。設計上の API（`Debug`, `Display`, `std::error::Error`）は変わらない。

---

## アーキテクチャパターン評価

| オプション | 説明 | 強み | リスク/制限 | 採否 |
|-----------|------|------|------------|------|
| `std::sync::OnceLock` | 標準ライブラリの遅延初期化 | 外部依存なし、stable、`Sync` 保証 | Rust 1.70+ 必須（本PJ は Edition 2024 のため問題なし） | **採用** |
| `once_cell::sync::OnceCell` | 外部クレートの遅延初期化 | API が豊富 | 外部依存追加が必要、`OnceLock` で十分 | 不採用 |
| `Mutex<Option<T>>` | 汎用排他制御 | 任意の型に対応 | ロックオーバーヘッド、コードが冗長 | 不採用 |
| `RwLock<Option<T>>` | 読み書き分離ロック | 読み取り並列性 | ロックオーバーヘッド、`OnceLock` で十分 | 不採用 |

---

## 設計決定

### 決定: `OnceLock` フィールドの配置

- **背景:** 遅延計算の実現手段
- **検討した選択肢:**
  1. `std::sync::OnceLock` — 標準ライブラリ
  2. `once_cell::sync::OnceCell` — 外部クレート
  3. `Mutex<Option<T>>` — 汎用ロック
- **選択:** `std::sync::OnceLock`
- **根拠:** 外部依存を増やさず、`&self` からの一度きりの初期化を安全に提供できる。`mesh_architecture.md` の設計判断とも一致する。
- **トレードオフ:** 動的メッシュ対応時に `OnceLock` をリセットできない（将来 `LazyCache` 型へ交換で対応）
- **フォローアップ:** 動的メッシュスペック（将来）でキャッシュ無効化戦略を定義する

### 決定: `geometry.rs` を独立モジュールに分離

- **背景:** 計算アルゴリズムのテスト可能性と責務分離
- **検討した選択肢:**
  1. `primitive_mesh.rs` 内にプライベートメソッドとして実装
  2. `geometry.rs` に `pub(crate)` 関数として分離
- **選択:** `geometry.rs` に分離
- **根拠:** ジオメトリ計算関数を独立してテスト可能にし、`primitive_mesh.rs` の行数を抑える。将来 `PolyMesh` や `FvMesh` がジオメトリ関数を再利用する可能性もある。
- **トレードオフ:** ファイル数が若干増えるが、責務分離の明確化と引き換えに許容できる

### 決定: `cell_centers` と `cell_volumes` の計算を内部で連携

- **背景:** 両者はピラミッド体積の計算を共有するため、別々に計算すると重複が生じる
- **選択:** プライベートメソッド `calc_cell_geometry()` で両者を同時計算し、それぞれの `OnceLock` に格納する
- **根拠:** 計算効率を保ちつつ、公開 API は個別アクセサとして要件を満たす
- **トレードオフ:** `cell_centers` のみが必要な場合でも `cell_volumes` も計算されるが、いずれも同時に必要になるケースが多いため実用上の問題は小さい

---

## リスクと対策

- **アルゴリズム精度リスク:** 非凸面や縮退面（三角形が一直線の頂点を持つ面等）でジオメトリ計算が不正確になる可能性がある → 単位立方体テストで基本精度を確認し、数値安定性は将来の統合テストで検証
- **依存フィールド間の `OnceLock` デッドロックリスク:** `cell_volumes` 計算内で `face_centers()` を呼ぶと再帰的な `OnceLock` 初期化が発生する → Rust の `OnceLock::get_or_init` はリエントラントではないため、同一スレッドからの同一 `OnceLock` の再帰初期化はデッドロックになる。設計上、`calc_cell_geometry()` では `self.face_centers()` の初期化が完了してから呼ばれることを保証するか、面ジオメトリを直接計算する実装とする（実装フェーズで注意が必要）
- **`thiserror` 依存の有無:** `MeshError` の手動実装でもフォーマット上の問題は発生しないが、コードが冗長になる → 実装フェーズで `thiserror` 追加を検討する

---

## 参考資料

- [Rust std::sync::OnceLock](https://doc.rust-lang.org/std/sync/struct.OnceLock.html) — 標準ライブラリ遅延初期化
- [once_cell クレート](https://docs.rs/once_cell/) — OnceLock の前身。本PJでは不採用
- `docs-dev/spec-ideas/mesh_architecture.md` — 3層メッシュ設計の根拠と設計判断一覧
- `.kiro/steering/rust-practices.md` — モジュール構成、`unsafe` 禁止、`OnceLock` 等の設計規約
- `.kiro/steering/error-handling.md` — `Result` 中心のエラーハンドリング方針
- `.kiro/steering/testing.md` — AAA パターン、数値比較の基準
- OpenFOAM ソース `src/OpenFOAM/meshes/primitiveMesh/primitiveMeshGeometry.C` — ジオメトリアルゴリズムの参照実装
