# Implementation Plan

- [ ] 1. 基盤型の実装（エラー型・データ型）
- [ ] 1.1 (P) MeshError にパッチ関連エラーバリアントを追加する
  - 面範囲不整合、面範囲の重複・欠落、隣接セル中心の欠落、隣接セル中心の長さ不一致の 4 バリアントを追加する
  - 各バリアントに具体的な不整合箇所（パッチ名・範囲・期待サイズ等）を含む構造化エラー情報を持たせる
  - 既存の 3 バリアント（OwnerLengthMismatch, NeighborIndexOutOfRange, PointIndexOutOfRange）は変更しない
  - _Requirements: 9.1, 9.2, 9.3_

- [ ] 1.2 (P) PatchKind enum と Transform 型を実装する
  - パッチ種別を識別する PatchKind enum（Wall, Cyclic, Processor, Empty, Symmetry, Wedge）を定義する
  - 周期パッチの幾何的変換を表現する Transform enum を定義する（並進：分離ベクトル、回転：軸・角度・中心）
  - 両型とも値型として振る舞い、Debug・Clone・PartialEq を導出する
  - _Requirements: 3.1_

- [ ] 1.3 (P) Zone と FaceZone 型を実装する
  - 名前付きインデックスグループ Zone 型を実装する（セルゾーン・ポイントゾーンで共用）
  - 名前・インデックスリスト・flip map を保持する FaceZone 型を実装する
  - 各型に名前・インデックスリストへのアクセサを提供する
  - FaceZone には flip map アクセサを追加提供する
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5_

- [ ] 1.4 (P) GlobalMeshData 構造体を実装する
  - 並列トポロジ情報（グローバルセル数・ポイント数・面数・内部面数）を保持する構造体を定義する
  - 各フィールドへのアクセサメソッドを提供する
  - _Requirements: 6.1, 6.2_

- [ ] 2. (P) PatchSpec enum を実装する
  - 6 種のパッチ仕様バリアント（Wall, Cyclic, Processor, Empty, Symmetry, Wedge）を定義する
  - 各バリアントにパッチ名・開始面インデックス・サイズを持たせる。結合パッチ（Cyclic, Processor）には face-cell マッピングと種別固有パラメータを追加する
  - 全バリアント共通のアクセサ（name, start, size）と結合判定メソッド（is_coupled）を実装する
  - Task 1.2 の Transform 型を Cyclic バリアントで使用する
  - _Requirements: 8.1_

- [ ] 3. (P) PolyPatch trait 階層を定義する
  - PolyPatch trait を定義する：パッチ名・開始面インデックス・サイズ・種別の取得メソッドを必須とする
  - Send + Sync を trait bounds に要求する
  - as_coupled() / as_coupled_mut() のデフォルト実装（None を返す）を提供する
  - move_points() フックのデフォルト実装（no-op）を提供する
  - CoupledPatch trait を PolyPatch のサブ trait として定義する：face-cell マッピング・隣接セル中心・隣接ランク番号・変換情報へのアクセスメソッドを必須とする
  - 両 trait ともオブジェクト安全にし、Box\<dyn PolyPatch\> および &dyn CoupledPatch として使用可能にする
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 1.8, 1.9_

- [ ] 4. パッチ具象型の実装
- [ ] 4.1 (P) 非結合パッチ型 4 種を実装する
  - WallPolyPatch, EmptyPolyPatch, SymmetryPolyPatch, WedgePolyPatch を実装する
  - 各型は名前・開始面インデックス・サイズを保持し、PolyPatch trait を実装する
  - as_coupled() は全てデフォルト実装（None）を使用する
  - kind() でそれぞれ対応する PatchKind を返す
  - _Requirements: 2.1, 2.4, 2.5, 2.6, 2.9_

- [ ] 4.2 (P) CyclicPolyPatch を実装する
  - PolyPatch と CoupledPatch の両方を実装する
  - Transform による変換情報・face-cell マッピング・隣接セル中心を保持する
  - as_coupled() は Some を返し、neighbor_rank() は None を返す
  - コンストラクタで全フィールドを受け取り、構築後は不変とする
  - _Requirements: 2.2, 2.7, 3.2, 3.3, 3.4_

- [ ] 4.3 (P) ProcessorPolyPatch を実装する
  - PolyPatch と CoupledPatch の両方を実装する
  - 隣接ランク番号（i32）・face-cell マッピング・隣接セル中心を保持する
  - as_coupled() は Some を返し、neighbor_rank() は Some(rank) を返す。transform() は None を返す
  - コンストラクタで全フィールドを受け取り、構築後は不変とする
  - _Requirements: 2.3, 2.8, 4.1, 4.2_

- [ ] 5. PolyMesh 構造体の実装
- [ ] 5.1 PolyMesh 構造体を定義し PrimitiveMesh のアクセサを委譲する
  - PrimitiveMesh・パッチリスト・3 種ゾーンリスト・旧時刻点情報・グローバルデータを所有する構造体を定義する
  - PrimitiveMesh の全アクセサ（points, faces, owner, neighbor, n_internal_faces, n_cells, n_faces, n_points, cell_volumes, cell_centers, face_centers, face_areas, cell_cells, cell_faces, cell_points）への委譲メソッドを提供する
  - パッチリスト・ゾーンリスト・グローバルデータ・旧時刻点へのアクセサを提供する
  - global_data と old_points のセッターメソッドを提供する
  - Send + Sync をコンパイル時に保証する（静的アサーション）
  - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5, 7.6, 7.7_

- [ ] 5.2 PolyMesh の構築ロジックとバリデーションを実装する
  - PrimitiveMesh・PatchSpec リスト・隣接セル中心マップ・ゾーンリストを受け取るコンストラクタを実装する
  - PatchSpec を start の昇順にソートし、面範囲の整合性を検証する（境界面範囲との一致、重複・欠落の検出）
  - 各 PatchSpec を具象パッチ型に変換する。結合パッチには隣接セル中心マップから対応データを取得して注入する
  - 結合パッチ仕様に対応する隣接セル中心の存在と長さをバリデーションする
  - 各エラーケースで Task 1.1 で追加した MeshError バリアントを使用する
  - _Requirements: 8.2, 8.3, 8.4, 8.5, 8.6, 8.7_

- [ ] 6. テストユーティリティと結合テストを実装する
  - 直交格子から PolyMesh を生成するテスト用ユーティリティを #[cfg(test)] 内に提供する
  - PolyMesh 構築の成功ケース（結合パッチ有・無）をテストする
  - PolyMesh 構築の 4 種のエラーケース（面範囲不整合・重複・隣接データ欠落・長さ不一致）をテストする
  - PrimitiveMesh 委譲メソッドが元メソッドと同じ値を返すことをテストする
  - パッチの as_coupled() ダウンキャストが結合パッチで Some、非結合パッチで None を返すことをテストする
  - _Requirements: 8.8_

## 要件カバレッジ

| 要件 | タスク |
|------|--------|
| 1.1–1.9 | 3 |
| 2.1, 2.4–2.6, 2.9 | 4.1 |
| 2.2, 2.7 | 4.2 |
| 2.3, 2.8 | 4.3 |
| 3.1 | 1.2 |
| 3.2–3.4 | 4.2 |
| 4.1–4.2 | 4.3 |
| 5.1–5.5 | 1.3 |
| 6.1–6.2 | 1.4 |
| 7.1–7.7 | 5.1 |
| 8.1 | 2 |
| 8.2–8.7 | 5.2 |
| 8.8 | 6 |
| 9.1–9.3 | 1.1 |
