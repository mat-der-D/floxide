# Requirements Document

## Introduction

本仕様は `dugong-mesh` クレートに **PolyMesh レイヤー** を追加する。`PrimitiveMesh`（Spec 2-1 で実装済み）の上に、パッチ（境界条件の幾何的担体）、ゾーン（セル・面・点のグループ化）、並列メタデータを統合した `PolyMesh` 構造体を構築する。

PolyMesh は後続の `FvMesh`（Spec 2-3）の基盤となり、有限体積法メッシュに必要なパッチ分類と接続情報を提供する。

### スコープ外の明示事項

- **動的メッシュでの隣接セル中心の再交換**: 本スペックでは結合パッチの `neighbor_cell_centers` を構築時に確定し不変とする。動的メッシュの `move_points` で隣接セル中心を再交換する機構（`CoupledPatch` への更新メソッド追加、またはパッチ再構築）は Spec 2-3（FvMesh）で設計する。本スペックの `move_points()` フック（Req 1.5）と `face_cells()` メソッド（Req 1.6）が拡張ポイントとして機能する。
- **GlobalMeshData の共有点情報**: `shared_point_labels` / `shared_point_addressing` 等のプロセッサ間共有点トポロジ情報は本スペックではスコープ外とする。必要に応じて後続スペックで `GlobalMeshData` を拡張する。

## Requirements

### Requirement 1: PolyPatch trait 階層

**Objective:** 開発者として、境界パッチの共通インターフェースと結合パッチのサブインターフェースが欲しい。これにより、壁・周期・プロセッサ等の異なるパッチ種別を統一的に扱える。

#### Acceptance Criteria

1. The dugong-mesh shall `PolyPatch` trait を公開し、パッチ名・開始面インデックス・サイズ・パッチ種別の取得メソッドを提供する
2. The dugong-mesh shall `PolyPatch` trait に `Send + Sync` bounds を要求する
3. The dugong-mesh shall `PolyPatch` trait にオプショナルな `as_coupled()` メソッド（不変参照）を提供し、結合パッチへのダウンキャストを可能にする（デフォルト実装は `None` を返す）
4. The dugong-mesh shall `PolyPatch` trait にオプショナルな `as_coupled_mut()` メソッド（可変参照）を提供し、結合パッチへの可変ダウンキャストを可能にする（デフォルト実装は `None` を返す）
5. The dugong-mesh shall `PolyPatch` trait にデフォルト実装の `move_points()` フックメソッドを提供し、メッシュ点移動時にパッチ種別ごとの更新処理を可能にする（デフォルトは何もしない）
6. The dugong-mesh shall `CoupledPatch` trait を `PolyPatch` のサブ trait として公開し、face-cell マッピング・隣接セル中心・隣接ランク番号・変換情報へのアクセスメソッドを提供する
7. The dugong-mesh shall `CoupledPatch` trait の `neighbor_rank()` メソッドが `Option<i32>` を返し、`ProcessorPolyPatch` は `Some(rank)` を、`CyclicPolyPatch` は `None` を返す
8. The dugong-mesh shall `PolyPatch` trait をオブジェクト安全にし、`Box<dyn PolyPatch>` として使用可能にする
9. The dugong-mesh shall `CoupledPatch` trait をオブジェクト安全にし、`&dyn CoupledPatch` および `&mut dyn CoupledPatch` として使用可能にする

### Requirement 2: パッチ具象型

**Objective:** 開発者として、CFD で一般的な境界パッチ種別の具象実装が欲しい。これにより、壁・周期・プロセッサ分割・対称面等の境界を表現できる。

#### Acceptance Criteria

1. The dugong-mesh shall `WallPolyPatch` 型を提供し、`PolyPatch` を実装する
2. The dugong-mesh shall `CyclicPolyPatch` 型を提供し、`PolyPatch` と `CoupledPatch` の両方を実装する
3. The dugong-mesh shall `ProcessorPolyPatch` 型を提供し、`PolyPatch` と `CoupledPatch` の両方を実装する
4. The dugong-mesh shall `EmptyPolyPatch` 型を提供し、`PolyPatch` を実装する
5. The dugong-mesh shall `SymmetryPolyPatch` 型を提供し、`PolyPatch` を実装する
6. The dugong-mesh shall `WedgePolyPatch` 型を提供し、`PolyPatch` を実装する
7. When `as_coupled()` が `CyclicPolyPatch` に対して呼ばれたとき, the dugong-mesh shall `Some(&dyn CoupledPatch)` を返す
8. When `as_coupled()` が `ProcessorPolyPatch` に対して呼ばれたとき, the dugong-mesh shall `Some(&dyn CoupledPatch)` を返す
9. When `as_coupled()` が非結合パッチ（`WallPolyPatch` 等）に対して呼ばれたとき, the dugong-mesh shall `None` を返す

### Requirement 3: CyclicPolyPatch の変換情報

**Objective:** 開発者として、周期境界パッチが幾何的変換情報（並進・回転）を保持して欲しい。これにより、周期境界での面の対応付けが可能になる。

#### Acceptance Criteria

1. The dugong-mesh shall `Transform` 型を提供し、周期パッチの幾何的変換（並進ベクトルまたは回転パラメータ）を表現する
2. The dugong-mesh shall `CyclicPolyPatch` に変換情報へのアクセスメソッドを提供する
3. The dugong-mesh shall `CyclicPolyPatch` に隣接セル中心の取得メソッドを提供する（隣接セル中心は構築時にコンストラクタ引数として渡される）
4. The dugong-mesh shall `CyclicPolyPatch` に face-cell マッピングの取得メソッドを提供する

### Requirement 4: ProcessorPolyPatch の並列情報

**Objective:** 開発者として、プロセッサ境界パッチが MPI 通信に必要な情報を保持して欲しい。これにより、ドメイン分割後の halo 交換が可能になる。

#### Acceptance Criteria

1. The dugong-mesh shall `ProcessorPolyPatch` に隣接セル中心の取得メソッドを提供する（隣接セル中心は構築時にコンストラクタ引数として渡される）
2. The dugong-mesh shall `ProcessorPolyPatch` に face-cell マッピングへのアクセスメソッドを提供する

### Requirement 5: ゾーン型

**Objective:** 開発者として、セル・面・点のグループ（ゾーン）を名前付きで管理したい。これにより、メッシュの部分領域を選択的に操作できる。

#### Acceptance Criteria

1. The dugong-mesh shall `Zone` 型を提供し、名前とインデックスリストを保持する（cellZone / pointZone 共用）
2. The dugong-mesh shall `FaceZone` 型を提供し、名前・インデックスリスト・flip map を保持する
3. The dugong-mesh shall ゾーンの名前によるアクセスメソッドを提供する
4. The dugong-mesh shall `Zone` および `FaceZone` のインデックスリストへのアクセスメソッドを提供する
5. The dugong-mesh shall `FaceZone` の flip map へのアクセスメソッドを提供する

### Requirement 6: GlobalMeshData（並列トポロジ情報）

**Objective:** 開発者として、並列計算時のグローバルメッシュ情報を保持する構造が欲しい。これにより、全ランクにまたがるメッシュのトポロジ情報にアクセスできる。

#### Acceptance Criteria

1. The dugong-mesh shall `GlobalMeshData` 構造体を提供し、並列トポロジ情報（グローバルセル数、ポイント数等）を保持する
2. The dugong-mesh shall `GlobalMeshData` の各フィールドへのアクセスメソッドを提供する

### Requirement 7: PolyMesh 構造体

**Objective:** 開発者として、`PrimitiveMesh` にパッチ・ゾーン・並列情報を統合した `PolyMesh` が欲しい。これにより、有限体積法に必要なメッシュの全情報を一元管理できる。

#### Acceptance Criteria

1. The dugong-mesh shall `PolyMesh` 構造体を提供し、`PrimitiveMesh`・パッチリスト・ゾーンリスト・旧時刻点情報・グローバルデータを保持する
2. The dugong-mesh shall `PolyMesh` から `PrimitiveMesh` の全アクセサへの委譲メソッドを提供する（`points()`, `faces()`, `owner()`, `neighbor()`, `n_internal_faces()`, `n_cells()`, `n_faces()`, `n_points()`, `cell_volumes()`, `cell_centers()`, `face_centers()`, `face_areas()`, `cell_cells()`, `cell_faces()`, `cell_points()`）
3. The dugong-mesh shall `PolyMesh` からパッチリストへのアクセスメソッドを提供する
4. The dugong-mesh shall `PolyMesh` から `cell_zones`・`face_zones`・`point_zones` の3つの独立したゾーンリストへのアクセスメソッドを提供する
5. The dugong-mesh shall `PolyMesh` から `GlobalMeshData` へのアクセスメソッドを提供する（`Option` 型、シリアル時は `None`）
6. The dugong-mesh shall `PolyMesh` から旧時刻点座標へのアクセスメソッドを提供する（`Option` 型、非動的メッシュ時は `None`）
7. The dugong-mesh shall `PolyMesh` が `Send + Sync` であることを保証する

### Requirement 8: PolyMesh 構築

**Objective:** 開発者として、`PolyMesh` を検証済みのデータから安全に構築したい。これにより、不整合なメッシュの生成を防止できる。

#### Acceptance Criteria

1. The dugong-mesh shall `PatchSpec` enum を提供し、各パッチ種別の構築に必要なパラメータを値型として保持する（面範囲・パッチ名・種別固有パラメータ。結合パッチは face-cell マッピングを含む）
2. The dugong-mesh shall `PolyMesh` のコンストラクタを提供し、`PrimitiveMesh`・パッチ仕様リスト（`Vec<PatchSpec>`）・結合パッチの隣接セル中心マップ・ゾーンリストを受け取る
3. The dugong-mesh shall `PolyMesh` コンストラクタ内で `PatchSpec` から具象パッチ型（`Box<dyn PolyPatch>`）を生成し、結合パッチには隣接セル中心マップから対応データを注入する
4. When パッチの面範囲が `PrimitiveMesh` の境界面範囲と整合しないとき, the dugong-mesh shall エラーを返す
5. When パッチの面範囲が重複または欠落しているとき, the dugong-mesh shall エラーを返す
6. When 結合パッチ仕様に対応する隣接セル中心がマップに存在しないとき, the dugong-mesh shall エラーを返す
7. When 隣接セル中心の要素数がパッチの面数と一致しないとき, the dugong-mesh shall エラーを返す
8. The dugong-mesh shall テスト用に直交格子から `PolyMesh` を生成するユーティリティを提供する

### Requirement 9: エラーハンドリング

**Objective:** 開発者として、PolyMesh 関連のエラーが既存の `MeshError` 型に統合されて欲しい。これにより、mesh クレート全体で一貫したエラー処理ができる。

#### Acceptance Criteria

1. The dugong-mesh shall パッチ関連のエラーバリアントを `MeshError` に追加する（面範囲不整合・隣接セル中心の欠落・長さ不一致）
2. If パッチの面範囲検証が失敗したとき, the dugong-mesh shall 具体的な不整合箇所を含むエラーメッセージを返す
3. If 結合パッチの隣接セル中心が未提供または長さ不一致のとき, the dugong-mesh shall パッチ名と期待サイズを含むエラーメッセージを返す
