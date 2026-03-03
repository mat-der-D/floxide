# 実装タスク: mesh-primitive

- [x] 1. クレート基盤の整備

- [x] 1.1 Cargo.toml に依存関係を追加する
  - `crates/mesh/Cargo.toml` に `thiserror` を依存クレートとして追加する
  - `dugong-types` がパス依存として正しく設定されていることを確認する
  - Rust Edition 2024 設定を確認する
  - _Requirements: 1.1, 7.6_

- [x] 1.2 モジュール宣言と公開 API の設定
  - `lib.rs` に `mod error;`・`mod geometry;`・`mod primitive_mesh;` を宣言する
  - `pub use error::MeshError;` と `pub use primitive_mesh::PrimitiveMesh;` を設定する
  - 各モジュールファイルの空スタブを作成して `cargo check` を通す
  - _Requirements: 1.1, 6.1_

- [x] 2. `MeshError` エラー型の実装（error.rs）
  - `OwnerLengthMismatch`・`NeighborLengthMismatch`・`OwnerIndexOutOfRange`・`NeighborIndexOutOfRange`・`PointIndexOutOfRange` の5バリアントを定義する
  - 各バリアントにエラー発生箇所を特定できるフィールド（`face`・`cell`・`n_cells`・`n_points`・`expected`・`got` など）を設ける
  - `#[derive(Debug, thiserror::Error)]` でトレイトを自動導出し、各バリアントに `#[error("...")]` で具体的なメッセージを付与する
  - `std::error::Error` が実装されていることを確認する
  - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5, 7.6_

- [x] 3. `PrimitiveMesh` 基本実装（primitive_mesh.rs）

- [x] 3.1 構造体の定義
  - 基本データフィールド（`points: Vec<Vector>`・`faces: Vec<Vec<usize>>`・`owner: Vec<usize>`・`neighbor: Vec<usize>`・`n_internal_faces: usize`・`n_cells: usize`）を定義する
  - 遅延計算フィールド（`cell_centers`・`cell_volumes`・`face_centers`・`face_areas`・`cell_cells`・`cell_faces`・`cell_points` として各 `std::sync::OnceLock`）を定義する
  - 全フィールドを非 `pub` として不変性を確保し、`unsafe` コードは一切使用しない
  - `dugong_types::Vector` を座標型・ジオメトリデータ型として採用する
  - _Requirements: 1.1, 1.3, 1.4, 5.1, 5.4_

- [x] 3.2 コンストラクタの不変条件バリデーション
  - `PrimitiveMesh::new(points, faces, owner, neighbor, n_internal_faces, n_cells) -> Result<Self, MeshError>` を実装する
  - `owner.len() != faces.len()` なら `OwnerLengthMismatch` を返す
  - `neighbor.len() != n_internal_faces` なら `NeighborLengthMismatch` を返す
  - `owner` に `n_cells` 以上のインデックスがあれば `OwnerIndexOutOfRange` を返す
  - `neighbor` に `n_cells` 以上のインデックスがあれば `NeighborIndexOutOfRange` を返す
  - 各面の点インデックスに `points.len()` 以上の値があれば `PointIndexOutOfRange` を返す
  - 全検証通過後、`OnceLock` を未初期化状態として `Ok(Self)` を返す
  - _Requirements: 1.2, 1.3, 1.5, 7.1, 7.2, 7.3, 7.4, 7.5_

- [x] 3.3 基本アクセサの実装
  - `points(&self) -> &[Vector]`・`faces(&self) -> &[Vec<usize>]`・`owner(&self) -> &[usize]`・`neighbor(&self) -> &[usize]` を実装する
  - `n_internal_faces(&self) -> usize`・`n_cells(&self) -> usize`・`n_faces(&self) -> usize`・`n_points(&self) -> usize` を実装する
  - いずれも `&self` のみで呼び出せる不変メソッドとし、内部フィールドへのスライス参照を返す
  - _Requirements: 1.4, 6.1, 6.2, 6.3, 6.4, 6.5, 6.6, 6.7, 6.8_

- [x] 4. 面ジオメトリの実装

- [x] 4.1 (P) 面重心・面積ベクトル計算関数の実装（geometry.rs）
  - `pub(crate) fn compute_face_geometry(points: &[Vector], face: &[usize]) -> (Vector, Vector)` を実装する
  - 面頂点の単純平均を参照点としてファン三角形分割を行い、各三角形の面積ベクトル（`0.5 * (v_next - p_ref) × (v_cur - p_ref)`）を算出する
  - 各三角形の面積ベクトルの合計を面積ベクトルとし、面積加重平均で面重心を計算する
  - 面積ベクトルのノルムが実面積と一致することを単体テストで確認する（相対誤差 1e-10）
  - _Requirements: 3.5, 3.6_

- [x] 4.2 (P) 面ジオメトリ遅延計算アクセサの実装（primitive_mesh.rs）
  - `face_centers(&self) -> &[Vector]` と `face_areas(&self) -> &[Vector]` を `OnceLock::get_or_init` パターンで実装する
  - 全面に対して `compute_face_geometry` を呼び出し、`face_centers` と `face_areas` を同時にキャッシュする
  - 面積ベクトルの向きが頂点順序（OpenFOAM 慣行: 内部面は owner → neighbor、境界面は外向き）に従うことを確認する
  - 2回目以降の呼び出しでキャッシュ済みの値を返すことを確認する
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.7_

- [x] 5. セルジオメトリの実装

- [x] 5.1 (P) セル体積・重心計算関数の実装（geometry.rs）
  - 各セルの全所属面について面中心と面積ベクトルを計算し、ピラミッド分解でセル体積を計算する内部関数を実装する
  - ピラミッド体積: `face_area_vec · (face_center - ref_point) / 3`（owner セルは正、neighbor セルは負の寄与）
  - 各ピラミッドの重心（`0.75 * ref_point + 0.25 * face_center`）を体積加重平均してセル重心を計算する
  - 面ジオメトリ計算には `compute_face_geometry`（4.1 で実装済み）を内部利用する
  - _Requirements: 2.5, 2.6_

- [x] 5.2 (P) セルジオメトリ遅延計算アクセサの実装（primitive_mesh.rs）
  - `cell_volumes(&self) -> &[f64]` と `cell_centers(&self) -> &[Vector]` を `OnceLock` パターンで実装する
  - 2 つのアクセサは OnceLock 初期化ブロック内でセル体積とセル重心を一括して計算して効率化する
  - キャッシュ済みの場合は再計算せず既存のスライスを返す
  - _Requirements: 2.1, 2.2, 2.3, 2.4_

- [x] 6. セル接続情報の実装

- [x] 6.1 (P) cell_faces・cell_cells・cell_points 計算関数の実装（geometry.rs）
  - `owner`/`neighbor` を全面走査して各セルの所属面インデックスリストを構築する関数を実装する
  - 内部面のみを対象として各セルの隣接セルリストを導出する関数を実装する（境界面は隣接セルなし）
  - 各セルの所属面が参照する頂点インデックスを `BTreeSet` 等で重複なく収集する関数を実装する
  - _Requirements: 4.4, 4.5, 4.6, 4.7_

- [x] 6.2 (P) 接続情報遅延計算アクセサの実装（primitive_mesh.rs）
  - `cell_faces(&self) -> &[Vec<usize>]`・`cell_cells(&self) -> &[Vec<usize>]`・`cell_points(&self) -> &[Vec<usize>]` を `OnceLock` パターンで実装する
  - 各アクセサは初回アクセス時のみ計算を実行してキャッシュし、以降はキャッシュ済みの値を返す
  - _Requirements: 4.1, 4.2, 4.3, 4.4_

- [x] 7. テスト実装と数値精度検証

- [x] 7.1 テストヘルパーの作成
  - 単一立方体セル（8点・6面・`n_internal_faces=0`・`n_cells=1`）を返す `make_unit_cube_mesh()` を `#[cfg(test)]` 内に実装する
  - 内部面を1つもつ2セルメッシュを返す `make_two_cell_mesh()` も実装し、接続情報テストおよび面積ベクトル保存則テストに備える
  - _Requirements: 8.1, 8.2_

- [x] 7.2 コンストラクタのエラー処理テスト
  - owner 長さ不一致・neighbor 長さ不一致・owner インデックス範囲外・点インデックス範囲外の各ケースで `Err` が返ることを検証する
  - 正常ケース（単位立方体）で `Ok(mesh)` が返ることも検証する
  - _Requirements: 7.2, 7.3, 7.4, 7.5, 8.5_

- [x] 7.3 セルジオメトリの数値精度テスト
  - 単位立方体のセル体積が 1.0 と一致することを検証する（相対誤差 1e-10）
  - 単位立方体のセル重心が (0.5, 0.5, 0.5) と一致することを検証する（絶対誤差 1e-10）
  - 同一アクセサを2回呼び出した際に同じ参照が返ることを確認してキャッシュを検証する
  - _Requirements: 2.3, 2.4, 2.5, 2.6, 8.2, 8.3, 8.6_

- [x] 7.4 面ジオメトリの数値精度テスト
  - 単位立方体の各面積ベクトルのノルムが 1.0 と一致することを検証する（相対誤差 1e-10）
  - 2セルメッシュの内部面について面積ベクトルの総和がゼロベクトルと一致することを検証する（絶対誤差 1e-12）
  - _Requirements: 3.4, 3.5, 3.6, 3.7, 8.4, 8.6_

- [x] 7.5 接続情報と Send/Sync テスト
  - 単位立方体の `cell_faces` がセルの全6面を含むことを検証する
  - 2セルメッシュで `cell_cells` が正しい隣接セルを返すことを検証する
  - 単位立方体の `cell_points` に重複がないことを検証する
  - コンパイル時に `PrimitiveMesh: Send + Sync` であることを型チェックで確認する（`fn assert_send_sync<T: Send + Sync>()` を使用）
  - _Requirements: 4.5, 4.6, 4.7, 5.1, 5.2, 5.3, 8.1_
