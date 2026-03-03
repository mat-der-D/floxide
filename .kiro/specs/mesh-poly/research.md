# リサーチ & 設計決定ログ

## サマリー
- **機能**: `mesh-poly`（PolyMesh レイヤー）
- **ディスカバリ範囲**: Extension（既存 `PrimitiveMesh` の上位レイヤー構築）
- **主要な知見**:
  - `PrimitiveMesh` は OnceLock による遅延計算パターンを採用しており、PolyMesh はこれを所有する形で構成する
  - OpenFOAM の境界面規約（内部面が先、境界面が後）が既に `PrimitiveMesh` に実装済み
  - `MeshError` は `thiserror` ベースで 3 バリアントが存在し、パッチ関連バリアントの追加が必要

## リサーチログ

### PrimitiveMesh の拡張ポイント分析

- **コンテキスト**: PolyMesh が PrimitiveMesh をどのように統合するかの設計判断
- **参照ソース**: `crates/mesh/src/primitive_mesh.rs`, `crates/mesh/src/lib.rs`
- **知見**:
  - `PrimitiveMesh` は `points`, `faces`, `owner`, `neighbor`, `n_cells` をコアデータとして持ち、幾何量（`cell_centers`, `cell_volumes`, `face_centers`, `face_areas`）と接続情報（`cell_cells`, `cell_faces`, `cell_points`）を `OnceLock` で遅延計算する
  - 公開 API は `PrimitiveMesh` と `MeshError` のみ。`geometry` モジュールは `pub(crate)` で非公開
  - テストユーティリティ（`make_unit_cube_mesh`, `make_two_cell_mesh`）は `#[cfg(test)]` 内のプライベート関数で、外部クレートから利用不可
  - `Vector` 型（`dugong_types::tensor::Vector`）が座標とベクトルの両方に使用される（`Point` 型は存在しない）
- **設計への影響**:
  - PolyMesh は `PrimitiveMesh` を所有し、全アクセサを委譲メソッドで公開する
  - 継承ではなくコンポジション（Rust に継承はない）
  - テストユーティリティは `PolyMesh` 用にも必要（直交格子生成ユーティリティ）

### OpenFOAM のパッチ設計との対応

- **コンテキスト**: PolyPatch trait 階層の設計根拠
- **参照ソース**: OpenFOAM の `polyPatch`, `coupledPolyPatch`, `cyclicPolyPatch`, `processorPolyPatch` クラス階層
- **知見**:
  - OpenFOAM では `polyPatch` が基底クラスで、`coupledPolyPatch` が結合パッチのサブクラス
  - 結合パッチ（`coupled`）は隣接セル情報を持つ特殊パッチ（周期境界、プロセッサ境界）
  - 非結合パッチ: `wall`, `empty`, `symmetryPlane`, `wedge`
  - 結合パッチ: `cyclic`, `processor`
  - OpenFOAM では仮想関数で `movePoints()` フックを提供
- **設計への影響**:
  - `PolyPatch` trait + `CoupledPatch` サブ trait の2層構造で OpenFOAM の継承階層を表現
  - オブジェクト安全性の確保により `Box<dyn PolyPatch>` でパッチリストを保持
  - `as_coupled()` メソッドでダウンキャストパターンを提供

### パッチ面範囲の検証ロジック

- **コンテキスト**: PolyMesh 構築時のパッチ整合性検証
- **参照ソース**: OpenFOAM の `polyBoundaryMesh` 検証ロジック
- **知見**:
  - OpenFOAM では境界面（`n_internal_faces..n_faces`）がパッチに分割される
  - 各パッチは連続する面範囲を持ち、`start` と `size` で定義される
  - パッチの面範囲は重複なく、隙間なく境界面全体をカバーする必要がある
  - パッチの順序は `start` の昇順
- **設計への影響**:
  - 構築時にパッチ面範囲の整合性を検証する
  - エラーバリアント: `PatchFaceRangeMismatch`（境界面範囲と不一致）、`PatchFaceOverlapOrGap`（重複・欠落）

### ゾーン型の設計

- **コンテキスト**: cellZone / faceZone / pointZone の統一的な表現
- **参照ソース**: OpenFOAM の `cellZone`, `faceZone`, `pointZone`
- **知見**:
  - `cellZone` と `pointZone` は同一構造（名前 + インデックスリスト）
  - `faceZone` は追加で `flipMap`（各面の法線方向反転フラグ）を持つ
  - ゾーンは名前でアクセスされる
- **設計への影響**:
  - `Zone`（名前 + `Vec<usize>`）を cellZone / pointZone で共有
  - `FaceZone`（名前 + `Vec<usize>` + `Vec<bool>` flip map）を別型として定義

### GlobalMeshData の設計

- **コンテキスト**: 並列計算時のグローバルメッシュ情報
- **参照ソース**: OpenFOAM の `globalMeshData`
- **知見**:
  - グローバルセル数、ポイント数、面数などの合計値を保持
  - シリアル実行時は不要（`Option` で表現）
  - MPI の情報（ランク数等）は保持しない（MPI レイヤーの責務）
- **設計への影響**:
  - `GlobalMeshData` はシンプルな構造体として設計
  - `PolyMesh` 内で `Option<GlobalMeshData>` として保持

## アーキテクチャパターン評価

| オプション | 説明 | 利点 | リスク / 制限 | 備考 |
|-----------|------|------|-------------|------|
| コンポジション（選択） | PolyMesh が PrimitiveMesh を所有し委譲 | Rust の所有権モデルに適合、明確な責務分離 | 委譲メソッドのボイラープレート | OpenFOAM の継承を Rust のコンポジションで再表現 |
| Deref パターン | `Deref<Target=PrimitiveMesh>` で自動委譲 | ボイラープレート削減 | アンチパターン（「Deref 多態性」）、暗黙的な API 公開 | Rust コミュニティで非推奨 |
| trait ベース抽象化 | 共通 Mesh trait を定義 | 柔軟性が高い | 過度な抽象化、この段階では不要 | 将来必要になれば追加可能 |

## 設計決定

### 決定: PolyMesh のコンポジション構造

- **コンテキスト**: PolyMesh が PrimitiveMesh の機能をどう公開するか
- **代替案**:
  1. コンポジション + 手動委譲メソッド
  2. `Deref<Target=PrimitiveMesh>` による自動委譲
  3. 共通 trait による抽象化
- **選択**: オプション 1（コンポジション + 手動委譲）
- **根拠**: Rust のベストプラクティスに従い、明示的な API を提供。`Deref` 多態性は Rust コミュニティでアンチパターンとされる。委譲メソッドは多いが一度書けば安定する
- **トレードオフ**: ボイラープレートコードが増えるが、API の明確性と型安全性を優先
- **フォローアップ**: マクロによる委譲の自動生成は将来検討（現時点では手動で十分）

### 決定: PolyPatch のオブジェクト安全 trait 設計

- **コンテキスト**: 異なるパッチ種別を統一的に扱う仕組み
- **代替案**:
  1. trait オブジェクト（`Box<dyn PolyPatch>`）
  2. enum によるバリアント列挙
- **選択**: オプション 1（trait オブジェクト）
- **根拠**: OpenFOAM の設計思想を継承しつつ、将来のパッチ種別追加に対してオープン。enum は閉じた型であり、ユーザー定義パッチ種別の追加を妨げる
- **トレードオフ**: 動的ディスパッチのオーバーヘッドがあるが、パッチ操作はホットパスではないため無視可能
- **フォローアップ**: `as_coupled()` によるダウンキャストパターンの ergonomics を実装時に検証

### 決定: Specification / Factory 分離による構築パターン

- **コンテキスト**: CoupledPatch（CyclicPolyPatch, ProcessorPolyPatch）の `neighbor_cell_centers` は構築後に setter で設定する設計だったが、これは「構築〜setter 呼び出し」間で不変条件（`neighbor_cell_centers().len() == size()`）が破れる半初期化状態を許容していた。Rust の「不正な状態を表現不可能にする」原則に反する
- **代替案**:
  1. Specification / Factory 分離 — `PatchSpec` enum で仕様を定義し、`PolyMesh::new` 内で全データが揃った状態で具象パッチを構築
  2. Builder パターン — `PolyMeshBuilder` が仕様を蓄積し、`build()` で全バリデーション
  3. 二層型システム — `UnlinkedPatch` / `LinkedPatch` trait で構築段階を型レベル表現
- **選択**: オプション 1（Specification / Factory 分離）
- **根拠**: 最もシンプルかつ Rust らしい。`PatchSpec` は enum（閉じた型）なのでパース・バリデーションが容易。MPI 通信ロジックがパッチオブジェクトから完全に分離される。半初期化状態が型レベルで存在不可能になる
- **トレードオフ**: `PatchSpec` enum と具象パッチ型の二重定義が生じる。enum の拡張性が trait オブジェクトより低い。ただし CFD のパッチ種別は安定しており、将来必要になれば `PatchSpec::Custom(Box<dyn CustomPatchSpec>)` で拡張可能
- **フォローアップ**: なし

### 決定: Transform 型の設計

- **コンテキスト**: CyclicPolyPatch の幾何的変換情報の表現
- **代替案**:
  1. enum（`Translational(Vector)` / `Rotational { axis, angle, center }`）
  2. trait オブジェクト
  3. アフィン変換行列
- **選択**: オプション 1（enum）
- **根拠**: 変換の種類は有限（並進・回転）で拡張の必要性が低い。enum は値型として効率的かつパターンマッチで安全に分岐可能
- **トレードオフ**: 新しい変換種別の追加にはコード変更が必要だが、CFD における周期境界の変換は並進と回転のみ
- **フォローアップ**: なし

## リスク & 緩和策

- **委譲メソッドの保守コスト** — PrimitiveMesh に新しいアクセサが追加された場合、PolyMesh にも追加が必要。緩和: 将来的にマクロ化を検討
- **オブジェクト安全性の制約** — PolyPatch trait にジェネリックメソッドを追加できない。緩和: associated type と具象メソッドで対応
- **パッチ面範囲検証の複雑性** — 不正なパッチ構成のエラーメッセージが分かりにくくなる可能性。緩和: 具体的な不整合箇所を含むエラーバリアントを設計

## 参考

- OpenFOAM ソースコード: `polyMesh`, `polyPatch`, `polyBoundaryMesh`, `globalMeshData` クラス群
- Rust API ガイドライン: [Deref 多態性の回避](https://rust-lang.github.io/api-guidelines/predictability.html)
- `dugong-mesh` 既存実装: `crates/mesh/src/primitive_mesh.rs`
