# 実装計画 — spec-init 単位への分解

本文書は `crate_design.md` の段階的実装順序を、`/kiro:spec-init` で実行する仕様単位（spec）に分解したものである。

---

## 設計原則

### spec の粒度

- **1 spec = 1つの明確な成果物**（コンパイル・テスト可能な単位）
- クレートが複数の独立した責務を持つ場合は分割する
- 逆に、クレートの責務が小さければ 1 spec にまとめる
- 各 spec は前段の spec に依存するが、同一フェーズ内で独立な spec は並行実行可能

### 命名規約

spec 名: `{クレート名}-{サブ機能}` （例: `types-tensor`, `fields-typestate`）

### 各 spec の完了条件

- `cargo build` が通ること
- `cargo test` で機能テストが通ること
- `cargo clippy` で警告がないこと
- 公開 API にドキュメントコメントがあること

---

## フェーズ一覧

```
Phase 1: 基盤型システム        [types]
Phase 2: メッシュとフィールド    [mesh, fields]
Phase 3: 離散化と求解          [discretization, solvers]
Phase 4: 拡張機構              [runtime, io]
Phase 5: 物理モデル            [models]
Phase 6: 統合検証              [apps/simple-solver]
```

### 依存関係図（spec 単位）

```
P1  types-tensor ──→ types-dimension ──→ types-quantity
         │                                    │
P2       └──→ mesh-primitive                  │
                    │                         │
              mesh-poly                       │
                    │                         │
              mesh-fv ──────→ fields-core ────┘
                                    │
                               fields-boundary
                                    │
P3                  ┌───────────────┘
                    │
            discretization-matrix
                    │
          discretization-implicit
                    │
          discretization-explicit
                    │
               solvers-core
                    │
P4    runtime-factory    io-config
            │               │
            │           io-field-rw
            │               │
P5    models-turbulence ────┘
                 │
P6    simple-solver-integration
```

---

## Phase 1: 基盤型システム (`types`)

外部依存: `typenum`（型レベル次元算術）のみ。プロジェクト全体の基盤。

### Spec 1-1: `types-tensor`

**概要**: テンソル型の定義と基本演算

**スコープ**:
- `Scalar`（`f64` alias）
- `Vector([f64; 3])`
- `Tensor([f64; 9])`（row-major）
- `SymmTensor([f64; 6])`
- `SphericalTensor(f64)`
- 各型の基本演算（`Add`, `Sub`, `Neg`, `Mul<f64>`）
- 異型間演算（約 25 impl）
  - 加算（異型）: `SymmTensor + SphericalTensor → SymmTensor` 等
  - スカラー倍: `f64 × {Vector, Tensor, ...}` 両方向
  - 縮約: `Tensor × Vector → Vector` 等
  - 二重縮約: `Tensor : Tensor → f64` 等
- 型変換メソッド: `symm()`, `two_symm()`, `sph()`, `skew()`, `dev()`, `trace()`, `det()`, `transpose()`
- `From` 変換: `From<SphericalTensor> for SymmTensor` 等

**成果物**: `crates/types/src/tensor/` モジュール

**テスト**:
- 全演算の数値正確性
- 特殊値（零テンソル、単位テンソル）
- 型変換の往復一貫性

---

### Spec 1-2: `types-field-value`

**概要**: `FieldValue` trait と ランク昇降 trait の定義

**前提**: Spec 1-1

**スコープ**:
- `FieldValue` trait:
  ```rust
  trait FieldValue: Copy + Add<Output=Self> + Sub<Output=Self> + Mul<f64, Output=Self> + Neg<Output=Self> {
      fn zero() -> Self;
      fn mag(&self) -> f64;
  }
  ```
- 全テンソル型への `FieldValue` 実装
- `HasGrad` trait: `type GradOutput: FieldValue`
- `HasDiv` trait: `type DivOutput: FieldValue`
- 各テンソル型へのランク昇降 impl:
  - `f64: HasGrad<GradOutput=Vector>`
  - `Vector: HasGrad<GradOutput=Tensor> + HasDiv<DivOutput=f64>`
  - `Tensor: HasDiv<DivOutput=Vector>`
  - `SymmTensor: HasDiv<DivOutput=Vector>`

**成果物**: `crates/types/src/traits/` モジュール

**テスト**:
- trait bounds の型レベル検証
- `zero()` / `mag()` の正確性

---

### Spec 1-3: `types-dimension`

**概要**: コンパイル時次元検査システム

**前提**: Spec 1-1, 1-2

**スコープ**:
- `Dim<V, M: Integer, L: Integer, T: Integer>` 構造体（`typenum` 型レベル整数）
- `Quantity` trait: `type Value = V`
- `Dim` への `Quantity` 実装
- 同次元の加算・減算（`typenum` で次元一致を型レベルで強制）
- 異次元の乗除算（`Sum<M1, M2>` / `Diff<M1, M2>` による次元指数算術）
- 物理量の型エイリアス:
  - `Pressure = Dim<f64, P1, N1, N2>`
  - `Velocity = Dim<Vector, Z0, P1, N1>`
  - `Density = Dim<f64, P1, N3, Z0>` 等
- `Quantity::Value: FieldValue` による接合

**成果物**: `crates/types/src/dimension/` モジュール

**テスト**:
- 同次元加算の型検査
- 異次元乗除算の次元指数計算
- `compile_fail` テスト: 異次元加算がコンパイルエラーになること
- 物理量の演算チェーン（例: `Pressure / Density → ?`）

---

## Phase 2: メッシュとフィールド (`mesh`, `fields`)

### Spec 2-1: `mesh-primitive`

**概要**: トポロジエンジン (`PrimitiveMesh`) の実装

**前提**: Phase 1 完了

**スコープ**:
- `Face = Vec<usize>`（任意多角形）の型エイリアス
- `PrimitiveMesh` 構造体:
  - 基本データ（構築時確定・不変）: `points`, `faces`, `owner`, `neighbor`, `n_internal_faces`, `n_cells`
  - 遅延計算ジオメトリ（`OnceCell`）: `cell_centers`, `face_centers`, `cell_volumes`, `face_areas`
  - 遅延計算接続情報（`OnceCell`）: `cell_cells`, `cell_faces`, `cell_points`
- `&self` アクセサメソッド（アクセス時に初期化）:
  - `cell_volumes()`, `cell_centers()`, `face_centers()`, `face_areas()`
  - `cell_cells()`, `cell_faces()`, `cell_points()`
- 内部計算メソッド: `calc_cell_volumes()`, `calc_face_centers()` 等
- 直交格子生成ユーティリティ:
  - `PrimitiveMesh::unit_cube(nx, ny, nz)` — テスト用の単純なメッシュ生成
- トポロジの整合性検証（owner/neighbor 一貫性等）

**成果物**: `crates/mesh/src/primitive_mesh.rs`

**テスト**:
- 直交格子の生成と体積・面積の正確性（手計算との比較）
- 遅延計算の初期化が一度だけ行われること（`OnceCell` の意味論）
- トポロジの整合性チェック
- `cell_cells` / `cell_faces` の正確性

---

### Spec 2-2: `mesh-poly`

**概要**: パッチ・ゾーン・並列メタデータ管理 (`PolyMesh`) の実装

**前提**: Spec 2-1

**スコープ**:
- パッチ trait 階層:
  - `PolyPatch` trait: `name()`, `start()`, `size()`, `patch_type()`, `as_coupled()`, `move_points()` フック
  - `CoupledPatch` trait（`PolyPatch` のサブ trait）: `face_cells()`, `neighbor_cell_centers()`, `set_neighbor_cell_centers()`, `neighbor_rank()`, `transform()`
- パッチ具象型:
  - `WallPolyPatch` (`PolyPatch`)
  - `CyclicPolyPatch` (`PolyPatch` + `CoupledPatch`)
  - `ProcessorPolyPatch` (`PolyPatch` + `CoupledPatch`)
  - `EmptyPolyPatch`, `SymmetryPolyPatch`, `WedgePolyPatch` (`PolyPatch`)
- `Transform` 型（cyclic パッチの変換情報）
- ゾーン型:
  - `Zone { name: String, indices: Vec<usize> }`（cellZone / pointZone 共用）
  - `FaceZone { name, indices, flip_map }`
- `GlobalMeshData` 構造体（並列トポロジ情報）
- `PolyMesh` 構造体:
  - `primitive: PrimitiveMesh`
  - `patches: Vec<Box<dyn PolyPatch>>`
  - `cell_zones`, `face_zones`, `point_zones`
  - `old_points: Option<...>`, `global_data: Option<GlobalMeshData>`
- 委譲メソッド（`PrimitiveMesh` へのアクセス）

**成果物**: `crates/mesh/src/{poly_mesh, patches/, zones}.rs`

**テスト**:
- `PolyPatch` のオブジェクト安全性
- `as_coupled()` によるアップキャスト
- `WallPolyPatch` / `CyclicPolyPatch` の生成と属性アクセス
- ゾーンのインデックスアクセス

---

### Spec 2-3: `mesh-fv`

**概要**: 有限体積法メッシュ (`FvMesh`) と並列対応構築フローの実装

**前提**: Spec 2-2

**スコープ**:
- `FvPatch` trait: `poly_patch()`, `delta()`, `weights()`
- `CoupledFvPatch` trait（`FvPatch` のサブ trait）: `poly_coupled_patch()`, `delta_neighbor()`
- `LduMesh` trait: `ldu_addressing()`, `n_cells()`
- `LduAddressing` 構造体: `lower`, `upper`, `losort`, `owner_start`, `losort_start`
- `FvMesh` 構造体:
  - `poly: PolyMesh`
  - `fv_patches: Vec<Box<dyn FvPatch>>`
  - `processor_patch_indices: Vec<usize>`
  - 遅延計算（`OnceCell`）: `ldu_addressing`, `weights`, `delta_coeffs`, `non_orth_delta_coeffs`, `non_orth_correction_vectors`
  - 旧時刻データ: `v0`, `v00`（非定常計算用）
  - `mover: Option<Box<dyn MeshMover>>`
- `impl LduMesh for FvMesh`
- 委譲メソッド（`PolyMesh` / `PrimitiveMesh` へのアクセス）
- 二相構築パターン（`build_fv_mesh` 関数）:
  - Phase 1-2: 具象型のまま構築・初期化（`CyclicPolyPatch` の `neighbor_cell_centers` 計算、`ProcessorPolyPatch` の MPI 交換 placeholder）
  - Phase 3: 型消去（`Vec<Box<dyn PolyPatch>>`）+ `processor_patch_indices` 記録
  - Phase 4: `PolyMesh` → `FvMesh` 構築
- 動的メッシュ対応 `move_points()` の骨格（MPI 交換は placeholder）
- 直交格子から `FvMesh` を生成するテスト用ユーティリティ:
  - `FvMesh::unit_cube(nx, ny, nz)`

**成果物**: `crates/mesh/src/{fv_mesh, fv_patches/, ldu_addressing, build}.rs`

**テスト**:
- 直交格子での LDU アドレッシングの正確性
- 補間係数（`weights`, `delta_coeffs`）の数値検証
- `LduMesh` trait 経由のアドレッシングアクセス
- `processor_patch_indices` が正しく記録されること
- `FvMesh` から `PrimitiveMesh` の各量への委譲アクセス

---

### Spec 2-4: `fields-core`

**概要**: フィールド型と typestate パターン（シリアル版）

**前提**: Spec 2-3

**スコープ**:
- `Fresh` / `Stale` マーカー型
- `VolumeField<'mesh, T, State>`:
  - `mesh: &'mesh FvMesh`
  - `internal: Vec<T>`
  - `_state: PhantomData<S>`
  - 構築は `Stale` で開始
- `SurfaceField<'mesh, T>`:
  - 面中心フィールド（面積分量 / 補間量）
- 状態遷移メソッド（シリアル版・境界条件なし）:
  - `map_internal(f: impl Fn(T) -> T) -> VolumeField<'mesh, T, Stale>`
  - placeholder の `evaluate_boundaries() -> VolumeField<'mesh, T, Fresh>`（本実装は Spec 2-5）
- フィールドの基本演算（`Add`, `Sub`, `Mul<f64>` on internal values）

**成果物**: `crates/fields/src/{volume_field, surface_field, state}.rs`

**テスト**:
- typestate 遷移の型レベル検証
- `compile_fail`: `Stale` フィールドを `Fresh` 要求の関数に渡せないこと
- フィールド演算の数値正確性
- ライフタイム: フィールドがメッシュより長生きできないこと

---

### Spec 2-5: `fields-boundary`

**概要**: 境界条件システム

**前提**: Spec 2-4

**スコープ**:
- `PhysicalBC<T>` trait:
  ```rust
  trait PhysicalBC<T> {
      fn evaluate(&mut self, internal: &[T]);
      fn patch_values(&self) -> &[T];
  }
  ```
- `ProcessorPatch<T>` 構造体（ランク番号・バッファ保持、MPI 通信は placeholder）
- `BoundaryPatch<T>` enum:
  ```rust
  enum BoundaryPatch<T> {
      Physical(Box<dyn PhysicalBC<T>>),
      Processor(ProcessorPatch<T>),
  }
  ```
- `VolumeField` への境界条件統合:
  - `boundaries: Vec<BoundaryPatch<T>>` フィールド追加
  - `evaluate_boundaries()` の本実装（シリアル版: 物理 BC の evaluate のみ）
  - `boundary_values(patch_id)` 統一アクセスメソッド
- 基本的な物理境界条件:
  - `FixedValue<T>`: ディリクレ条件
  - `ZeroGradient<T>`: ノイマン条件（勾配ゼロ）

**成果物**: `crates/fields/src/{boundary, boundary_conditions/}.rs`

**テスト**:
- `FixedValue` / `ZeroGradient` の evaluate 正確性
- `evaluate_boundaries` → `Fresh` への遷移
- `boundary_values` の統一アクセス
- `dyn PhysicalBC<T>` のオブジェクト安全性

---

## Phase 3: 離散化と求解 (`discretization`, `solvers`)

### Spec 3-1: `discretization-matrix`

**概要**: FvMatrix の構造と基本操作

**前提**: Phase 2 完了

**スコープ**:
- `FvMatrix<V>` 構造体（次元なし）:
  - 対角成分: `Vec<f64>`（又は `Vec<V>`）
  - 上三角・下三角（隣接セル成分）: `Vec<f64>`
  - ソース項: `Vec<V>`
  - メッシュ参照（疎行列の接続情報）
- 行列の加算（`Add<FvMatrix<V>>`）: PDE 項の足し合わせ
- 行列の減算（`Sub<FvMatrix<V>>`）
- `.rhs(source)` メソッド: 右辺ソース項の設定
- 対角優勢化（`relax(factor)`）: SIMPLE アルゴリズム用の緩和

**成果物**: `crates/discretization/src/fv_matrix.rs`

**テスト**:
- 行列加算の正確性
- `.rhs()` によるソース項設定
- 緩和係数の適用

---

### Spec 3-2: `discretization-implicit`

**概要**: 陰的離散化演算子（`ImplicitOps`）

**前提**: Spec 3-1

**スコープ**:
- `Schemes` 構造体（数値スキーム設定の保持）:
  - 時間スキーム（Euler）
  - 対流スキーム（upwind — 最初の実装）
  - 拡散スキーム（線形補間）
- `ImplicitOps<'a>` 構造体:
  - `mesh: &'a FvMesh`
  - `schemes: &'a Schemes`
- 陰的離散化演算子:
  - `ddt(field: &VolumeField<Q, Fresh>) -> FvMatrix<Q::Value>`: 時間微分（Euler 陰的）
  - `div(flux: &SurfaceField<C>, field: &VolumeField<Q, Fresh>) -> FvMatrix<Q::Value>`: 対流項（upwind）
  - `laplacian(coeff: &VolumeField<C>, field: &VolumeField<Q, Fresh>) -> FvMatrix<Q::Value>`: 拡散項
- **次元消去**: 入力は次元付きフィールド、出力は次元なし `FvMatrix<V>`

**成果物**: `crates/discretization/src/{schemes, implicit_ops}.rs`

**テスト**:
- 1D 直交格子での係数行列の手計算検証
- 対流スキーム（upwind）の行列構造
- 拡散項の対称性

---

### Spec 3-3: `discretization-explicit`

**概要**: 陽的評価演算子（`ExplicitOps`）

**前提**: Spec 3-1, 2-3（境界条件アクセスが必要）

**スコープ**:
- `ExplicitOps<'a>` 構造体
- 陽的評価演算子:
  - `grad(field: &VolumeField<Q, Fresh>) -> VolumeField<Q::GradOutput, Stale>`: 勾配（ガウスの定理）
  - `div(field: &SurfaceField<Q>) -> VolumeField<Q::DivOutput, Stale>`: 発散
  - `laplacian(coeff, field) -> VolumeField<Q::Value, Stale>`: 拡散項の陽的評価
- 面補間ユーティリティ:
  - セル中心値 → 面中心値の線形補間

**成果物**: `crates/discretization/src/{explicit_ops, interpolation}.rs`

**テスト**:
- 線形フィールドの勾配が定数であること
- ガウスの定理の検証（閉じた体積での発散積分）
- 面補間の正確性

---

### Spec 3-4: `solvers-core`

**概要**: 線形ソルバーの trait と最初の実装

**前提**: Spec 3-1

**スコープ**:
- `LinearSolver` trait:
  ```rust
  trait LinearSolver {
      fn solve<V: FieldValue>(&self, matrix: &FvMatrix<V>, x: &mut [V]) -> SolveResult;
  }
  ```
- `SolveResult` 構造体（converged, iterations, initial/final residual）
- `SolverSettings`（tolerance, max iterations）
- CG（共役勾配法）実装:
  - 対称正定値行列のソルバー（拡散方程式の圧力方程式用）
- BiCGSTAB 実装:
  - 非対称行列のソルバー（対流項を含む運動量方程式用）
- 前処理:
  - 対角スケーリング（最小限の前処理）

**成果物**: `crates/solvers/src/` 全体

**テスト**:
- 既知の解を持つ小さな系での収束検証
- 収束判定の正確性（残差の減少）
- 非収束ケースの `SolveResult` の挙動

---

## Phase 4: 拡張機構 (`runtime`, `io`)

Phase 3 と独立して並行開発可能な部分を含む。

### Spec 4-1: `runtime-factory`

**概要**: `inventory` ベースの実行時型選択機構

**前提**: なし（独立して開発可能。統合は後段）

**スコープ**:
- `inventory` crate の導入
- ファクトリ登録のパターン定義:
  ```rust
  struct Factory<T: ?Sized> {
      name: &'static str,
      constructor: fn() -> Box<T>,
  }
  ```
- 解決関数:
  - `resolve<T>(name: &str) -> Option<Box<T>>`
  - 登録名一覧取得
  - 未知の名前に対するエラーメッセージ（利用可能な名前の列挙）
- 登録用のヘルパーマクロ（将来拡張の基盤）

**成果物**: `crates/runtime/src/` 全体

**テスト**:
- ファクトリ登録と解決の往復
- 未知の名前に対するエラー
- 複数ファクトリの共存

---

### Spec 4-2: `io-config`

**概要**: 設定ファイルの読み込みシステム

**前提**: なし（独立して開発可能）

**設計判断が必要**:
- フォーマットの選定（TOML / YAML / OpenFOAM 辞書形式 / 独自）
- 本 spec 開始前に `/kiro:spec-requirements` で要件を明確化する

**スコープ**:
- 辞書（Dictionary）型の定義:
  - ネストしたキー・バリュー構造
  - 型付き値の取得（`get::<T>(key)` → `Result<T, ConfigError>`）
- 設定ファイルの読み込み:
  - `serde` によるデシリアライズ
  - 選定したフォーマットのパーサー統合
- ソルバー設定の構造体:
  - 数値スキーム設定（`Schemes`）
  - ソルバー設定（`SolverSettings`）
  - 時間ステップ設定

**成果物**: `crates/io/src/{config, dictionary}.rs`

**テスト**:
- 設定ファイルの読み込みと型付きアクセス
- 必須キー欠落時のエラー
- ネストした辞書のアクセス

---

### Spec 4-3: `io-field-rw`

**概要**: フィールドとメッシュの読み書き

**前提**: Spec 4-2, Phase 2 完了

**スコープ**:
- フィールドの書き出し:
  - `VolumeField<T>` → ファイル（フォーマットは設定で選択）
  - タイムステップごとのディレクトリ構造
- フィールドの読み込み:
  - ファイル → `VolumeField<T>` の再構築
  - 初期条件の設定ファイルからの読み込み
- メッシュの読み込み:
  - OpenFOAM polyMesh フォーマット（`constant/polyMesh/`）
  - または独自の簡易フォーマット（要設計判断）

**成果物**: `crates/io/src/{field_io, mesh_io}.rs`

**テスト**:
- フィールドの書き出し → 読み込みの往復一貫性
- メッシュ読み込みの正確性

---

## Phase 5: 物理モデル (`models`)

### Spec 5-1: `models-turbulence`

**概要**: 乱流モデルの trait と最初の実装

**前提**: Phase 3 完了, Spec 4-1

**スコープ**:
- `TurbulenceModel` trait:
  - `correct()`: 乱流場の更新
  - `nut()`: 渦粘性の取得
  - `k()`: 乱流エネルギー
  - `epsilon()` / `omega()`: 散逸率
- k-omega SST モデルの実装:
  - k 方程式と omega 方程式の離散化
  - 壁関数（簡易版）
- `inventory` によるファクトリ登録:
  ```rust
  inventory::submit! {
      TurbulenceModelFactory { name: "kOmegaSST", constructor: || Box::new(KOmegaSST::new()) }
  }
  ```

**成果物**: `crates/models/src/` 全体

**テスト**:
- 渦粘性の計算検証
- ファクトリ登録と `"kOmegaSST"` からの生成
- 乱流場の更新（1 ステップ）

---

## Phase 6: 統合検証 (`apps/simple-solver`)

### Spec 6-1: `simple-solver-integration`

**概要**: SIMPLE アルゴリズムによるソルバー全体の統合

**前提**: Phase 1–5 完了

**スコープ**:
- SIMPLE アルゴリズムの実装:
  1. 運動量方程式の組み立てと求解（予測ステップ）
  2. 圧力方程式の組み立てと求解（修正ステップ）
  3. 速度の修正
  4. 乱流モデルの更新
  5. 収束判定
- ソルバー構造体（依存性注入の起点）:
  ```rust
  struct SimpleSolver<'mesh> {
      mesh: &'mesh FvMesh,
      p: VolumeField<'mesh, Scalar, _>,
      u: VolumeField<'mesh, Vector, _>,
      phi: SurfaceField<'mesh, Scalar>,
      turbulence: Box<dyn TurbulenceModel>,
      // ...
  }
  ```
- I/O 統合:
  - 設定ファイルからのソルバーパラメータ読み込み
  - 結果の書き出し（ソルバーが明示列挙）
- 検証ケース:
  - リッド駆動キャビティ流れ（2D / 低レイノルズ数）

**成果物**: `apps/simple-solver/src/` 全体

**テスト**:
- リッド駆動キャビティ流れの定常解との比較
- 残差の収束履歴
- 全クレートの統合が正常に機能すること

---

## Phase 間の依存関係と並行性

```
時間軸 →

Phase 1: [types-tensor] → [types-field-value] → [types-dimension]
                                                        │
Phase 2:       [mesh-primitive] → [mesh-poly] → [mesh-fv] → [fields-core] → [fields-boundary]
                                                                                    │
Phase 3:                                              [disc-matrix] → [disc-implicit] → [disc-explicit]
                                                           │
                                                      [solvers-core]
                                                           │
Phase 4: [runtime-factory] ─────────────────────┐         │
         [io-config] → [io-field-rw] ───────────┤         │
                                                 │         │
Phase 5:                            [models-turbulence] ←──┘
                                          │
Phase 6:                      [simple-solver-integration]
```

**並行開発可能な組み合わせ**:
- Phase 4 の `runtime-factory` と `io-config` は Phase 1 完了後から開発開始可能
- Phase 3 の `solvers-core` は `discretization-matrix` 完了後から独立して開発可能
- Phase 3 の `discretization-explicit` は `discretization-implicit` と並行開発可能

---

## 未決定事項（spec 開始前に解決が必要）

| 課題 | 関連 spec | 選択肢 | 推奨 |
|------|----------|--------|------|
| 辞書フォーマット | `io-config` | TOML / YAML / 独自 | TOML（Rust エコシステムとの親和性） |
| メッシュ読み込みフォーマット | `io-field-rw` | OpenFOAM polyMesh / 独自 | 両方（polyMesh 優先） |
| MPI なしシリアルビルド | `fields-boundary` | feature flag / SingleWorld | feature flag (`default = ["mpi"]`) |
| フィールド間の BC 依存 | `fields-boundary` | evaluate にコンテキスト / ソルバー管理 | ソルバー管理（Phase 6 で統合） |
| PhysicalBC の行列寄与 | `discretization-implicit` | trait メソッド追加 / 別 trait | trait メソッド追加（`matrix_contribution()`） |

---

## spec 実行の手順

各 spec の実行フロー:

```
/kiro:spec-init "{spec 名}: {概要}"
/kiro:spec-requirements {spec 名}
/kiro:spec-design {spec 名}
/kiro:spec-tasks {spec 名}
/kiro:spec-impl {spec 名}
/kiro:validate-impl {spec 名}
```

Phase 完了ごとに `cargo build && cargo test` で全体の整合性を確認する。
