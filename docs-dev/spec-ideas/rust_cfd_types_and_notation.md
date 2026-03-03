# Rust 設計ビジョン

OpenFOAM の設計哲学にインスパイアされた、Rust ネイティブな CFD フレームワークの設計方針。
OpenFOAM の翻訳ではなく、Rust の型システムを活かした re-imagination を目指す。

## 目次

1. [根本原則](#1-根本原則)
2. [次元システム — コンパイル時検査と型消去](#2-次元システム--コンパイル時検査と型消去)
3. [fvm / fvc 演算子の設計](#3-fvm--fvc-演算子の設計)
4. [PDE の方程式記法](#4-pde-の方程式記法)
5. [未決定の設計課題](#5-未決定の設計課題)

---

## 1. 根本原則

OpenFOAM の中核思想「コードは数学のように見えるべき」を継承しつつ、Rust の型システムで **数学的正しさの静的保証** を追加する。

| 軸 | C++ / OpenFOAM | Rust で目指すもの |
|---|---|---|
| 記法の見た目 | `fvm::ddt(rho, U) + fvm::div(phi, U)` | ほぼ同等（`fvm.ddt(&rho, &u) + fvm.div(&phi, &u)`） |
| 次元整合性 | 実行時検査（`dimensionSet`） | **コンパイル時検査**（`typenum` 型レベル整数） |
| 陰的/陽的の混同防止 | 名前空間の慣例（`fvm::` / `fvc::`） | **型システムで強制**（`ImplicitOps` / `ExplicitOps`） |
| スキーム選択の依存 | グローバル辞書からの暗黙参照 | **コンテキストオブジェクトへの明示的保持** |

---

## 2. 次元システム — コンパイル時検査と型消去

### 次元付き量

`typenum` 型レベル整数を用いてコンパイル時に次元を型パラメータとして保持する（stable Rust）。

```rust
/// 次元付き値。V は生の数値型（f64, Vector 等）
use typenum::{Integer, P1, P2, N1, N2, N3, Z0};
struct Dim<V, M: Integer, L: Integer, T: Integer> {
    value: V,
    _phantom: PhantomData<(M, L, T)>,
}

// typenum: P1=+1, P2=+2, P3=+3, N1=-1, N2=-2, N3=-3, Z0=0
type Pressure    = Dim<f64,    P1, N1, N2>;  // Pa
type Velocity    = Dim<Vector, Z0, P1, N1>;  // m/s
type Density     = Dim<f64,    P1, N3, Z0>;  // kg/m³
type Viscosity   = Dim<f64,    P1, N1, N1>;  // Pa·s
type MassFlux    = Dim<f64,    P1, N2, N1>;  // kg/(m²·s)
```

- 同次元の加算：コンパイル時に保証
- 異次元の乗除算：次元指数の算術をコンパイル時に解決
- 型エイリアスで人間可読な名前を付与
- 中間式の次元は型推論に委ねる（明示不要）

### 次元消去の境界

`fvm` 演算子が **次元付きフィールド → 次元なし行列** への変換境界となる。

```
物理の世界（次元付き）            数学の世界（次元なし）
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
VolumeField<Velocity>   ──fvm.ddt──→   FvMatrix<Vector>
VolumeField<Pressure>   ──fvc.grad──→  ExplicitField<Vector>
                                            ↓
                                       solver.solve()
                                            ↓
                                       解ベクトル: Vec<Vector>
```

**設計上の根拠：**

- FvMatrix は物理を知らない純粋な線形代数の器になる
- 同じ FvMatrix 型・同じソルバーで任意の物理方程式を解ける
- 不要な単相化（monomorphization）を防ぎ、コンパイル時間・バイナリサイズを抑制
- OpenFOAM の責務分解「原則 A：数学形式と物理内容の分離」と正確に一致

---

## 3. fvm / fvc 演算子の設計

### コンテキストオブジェクト方式

OpenFOAM の `fvm::` / `fvc::` 名前空間を Rust のオブジェクトに置き換える。
スキーム設定をコンテキストに保持し、PDE 式の中に数値手法の選択を混入させない。

```rust
struct ImplicitOps<'a> { mesh: &'a FvMesh, schemes: &'a Schemes }
struct ExplicitOps<'a> { mesh: &'a FvMesh, schemes: &'a Schemes }
```

### ImplicitOps（陰的離散化 → FvMatrix）

```rust
impl ImplicitOps<'_> {
    fn ddt<Q>(&self, field: &VolumeField<Q>) -> FvMatrix<Q::Value>;
    fn rho_ddt<C, Q>(&self, coeff: &VolumeField<C>, field: &VolumeField<Q>) -> FvMatrix<Q::Value>;
    fn div<C, Q>(&self, flux: &SurfaceField<C>, field: &VolumeField<Q>) -> FvMatrix<Q::Value>;
    fn laplacian<C, Q>(&self, coeff: &VolumeField<C>, field: &VolumeField<Q>) -> FvMatrix<Q::Value>;
}
```

### ExplicitOps（陽的評価 → フィールド値）

```rust
impl ExplicitOps<'_> {
    fn ddt<Q>(&self, field: &VolumeField<Q>) -> ExplicitField<Q::Value>;
    fn div<Q>(&self, field: &SurfaceField<Q>) -> ExplicitField<Q::Value>;
    fn grad<Q>(&self, field: &VolumeField<Q>) -> ExplicitField</* vector type */>;
    fn laplacian<C, Q>(&self, coeff: &VolumeField<C>, field: &VolumeField<Q>) -> ExplicitField<Q::Value>;
    fn curl<Q>(&self, field: &VolumeField<Q>) -> ExplicitField</* vector type */>;
}
```

### fvm と fvc で引数構造が自然に異なる理由

`fvm::div(flux, field)` が二引数なのは API の好みではなく、離散化の数学的構造の反映である。

- `flux`（SurfaceField）と `field`（VolumeField）は異なるメッシュ位置に住んでおり直接掛け算できない
- field のセル中心→面中心への補間スキーム（upwind, TVD 等）の選択に flux の方向情報が必要
- 陰的離散化では field が未知数、flux が既知係数として分離される（非線形方程式の線形化）

一方 `fvc::div(surface_field)` は一引数で成立する。面上で値が既知であれば、ガウスの定理で面積分を計算するだけだからである。

この非対称性により、ImplicitOps と ExplicitOps で同名メソッドのシグネチャが自然に異なり、**オーバーロードの問題はほぼ発生しない**。唯一 `ddt` に一引数/二引数のバリアントが残るが、`ddt` / `rho_ddt` の名前分けで十分対処可能。

---

## 4. PDE の方程式記法

### 基盤：.rhs() メソッド

Rust の `==`（PartialEq）は `bool` を返す必要があり、PDE の「左辺 = 右辺」としては使えない。
そこで `.rhs()` メソッドで右辺（ソース項）を設定する。

```rust
let u_eqn = (fvm.ddt(&rho, &u) + fvm.div(&phi, &u) - fvm.laplacian(&mu, &u))
    .rhs(fvc.grad(&p));
u_eqn.solve();
```

### 糖衣構文：pde! マクロ

`.rhs()` の上に構文マクロを提供し、数学的記法に近づける。

```rust
let u_eqn = pde!(
    fvm.ddt(&rho, &u) + fvm.div(&phi, &u) - fvm.laplacian(&mu, &u)
    == fvc.grad(&p)
);
u_eqn.solve();
```

マクロ内部では `==` の左右を分割してパースし、`.rhs()` 呼び出しに展開する。
段階的な開発が可能：まず `.rhs()` を実装し、マクロは後から追加する。

---

## 参考

- [OpenFOAM 設計思想](./openfoam_design_philosophy.md)
- [OpenFOAM モジュール構造](./openfoam_module_structure.md)
- [OpenFOAM 責務分解の分析](./openfoam_responsibility_decomposition.md)
- [ドキュメント整備計画](./document_plan.md)
