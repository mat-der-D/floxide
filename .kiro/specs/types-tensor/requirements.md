# Requirements Document

## Introduction

本仕様は `dugong-types` クレートの `tensor` モジュールに関する要件を定義する。
CFD（数値流体力学）フレームワークの基盤となるテンソル型群（Scalar, Vector, Tensor, SymmTensor, SphericalTensor）の定義と、
それらに対する基本演算・異型間演算・型変換を提供する。

プロジェクト全体の最下層に位置し、外部クレートへの依存を持たない純粋な数学型ライブラリである。

## Requirements

### Requirement 1: テンソル型の定義

**Objective:** As a フレームワーク開発者, I want CFD で必要な全ランクのテンソル型を型安全に区別できる構造体として利用したい, so that 物理量の型レベルでの誤用を防止できる。

#### Acceptance Criteria

1. The tensor module shall `Scalar` を `f64` の型エイリアスとして定義する。
2. The tensor module shall `Vector` を `[f64; 3]` を内部に持つ newtype 構造体として定義する。
3. The tensor module shall `Tensor` を `[f64; 9]` を内部に持つ row-major 順の newtype 構造体として定義する。
4. The tensor module shall `SymmTensor` を `[f64; 6]` を内部に持つ newtype 構造体として定義する。
5. The tensor module shall `SphericalTensor` を `f64` を内部に持つ newtype 構造体として定義する。
6. The tensor module shall すべてのテンソル型に `Copy`, `Clone`, `Debug`, `PartialEq` を実装する。
7. The tensor module shall 各テンソル型に対してコンストラクタ（`new`）を提供する。
8. The tensor module shall 各テンソル型に対して内部データへのアクセサを提供する。

---

### Requirement 2: 同型基本演算

**Objective:** As a フレームワーク開発者, I want 同じテンソル型同士の加減算およびスカラー倍・符号反転を標準演算子で記述したい, so that 数学的記法に近いコードを書ける。

#### Acceptance Criteria

1. The tensor module shall `Vector`, `Tensor`, `SymmTensor`, `SphericalTensor` に `Add<Self, Output=Self>` を実装する。
2. The tensor module shall `Vector`, `Tensor`, `SymmTensor`, `SphericalTensor` に `Sub<Self, Output=Self>` を実装する。
3. The tensor module shall `Vector`, `Tensor`, `SymmTensor`, `SphericalTensor` に `Neg<Output=Self>` を実装する。
4. The tensor module shall `Vector`, `Tensor`, `SymmTensor`, `SphericalTensor` に `Mul<f64, Output=Self>`（右スカラー倍）を実装する。
5. The tensor module shall `f64` に対して `Mul<Vector>`, `Mul<Tensor>`, `Mul<SymmTensor>`, `Mul<SphericalTensor>`（左スカラー倍）を実装する。
6. The tensor module shall `Vector`, `Tensor`, `SymmTensor`, `SphericalTensor` に `Div<f64, Output=Self>`（スカラー除算）を実装する。
7. The tensor module shall 同型の `AddAssign` および `SubAssign` を実装する。
8. The tensor module shall 同型の `MulAssign<f64>` および `DivAssign<f64>` を実装する。

---

### Requirement 3: 異型間演算

**Objective:** As a フレームワーク開発者, I want テンソルランクが異なる型同士の物理的に意味のある演算（加算・単縮約・二重縮約・テンソル積・クロス積）を型安全に行いたい, so that 不正なテンソル演算がコンパイル時に排除される。

**Design Principle:** `Mul` trait (`*` 演算子) は単縮約（single contraction）を表す。結果のランクは `rank(A) + rank(B) - 2` となる。この規則により全ての型の組み合わせで `*` の意味が統一される。ランク変化が異なる演算（テンソル積・クロス積・二重縮約）は名前付きメソッドで提供する。

#### Acceptance Criteria

**異型間加算・減算:**

1. The tensor module shall `SymmTensor + SphericalTensor` → `SymmTensor` の演算を提供する。
2. The tensor module shall `SphericalTensor + SymmTensor` → `SymmTensor` の演算を提供する。
3. The tensor module shall `Tensor + SymmTensor` → `Tensor` の演算を提供する。
4. The tensor module shall `Tensor + SphericalTensor` → `Tensor` の演算を提供する。
5. The tensor module shall 上記の異型間加算に対応する減算も提供する。

**単縮約（`*` 演算子、rank(A) + rank(B) - 2）:**

6. The tensor module shall `Vector * Vector` → `f64` の内積を `Mul` trait で提供する。
7. The tensor module shall `Tensor * Vector` → `Vector` の行列・ベクトル積を `Mul` trait で提供する。
8. The tensor module shall `Vector * Tensor` → `Vector` のベクトル・行列積を `Mul` trait で提供する。
9. The tensor module shall `Tensor * Tensor` → `Tensor` の行列積を `Mul` trait で提供する。
10. The tensor module shall `SymmTensor * Vector` → `Vector` の縮約を `Mul` trait で提供する。
11. The tensor module shall `SymmTensor * SymmTensor` → `Tensor` の行列積を `Mul` trait で提供する。

**二重縮約（名前付きメソッド、rank(A) + rank(B) - 4）:**

12. The tensor module shall テンソルの二重縮約 `Tensor : Tensor` → `f64` を `double_dot` メソッドで提供する。
13. The tensor module shall `SymmTensor : SymmTensor` → `f64` の二重縮約を `double_dot` メソッドで提供する。

**テンソル積（名前付きメソッド、rank(A) + rank(B)）:**

14. The tensor module shall テンソル積 `Vector ⊗ Vector` → `Tensor` を `outer` メソッドで提供する。

**クロス積（名前付きメソッド、rank(A) + rank(B) - 1）:**

15. The tensor module shall ベクトルクロス積 `Vector × Vector` → `Vector` を `cross` メソッドで提供する。

---

### Requirement 4: 型変換メソッド

**Objective:** As a フレームワーク開発者, I want テンソルに対する標準的な代数的分解・変換操作をメソッドとして利用したい, so that CFD で頻出するテンソル操作を簡潔に記述できる。

#### Acceptance Criteria

1. The tensor module shall `Tensor` に対称部分を返す `symm()` → `SymmTensor` メソッドを提供する。
2. The tensor module shall `Tensor` に2倍対称部分を返す `two_symm()` → `SymmTensor` メソッドを提供する。
3. The tensor module shall `Tensor` に球面部分を返す `sph()` → `SphericalTensor` メソッドを提供する。
4. The tensor module shall `Tensor` に反対称部分を返す `skew()` → `Tensor` メソッドを提供する。
5. The tensor module shall `Tensor` に偏差部分を返す `dev()` → `Tensor` メソッドを提供する。
6. The tensor module shall `Tensor` にトレースを返す `trace()` → `f64` メソッドを提供する。
7. The tensor module shall `Tensor` に行列式を返す `det()` → `f64` メソッドを提供する。
8. The tensor module shall `Tensor` に転置を返す `transpose()` → `Tensor` メソッドを提供する。
9. The tensor module shall `SymmTensor` に `trace()` → `f64` メソッドを提供する。
10. The tensor module shall `SymmTensor` に `det()` → `f64` メソッドを提供する。
11. The tensor module shall `SymmTensor` に `dev()` → `SymmTensor` メソッドを提供する。
12. The tensor module shall `SymmTensor` に `sph()` → `SphericalTensor` メソッドを提供する。
13. The tensor module shall `Vector` にマグニチュード（ユークリッドノルム）を返す `mag()` → `f64` メソッドを提供する。
14. The tensor module shall `Vector` に二乗マグニチュードを返す `mag_sqr()` → `f64` メソッドを提供する。
15. The tensor module shall `Tensor` にフロベニウスノルムを返す `mag()` → `f64` メソッドを提供する。

---

### Requirement 5: From 変換

**Objective:** As a フレームワーク開発者, I want テンソルランクが低い型から高い型への安全な型変換を `From` trait で利用したい, so that 型変換が Rust の慣用的パターンで行える。

#### Acceptance Criteria

1. The tensor module shall `From<SphericalTensor> for SymmTensor` を実装する（対角成分に展開）。
2. The tensor module shall `From<SphericalTensor> for Tensor` を実装する（対角成分に展開）。
3. The tensor module shall `From<SymmTensor> for Tensor` を実装する（対称テンソルの完全展開）。

---

### Requirement 6: 特殊値コンストラクタ

**Objective:** As a フレームワーク開発者, I want 零テンソル・単位テンソルなどの特殊値を簡潔に生成したい, so that テスト記述や初期化コードの可読性を高められる。

#### Acceptance Criteria

1. The tensor module shall `Vector::zero()` を提供し、全成分がゼロのベクトルを返す。
2. The tensor module shall `Tensor::zero()` を提供し、全成分がゼロのテンソルを返す。
3. The tensor module shall `Tensor::identity()` を提供し、3×3 単位行列を返す。
4. The tensor module shall `SymmTensor::zero()` を提供し、全成分がゼロの対称テンソルを返す。
5. The tensor module shall `SymmTensor::identity()` を提供し、対角成分が 1 の対称テンソルを返す。
6. The tensor module shall `SphericalTensor::zero()` を提供し、値がゼロの球面テンソルを返す。
7. The tensor module shall `SphericalTensor::identity()` を提供し、値が 1 の球面テンソルを返す。

---

### Requirement 7: 数値正確性と品質

**Objective:** As a フレームワーク開発者, I want テンソル演算の数値的正確性が検証されていることを保証したい, so that CFD シミュレーション結果の信頼性が担保される。

#### Acceptance Criteria

1. The tensor module shall 全ての同型演算（加算・減算・スカラー倍・符号反転）について成分ごとの数値正確性テストを提供する。
2. The tensor module shall 全ての異型間演算について手計算と一致する数値テストを提供する。
3. The tensor module shall 型変換メソッド（`symm`, `skew`, `dev`, `sph`, `trace`, `det`, `transpose`）について既知の入力に対する正確性テストを提供する。
4. The tensor module shall 特殊値（零テンソル・単位テンソル）の代数的性質をテストする（例: `A + zero == A`, `identity * v == v`）。
5. The tensor module shall `From` 変換の往復一貫性をテストする（例: `SymmTensor` → `Tensor` → `symm()` が元と一致）。
6. The tensor module shall 浮動小数点比較に適切な許容誤差を使用する。
7. The tensor module shall `cargo clippy` で警告がないこと。
8. The tensor module shall 全ての公開 API にドキュメントコメント（`///`）を付与する。
