# CFD 固有の規約

## OpenFOAM からの継承と変更

### 継承する思想
- **「コードは数学のように見えるべき」**: PDE の離散化コードが数学的記法に近い形で読めること
- **有限体積法（FVM）のフレームワーク設計**: メッシュ → フィールド → 離散化 → ソルバーの階層構造
- **ユーザーインターフェースとしての設定ファイル**: 実行時にモデル・スキーム・ソルバーを選択

### 継承しない設計
- **dlopen によるプラグイン**: 静的リンク＋`inventory` による実行時選択に置換
- **objectRegistry（文字列ベースのフィールド検索）**: 型付き依存性注入に置換
- **`dimensionSet` の実行時検査**: const generics によるコンパイル時検査に置換
- **仮想関数による境界条件の完全統一**: enum による内部分離＋フィールドレベルの統一

## 命名規約（CFD ドメイン固有）

### 物理量の変数名

OpenFOAM の慣例に従い、CFD コミュニティで共通の変数名を使用する：

| 変数名 | 物理量 | 型 |
|--------|--------|-----|
| `p` | 圧力 | `Scalar` |
| `U` / `u` | 速度 | `Vector` |
| `phi` | 質量流束 | `Scalar`（面上） |
| `rho` | 密度 | `Scalar` |
| `mu` | 粘性 | `Scalar` |
| `nu` | 動粘性 | `Scalar` |
| `k` | 乱流エネルギー | `Scalar` |
| `omega` | 比散逸率 | `Scalar` |
| `T` | 温度 | `Scalar` |

**Rust の規約との調整**: Rust は snake_case を要求するため、大文字の慣例変数名（`U`）は小文字（`u`）を使用する。型名と混同のリスクがある場合はフルネーム（`velocity`）を使用。

### 離散化演算子の命名

```rust
// 陰的（行列を生成）
fvm.ddt(field)           // 時間微分
fvm.div(flux, field)     // 発散（対流項）
fvm.laplacian(coeff, field)  // ラプラシアン（拡散項）

// 陽的（フィールド値を返す）
fvc.grad(field)          // 勾配
fvc.div(surface_field)   // 発散
fvc.curl(field)          // 回転
```

## 次元の表記

### 型エイリアスによる物理量の定義

```rust
// SI 基本次元: M (質量), L (長さ), T (時間)
type Pressure  = Dim<f64,      1, -1, -2>;  // kg/(m·s²) = Pa
type Velocity  = Dim<Vector,   0,  1, -1>;  // m/s
type Density   = Dim<f64,      1, -3,  0>;  // kg/m³
```

- 型エイリアスを `types` クレートに集約
- 中間式の次元は型推論に委ね、明示しない

## 数学表記のコード化パターン

### PDE の記述

```rust
// 数学: ∂(ρU)/∂t + ∇·(φU) - ∇·(μ∇U) = -∇p
let u_eqn = (fvm.ddt(&rho, &u) + fvm.div(&phi, &u) - fvm.laplacian(&mu, &u))
    .rhs(fvc.grad(&p));
u_eqn.solve();
```

### 次元消去の境界

- `fvm` 演算子が「物理の世界」（次元付き）→「数学の世界」（次元なし `FvMatrix`）への変換点
- `FvMatrix` とソルバーは物理次元を知らない — 純粋な線形代数

---
_CFD ドメインのパターンに焦点。個々のモデルや方程式の詳細は仕様に委ねる_
