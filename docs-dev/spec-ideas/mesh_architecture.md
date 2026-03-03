# 3層メッシュアーキテクチャ設計案

OpenFOAM のメッシュアーキテクチャを調査し、Rust の言語制約を考慮した上で設計した3層メッシュ構造。

---

## 全体構造

```
PrimitiveMesh (struct)
│  基本トポロジ + 遅延計算ジオメトリ/接続情報
│
PolyMesh (struct, owns PrimitiveMesh)
│  パッチ (dyn PolyPatch) + ゾーン + 並列メタデータ
│
FvMesh (struct, owns PolyMesh)
   FVパッチ (dyn FvPatch) + LDUアドレッシング + 補間係数 + 旧時刻データ
   impl LduMesh
```

C++ の継承チェーンを合成 (composition) で表現。層間アクセスは委譲メソッド（`Deref` 不使用）。

---

## 第1層: PrimitiveMesh

トポロジエンジン。基本データの格納と、そこから派生するジオメトリ・接続情報の遅延計算を担う。

```rust
pub type Face = Vec<usize>;  // 任意多角形

pub struct PrimitiveMesh {
    // 基本データ（構築時確定、不変）
    points: Vec<Vector>,
    faces: Vec<Face>,
    owner: Vec<usize>,
    neighbor: Vec<usize>,      // len == n_internal_faces
    n_internal_faces: usize,
    n_cells: usize,

    // 遅延計算ジオメトリ
    cell_centers: OnceCell<Vec<Vector>>,
    face_centers: OnceCell<Vec<Vector>>,
    cell_volumes: OnceCell<Vec<f64>>,
    face_areas:   OnceCell<Vec<Vector>>,   // 面積ベクトル（法線×面積）

    // 遅延計算接続情報
    cell_cells:  OnceCell<Vec<Vec<usize>>>,
    cell_faces:  OnceCell<Vec<Vec<usize>>>,
    cell_points: OnceCell<Vec<Vec<usize>>>,
    // 必要に応じて他の接続情報を追加
}
```

### 遅延計算パターン

```rust
impl PrimitiveMesh {
    pub fn cell_volumes(&self) -> &[f64] {
        self.cell_volumes.get_or_init(|| self.calc_cell_volumes())
    }

    fn calc_cell_volumes(&self) -> Vec<f64> {
        // points, faces, owner から計算
        todo!()
    }
}
```

### 設計判断

| 判断 | 選択 | 根拠 |
|------|------|------|
| 遅延計算の実現 | `OnceCell` | 構築後不変、`&self` で初期化可能、`Sync` を自動的に満たす |
| 面の表現 | `Vec<usize>`（任意多角形） | OpenFOAM 同等の汎用性。将来 `SmallVec<[usize; 4]>` で最適化可能 |
| `neighbor` の長さ | 内部面のみ | 境界面は neighbor なし。OpenFOAM と同じ慣行 |
| trait にするか | struct | 「異なるメッシュ実装を差し替える」ユースケースがない |
| 動的メッシュ対応 | 将来 `OnceCell` → リセット可能な `LazyCache` 型に置き換え | 公開 API（`&self` アクセサ）は変わらない |

---

## 第2層: PolyMesh

パッチ・ゾーン・並列メタデータを管理し、メッシュの構造的情報を担う。

```rust
pub struct PolyMesh {
    primitive: PrimitiveMesh,
    patches: Vec<Box<dyn PolyPatch>>,
    cell_zones: Vec<Zone>,
    face_zones: Vec<FaceZone>,
    point_zones: Vec<Zone>,
    old_points: Option<Vec<Vector>>,          // 非定常/メッシュ運動用
    global_data: Option<GlobalMeshData>,      // シリアル時 None
}

pub struct Zone {
    pub name: String,
    pub indices: Vec<usize>,
}

pub struct FaceZone {
    pub name: String,
    pub indices: Vec<usize>,
    pub flip_map: Vec<bool>,
}

pub struct GlobalMeshData {
    pub n_total_cells: usize,
    pub n_total_points: usize,
    pub shared_point_labels: Vec<usize>,
    pub shared_point_addressing: Vec<Vec<usize>>,
}
```

### ゾーン

OpenFOAM に倣い3種のゾーンを保持する:

- **cellZone**: セルのサブセット（MRF、多孔質領域、ソース項適用域など）
- **faceZone**: 面のサブセット + 向き情報（baffle、内部インターフェース、面積分計算）
- **pointZone**: 節点のサブセット（メッシュ運動制約など）

### GlobalMeshData

並列計算で必要なプロセッサ間共有トポロジ情報。グローバル残差計算（`n_total_cells` で正規化）、出力時のグローバルインデックス、メッシュ品質検査に使用。シリアル計算時は `None`。

---

## trait 階層: パッチ

### PolyPatch / CoupledPatch

```rust
pub trait PolyPatch: Send + Sync {
    fn name(&self) -> &str;
    fn start(&self) -> usize;
    fn size(&self) -> usize;
    fn patch_type(&self) -> &str;

    // CoupledPatch へのアップキャスト
    fn as_coupled(&self) -> Option<&dyn CoupledPatch> { None }
    fn as_coupled_mut(&mut self) -> Option<&mut dyn CoupledPatch> { None }

    // メッシュ操作フック（デフォルト: 何もしない）
    fn move_points(&mut self, _new_points: &[Vector]) {}
}

pub trait CoupledPatch: PolyPatch {
    fn face_cells(&self) -> &[usize];
    fn neighbor_cell_centers(&self) -> &[Vector];
    fn set_neighbor_cell_centers(&mut self, centers: Vec<Vector>);
    fn neighbor_rank(&self) -> Option<i32>;  // ProcessorPatch のみ Some
    fn transform(&self) -> &Transform;
}
```

### FvPatch / CoupledFvPatch

```rust
pub trait FvPatch: Send + Sync {
    fn poly_patch(&self) -> &dyn PolyPatch;
    fn delta(&self, mesh: &PrimitiveMesh) -> Vec<Vector>;
    fn weights(&self, mesh: &PrimitiveMesh) -> Vec<f64>;
}

pub trait CoupledFvPatch: FvPatch {
    fn poly_coupled_patch(&self) -> &dyn CoupledPatch;
    fn delta_neighbor(&self, mesh: &PrimitiveMesh) -> Vec<Vector>;
}
```

### LduMesh（ソルバー接合）

```rust
// mesh クレートに定義（orphan rule 制約）
pub trait LduMesh {
    fn ldu_addressing(&self) -> &LduAddressing;
    fn n_cells(&self) -> usize;
}

impl LduMesh for FvMesh { ... }
```

### パッチ具象型

| 型 | impl する trait | 用途 |
|----|----------------|------|
| `WallPolyPatch` | `PolyPatch` | 壁面 |
| `CyclicPolyPatch` | `PolyPatch` + `CoupledPatch` | 周期境界 |
| `ProcessorPolyPatch` | `PolyPatch` + `CoupledPatch` | プロセッサ間境界 |
| `EmptyPolyPatch` | `PolyPatch` | 2D計算の空方向 |
| `SymmetryPolyPatch` | `PolyPatch` | 対称面 |
| `WedgePolyPatch` | `PolyPatch` | 軸対称 |

### パッチ設計の根拠

OpenFOAM ではパッチがクラス階層と仮想メソッドで表現されており、以下が種別ごとに異なる:

- delta ベクトルの計算方法（壁面 vs cyclic vs processor）
- 補間重みの計算方法
- メッシュ操作時のフック処理

これらの振る舞いの差異を表現するため、単なる `enum` ではなく `dyn PolyPatch` trait を採用した。

`inventory` + ファクトリ登録はメッシュ I/O 時の文字列→具象型の生成に使用する。パッチ種別自体はフレームワーク内部で有限であり、ユーザーが新規追加する性質のものではないため、`inventory` は I/O のパーサー利便性のために使う。

---

## 第3層: FvMesh

有限体積法の離散化に必要なデータを担う。

```rust
pub struct FvMesh {
    poly: PolyMesh,
    fv_patches: Vec<Box<dyn FvPatch>>,
    processor_patch_indices: Vec<usize>,  // 構築時に記録

    // 遅延計算
    ldu_addressing: OnceCell<LduAddressing>,
    weights: OnceCell<Vec<f64>>,
    delta_coeffs: OnceCell<Vec<f64>>,
    non_orth_delta_coeffs: OnceCell<Vec<f64>>,
    non_orth_correction_vectors: OnceCell<Vec<Vector>>,

    // 旧時刻データ（非定常計算用）
    v0: Option<Vec<f64>>,
    v00: Option<Vec<f64>>,

    // プラグインスロット（当面 None、将来の拡張点）
    mover: Option<Box<dyn MeshMover>>,
}

pub struct LduAddressing {
    pub lower: Vec<usize>,
    pub upper: Vec<usize>,
    pub losort: Vec<usize>,
    pub owner_start: Vec<usize>,
    pub losort_start: Vec<usize>,
}
```

### 補間係数

OpenFOAM の `surfaceInterpolation` に相当。パッチごとの計算差異は `FvPatch` trait で吸収し、内部面の補間係数は FvMesh 内で遅延計算する。

| 係数 | 用途 |
|------|------|
| `weights` | セル中心値→面中心値の線形補間重み |
| `delta_coeffs` | セル中心間距離の逆数（勾配計算用） |
| `non_orth_delta_coeffs` | 非直交メッシュ補正係数 |
| `non_orth_correction_vectors` | 非直交メッシュ補正ベクトル |

### processor_patch_indices

構築時にプロセッサパッチのインデックスを記録しておくことで、動的メッシュでの並列ジオメトリ再交換時にダウンキャストなしで `CoupledPatch` trait 経由のアクセスが可能。

---

## メッシュ構築フロー（並列対応）

ダウンキャストを回避するため、型固有の初期化を型消去前に行う二相構築パターンを採用する。

```
Phase 1: I/O → PrimitiveMesh + パッチ定義（各プロセッサ独立）

Phase 2: 具象型のまま構築・初期化（型消去前）
  - CyclicPatch: 対パッチの face_cells からローカルに neighbor_cell_centers を計算
  - ProcessorPatch: 隣接ランクと MPI で cell_centers を交換

Phase 3: 初期化完了後に型消去
  - Vec<具象型> → Vec<Box<dyn PolyPatch>>
  - processor_patch_indices を記録

Phase 4: PolyMesh → FvMesh 構築
  - FvPatch を構築（CoupledPatch のジオメトリ情報は既に揃っている）
  - LDU アドレッシング、補間係数は遅延計算
```

```rust
pub fn build_fv_mesh(
    primitive: PrimitiveMesh,
    raw_patches: Vec<RawPatchDef>,
    world: Option<&SystemCommunicator>,
) -> FvMesh {
    let cell_centers = primitive.cell_centers();

    // Phase 2: 具象型のまま構築・初期化
    let mut walls: Vec<WallPolyPatch> = vec![];
    let mut cyclics: Vec<CyclicPolyPatch> = vec![];
    let mut processors: Vec<ProcessorPolyPatch> = vec![];
    // ... パッチ定義を振り分けて構築

    for cyc in &mut cyclics {
        let neighbor_centers: Vec<_> = cyc.neighbor_face_cells()
            .iter()
            .map(|&c| cell_centers[c])
            .collect();
        cyc.neighbor_cell_centers = neighbor_centers;
    }

    if let Some(world) = world {
        for proc in &mut processors {
            let local: Vec<_> = proc.face_cells
                .iter()
                .map(|&c| cell_centers[c])
                .collect();
            proc.neighbor_cell_centers =
                mpi_exchange(world, proc.neighbor_rank, &local);
        }
    }

    // Phase 3: 初期化完了後に型消去
    let patches: Vec<Box<dyn PolyPatch>> =
        walls.into_iter().map(|p| Box::new(p) as _)
        .chain(cyclics.into_iter().map(|p| Box::new(p) as _))
        .chain(processors.into_iter().map(|p| Box::new(p) as _))
        .collect();

    let poly = PolyMesh { primitive, patches, /* ... */ };
    FvMesh::new(poly)
}
```

### 動的メッシュでの再交換

構築後のジオメトリ再交換は `CoupledPatch` trait のメソッド経由で行う。ダウンキャスト不要。

```rust
impl FvMesh {
    pub fn move_points(&mut self, new_points: Vec<Vector>, world: &SystemCommunicator) {
        self.poly.primitive.set_points(new_points);
        self.clear_geometry();

        let centers = self.poly.primitive.cell_centers();

        for &i in &self.processor_patch_indices {
            let coupled = self.poly.patches[i].as_coupled_mut().unwrap();
            let local: Vec<_> = coupled.face_cells()
                .iter()
                .map(|&c| centers[c])
                .collect();
            let remote = mpi_exchange(world, coupled.neighbor_rank().unwrap(), &local);
            coupled.set_neighbor_cell_centers(remote);
        }
    }
}
```

---

## 既存設計との接合

| 既存設計 | 関係 |
|----------|------|
| `BoundaryPatch<T>` enum (Physical / Processor) | フィールド層の概念。メッシュ層のパッチとは別。フィールドが `FvPatch` のジオメトリを参照する |
| `PhysicalBC<T>` trait | `evaluate()` に `&dyn FvPatch` を渡すことでパッチジオメトリにアクセス |
| `evaluate_boundaries(world)` | メッシュ層は不変。フィールド層が値の MPI 交換を行う |
| `inventory` + `dyn Trait` | パッチファクトリ（I/O時）と境界条件ファクトリ（PhysicalBC）に適用 |
| `FvMatrix` | `mesh: &'mesh FvMesh` を保持し、`LduMesh` trait 経由でアドレッシングを取得 |

---

## 設計判断一覧

| 判断 | 結論 | 根拠 |
|------|------|------|
| C++ 継承 → Rust | 合成 + 委譲メソッド | Deref 乱用を避け明示的にアクセス |
| 遅延計算 | `OnceCell` | 構築後不変、`&self` で初期化、`Sync` |
| 面の表現 | `Vec<usize>`（任意多角形） | OpenFOAM 同等の汎用性 |
| パッチ種別 | `dyn PolyPatch` trait | 振る舞い（delta計算、フック）が種別ごとに異なる |
| coupled の統一 | `CoupledPatch` サブtrait | cyclic/processor を離散化コードから統一的に扱う |
| FV パッチ層 | `dyn FvPatch` trait | パッチごとの FV ジオメトリ計算差異を吸収 |
| パッチ × `inventory` | I/O 時のファクトリ登録に使用 | 文字列 → 具象型の生成 |
| LduMesh trait 配置 | mesh クレート | orphan rule 制約 |
| 並列ジオメトリ交換 | 型消去前に具象型で実施 | ダウンキャスト不要 |
| 動的メッシュでの再交換 | `CoupledPatch::set_neighbor_cell_centers` | trait 経由で型消去後も可能 |
| globalMeshData | `Option<GlobalMeshData>` in PolyMesh | シリアル時 None、並列時に構築 |
| ゾーン | 3種を PolyMesh に保持 | MRF/ソース項の基盤 |
| 動的メッシュプラグイン | `Option<Box<dyn MeshMover>>` 等 | 当面 None、拡張点として確保 |

---

## OpenFOAM との対比

### 十分に担保できている点

| 観点 | 根拠 |
|------|------|
| 任意多面体メッシュ | Face = 任意多角形、セル形状に制約なし |
| 3層の責務分離 | トポロジ / メッシュ管理 / FV離散化が明確に分離 |
| 遅延計算 | OnceCell で demand-driven パターンを再現 |
| ゾーン | 3種を PolyMesh に保持、MRF等の基盤あり |
| coupled パッチ統一 | CoupledPatch trait で cyclic/processor を統一 |
| FV パッチ層 | FvPatch trait でパッチごとのジオメトリ計算差異を吸収 |
| パッチのメッシュ操作フック | trait メソッドで種別ごとの更新処理を定義可能 |
| LDU アドレッシング | ソルバーとの接合が trait で定義済み |
| 並列ジオメトリ | ProcessorPatch が対面データを保持、構築時に交換 |

### 後段で対応が必要な点

| 観点 | 内容 | 対応時期 |
|------|------|----------|
| パッチローカルトポロジ | primitivePatch 相当（壁関数、非適合結合で必要） | 壁関数実装時 |
| BoundaryMesh ラッパー | 名前解決・グループ解決の一括操作 | I/O 実装時 |
| OnceCell リセット | 動的メッシュ導入時に LazyCache 型へ置き換え | 動的メッシュ実装時 |
| fvPatch と PhysicalBC の接合 | `PhysicalBC::evaluate()` に `&dyn FvPatch` を渡す経路 | fields-boundary (Spec 2-3) |

---

## 参考

- OpenFOAM ソース: `src/OpenFOAM/meshes/primitiveMesh/`, `src/OpenFOAM/meshes/polyMesh/`, `src/finiteVolume/fvMesh/`
- 既存設計: `docs-dev/spec-ideas/rust_cfd_mesh_field_parallel.md`
- 実行時選択: `docs-dev/spec-ideas/rust_cfd_runtime_selection.md`
- 実装計画: `docs-dev/implementation-plan.md`
