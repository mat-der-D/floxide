# 並列計算パターン

## アーキテクチャ

**rsmpi (MPI) + rayon ハイブリッド**: 分散メモリ並列（ドメイン分割＋halo 交換）に MPI、ノード内ループ並列に rayon を使用。

## MPI パターン

### 初期化

```rust
// Threading::Funneled: MPI 呼び出しはメインスレッドのみ
let (universe, _) = mpi::initialize_with_threading(mpi::Threading::Funneled).unwrap();
let world = universe.world();
```

### 通信の原則

1. **ブロッキング通信を基本とする**: OpenFOAM も実質的に通信と計算の重畳を行っていないため同等
2. **`Communicator` への参照をフィールドに保持しない**: ランク番号（`i32`）のみ保持し、通信時に `world.process_at_rank(rank)` で都度取得
3. **`#[derive(Equivalence)]`**: テンソル型（`Vector`, `Tensor` 等）に MPI データ型を自動導出

### halo 交換パターン

```rust
// evaluate_boundaries 内でプロセッサ境界の通信を実行
fn evaluate_boundaries(self, world: &Communicator) -> VolumeField<'a, T, Fresh> {
    for patch in &mut self.boundaries {
        match patch {
            BoundaryPatch::Physical(bc) => bc.evaluate(&self.internal),
            BoundaryPatch::Processor(proc) => {
                // 送信データ準備 → send/recv → 受信データ格納
                proc.exchange(world);
            }
        }
    }
    // typestate: Stale → Fresh
}
```

### 将来の非同期通信

- `evaluate_boundaries` の **内部実装のみ** を変更すれば対応可能
- 外部インターフェース（typestate, `boundary_values`）は不変
- 通信と物理 BC 計算の重畳は、必要性が実測で確認されてから実装

## rayon パターン

### ノード内並列

```rust
// スレッド数をランクあたりのコア数に制限
let pool = rayon::ThreadPoolBuilder::new()
    .num_threads(cores_per_node / ranks_per_node)
    .build().unwrap();
```

- セル・フェイスのループ並列に `par_iter` / `par_chunks` を使用
- MPI 呼び出しは rayon のスレッドから行わない（`Funneled` モード）

## ライフタイム構造

```
mpi::Universe                          // プログラム生存期間
  └─ world: &Communicator              // universe から借用
       └─ FvMesh                       // 各ランクが所有（setup 後は不変）
            └─ VolumeField<'mesh, T, State>  // mesh を借用
```

- **`Arc` 不使用**: メッシュは setup 後に不変のため、共有所有権は不要
- **`&'mesh FvMesh`**: フィールドはライフタイムパラメータでメッシュを借用
- **メッシュは MPI ライフタイムに依存しない**: 隣接ランク番号等の接続情報のみ保持

## スレッド安全性の原則

1. `VolumeField` は `Send` だが `Sync` ではない — 同時書き込みを防止
2. MPI 通信は `evaluate_boundaries` 内にカプセル化 — 散在する通信コードを防止
3. rayon の並列イテレータは内部値（`internal: Vec<T>`）のスライスに対してのみ使用

---
_並列化のパターンと原則に焦点。通信プロトコルの詳細は仕様に委ねる_
