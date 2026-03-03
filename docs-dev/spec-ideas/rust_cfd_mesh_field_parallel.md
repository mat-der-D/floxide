# Rust CFD メッシュ・フィールド・並列化の設計

議論を経て確定した設計判断をここに記録する。

---

## 1. 並列化技術の選定：rsmpi + rayon ハイブリッド

### 分散メモリ並列：rsmpi（`mpi` クレート）

ドメイン分割＋halo 交換による分散並列には MPI が唯一の現実的選択肢である。
Rust の MPI バインディングとして `rsmpi`（crates.io 名 `mpi`、v0.8.1）を採用する。

**採用しない手法とその理由：**
- Lamellar（PGAS）：α段階、halo 交換に不向き
- Constellation（アクターモデル）：TCP ベース、未成熟
- Tokio + TCP：レイテンシが MPI の 10-50 倍、集団通信の自前実装が必要
- ZeroMQ / NNG：マイクロサービス向け、HPC 最適化なし

**rsmpi の採用根拠：**
- 唯一の成熟した Rust MPI バインディング（活発にメンテナンス）
- `#[derive(Equivalence)]` でテンソル型をそのまま送受信可能
- scope ベースの非同期通信が Rust の借用規則と整合する
- OpenMPI / MPICH 等の既存 MPI 実装を活用可能

### ノード内並列：rayon

ループ並列（セル・フェイスの並列処理）に `par_iter` を使用する。

### 組み合わせ

MPI は `Threading::Funneled` で初期化する。MPI 呼び出しはメインスレッドのみ、rayon は計算に使用する。

```rust
let (universe, _) = mpi::initialize_with_threading(mpi::Threading::Funneled).unwrap();

// rayon のスレッド数をランクあたりのコア数に制限
let pool = rayon::ThreadPoolBuilder::new()
    .num_threads(cores_per_node / ranks_per_node)
    .build().unwrap();
```

---

## 2. フィールドとメッシュのライフタイム管理

### 方針：メッシュを借用、MPI ライフタイムから分離

```
mpi::Universe                          // プログラム生存期間
  └─ world: &Communicator              // universe から借用
       └─ FvMesh                       // 各ランクが所有（setup 後は不変）
            └─ VolumeField<'mesh, T, State>  // mesh を借用
```

- **メッシュは不変参照で保持する。** `Arc<FvMesh>` ではなくライフタイムパラメータ `'mesh` を使用する。メッシュは setup 後に変更されないため、共有所有権は不要。
- **メッシュは MPI ライフタイムに依存しない。** メッシュは隣接ランク番号（`i32`）等の接続情報を保持するが、`Communicator` への参照は持たない。
- **プロセッサ境界もランク番号のみ保持する。** 通信ハンドル（`Process` オブジェクト）は `evaluate_boundaries` 内で `world.process_at_rank(rank)` から都度取得する。

---

## 3. 境界条件の鮮度を型で保証する（typestate pattern）

### 型状態の定義

フィールドの境界条件が最新かどうかを型パラメータで表現する。

```rust
struct Fresh;  // 全境界（物理＋プロセッサ）が評価済み
struct Stale;  // 内部値が変更され、境界の再評価が必要

struct VolumeField<'a, T, S> {
    mesh: &'a FvMesh,
    internal: Vec<T>,
    boundaries: Vec<BoundaryPatch<T>>,
    _state: PhantomData<S>,
}
```

### 状態遷移

```
[構築] → Stale
            │
            ▼
  evaluate_boundaries(world)
            │
            ▼
         Fresh ←─────────────────────┐
            │                        │
            ▼                        │
  map_internal(f) / solve()    evaluate_boundaries(world)
            │                        │
            ▼                        │
         Stale ──────────────────────┘
```

- `evaluate_boundaries` は `self` を消費し `Fresh` を返す → Stale なフィールドが誤って使い続けられることを型レベルで防止
- `map_internal` は `self` を消費し `Stale` を返す → 内部値変更後の境界再評価を強制
- **離散化演算（`gradient` 等）は `Fresh` のみ受け付ける** → 境界が未評価のフィールドが演算に使われるとコンパイルエラー

### 型状態で保証すること / ランタイムに任せること

| 型で保証 | ランタイムに任せる |
|---------|---------------|
| 境界条件の鮮度（Fresh / Stale） | 具体的な通信パターン |
| フィールドが属するメッシュ（ライフタイム） | ドメイン分割の詳細 |
| 離散化演算への入力が有効であること | 通信のタイミング最適化 |

---

## 4. 境界条件の統一方式：enum による内部分離＋フィールドレベルの統一

### 設計判断

物理境界条件とプロセッサ境界を **enum で区別しつつ、単一リストに保持** する。

```rust
enum BoundaryPatch<T> {
    Physical(Box<dyn PhysicalBC<T>>),   // 多態（実行時選択）
    Processor(ProcessorPatch<T>),        // 具象型（MPI 直接使用）
}
```

### 設計根拠

OpenFOAM は物理境界とプロセッサ境界を同一の仮想関数インターフェース（`fvPatchField`）で統一しているが、Rust では以下の理由でこれを踏襲しない：

1. **dyn 互換性の問題。** rsmpi の `Communicator` trait はジェネリックメソッドを持つため `dyn` 化できない。`BoundaryCondition::evaluate(&mut self, comm: &dyn Communicator)` という統一インターフェースは成立しない。
2. **物理境界とプロセッサ境界は機構が根本的に異なる。** 前者は純粋なローカル計算、後者は MPI 通信。C++ では仮想関数で隠蔽できるが、Rust ではこの隠蔽が型システムと摩擦する。
3. **自前の通信抽象層（CommChannel 等）は不要。** rsmpi を使うと決めた以上、中間層は摩擦を生むだけ。

### 統一性の実現方法

統一は **trait レベルではなくフィールドレベル** で行う：

- **呼び出し側が見る世界：** `field.evaluate_boundaries(world)` で全境界を一括処理。物理/プロセッサの区別は不要。
- **離散化演算：** `field.boundary_values(patch_id)` で統一アクセス。`gradient` 等はパッチの種類を意識しない。
- **内部実装：** `evaluate_boundaries` 内で enum を match し、物理BC は `evaluate` を呼び、プロセッサ境界は MPI 通信を実行する。

```rust
// 離散化演算は境界の種類を意識しない
fn gradient(field: &VolumeField<f64, Fresh>) -> Vec<Vector> {
    // ...
    for patch_id in 0..mesh.n_patches() {
        let patch_vals = field.boundary_values(patch_id);  // 統一アクセス
        // 物理パッチもプロセッサパッチも同じコード
    }
}
```

### 物理境界条件の trait

MPI に依存しない純粋なインターフェースとして定義する。実行時選択は `inventory` + `dyn PhysicalBC<T>` で実現する。

```rust
trait PhysicalBC<T> {
    fn evaluate(&mut self, internal: &[T]);
    fn patch_values(&self) -> &[T];
}
```

### プロセッサ境界の構造

具象型として定義する。`Communicator` への参照を保持せず、ランク番号のみ持つ。

```rust
struct ProcessorPatch<T> {
    neighbor_rank: i32,
    face_cells: Vec<usize>,   // 送信すべきローカルセル
    send_buf: Vec<T>,
    recv_buf: Vec<T>,          // 受信した隣接プロセスの値
}
```

---

## 5. 通信と境界評価の重畳：当面ブロッキングで十分

### OpenFOAM の実態

OpenFOAM の `GeometricBoundaryField::evaluate()` は nonBlocking モードで2フェーズ評価を行う：

1. `initEvaluate`：全パッチに呼ぶ。プロセッサパッチが非同期 send/recv を開始。物理パッチは空実装。
2. `waitRequests`：**全通信の完了を待つ**。
3. `evaluate`：全パッチに呼ぶ。プロセッサパッチは受信データ取得。物理パッチはここで計算。

つまり、**物理BC の計算は通信完了後に行われており、通信との重畳は実質的に行われていない。**

### Rust での方針

v2 プロトタイプのブロッキング方式は OpenFOAM と同等の実行順序であり、性能上の問題はない。

将来、通信と計算の重畳が必要になった場合でも、`evaluate_boundaries` の **内部実装だけ** を変更すれば対応できる。外部インターフェース（typestate の Fresh/Stale、`boundary_values` による統一アクセス）は変わらない。

---

## プロトタイプ

設計の検証用コード：[`unified_field_v2.rs`](./unified_field_v2.rs)

---

## 今後の課題

本設計で未解決の課題は [未決定の設計課題](./rust_cfd_open_questions.md) を参照。

---

## 参考

- [Rust CFD 型システムと記法](./rust_cfd_types_and_notation.md)
- [テンソル型システム](./rust_cfd_tensor_types.md)
- [実行時選択メカニズム](./rust_cfd_runtime_selection.md)
- [OpenFOAM 責務分解の分析](./openfoam_responsibility_decomposition.md)
