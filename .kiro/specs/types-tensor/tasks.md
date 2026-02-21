# Implementation Plan

- [x] 1. テンソル型群の定義とモジュール構造の構築
  - tensor モジュール構造（mod.rs と各サブモジュールファイル）を作成し、lib.rs から公開する
  - Scalar を f64 の型エイリアスとして定義する
  - Vector, Tensor, SymmTensor, SphericalTensor を newtype 構造体として定義し、Copy, Clone, Debug, PartialEq を derive する
  - 各型に new() コンストラクタを実装する
  - 各型に内部データアクセサ（as_array(), value(), 成分名メソッド）を実装する
  - 全公開 API にドキュメントコメント（`///`）を付与する
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 1.8, 7.8_

- [x] 2. 同型基本演算の実装
- [x] 2.1 (P) 四則演算・符号反転・スカラー演算の実装
  - 4 つのテンソル型（Vector, Tensor, SymmTensor, SphericalTensor）に成分ごとの加算（Add）・減算（Sub）・符号反転（Neg）を実装する
  - 右スカラー倍（Mul<f64>）・左スカラー倍（f64 に対する Mul<T>）・スカラー除算（Div<f64>）を実装する
  - SphericalTensor は単一 f64 値に対する演算として実装する
  - 全公開 API にドキュメントコメントを付与する
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 7.8_

- [x] 2.2 複合代入演算子の実装
  - 4 つのテンソル型に AddAssign・SubAssign を実装する
  - 4 つのテンソル型に MulAssign<f64>・DivAssign<f64> を実装する
  - _Requirements: 2.7, 2.8_

- [x] 3. 異型間演算の実装
- [x] 3.1 (P) 異型間加算・減算の実装
  - SymmTensor と SphericalTensor の相互加算・減算を実装する（SphericalTensor を対角成分に展開）
  - Tensor と SymmTensor の加算・減算を実装する（SymmTensor を 9 成分に展開）
  - Tensor と SphericalTensor の加算・減算を実装する（SphericalTensor を対角に展開）
  - 低ランク型を高ランク型に暗黙展開する規則に従い、結果型は高ランク側とする
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5_

- [x] 3.2 単縮約演算の実装（Mul trait）
  - Vector 同士の内積（Vector * Vector → f64）を実装する
  - Tensor と Vector の行列・ベクトル積を双方向で実装する（Tensor * Vector → Vector、Vector * Tensor → Vector）
  - Tensor 同士の行列積（Tensor * Tensor → Tensor）を実装する
  - SymmTensor と Vector の縮約（SymmTensor * Vector → Vector）を 6 成分から直接計算する
  - SymmTensor 同士の行列積（SymmTensor * SymmTensor → Tensor）を実装する（結果は非対称）
  - 全演算が rank(A) + rank(B) - 2 の規則に従うことを確認する
  - _Requirements: 3.6, 3.7, 3.8, 3.9, 3.10, 3.11_

- [x] 3.3 二重縮約・テンソル積・クロス積の実装
  - Tensor に二重縮約メソッド（double_dot）を実装する（全 9 成分の要素積の和）
  - SymmTensor に二重縮約メソッド（double_dot）を実装する（対称性により非対角成分を 2 倍）
  - Vector にテンソル積メソッド（outer）を実装する（T_ij = a_i * b_j）
  - Vector にクロス積メソッド（cross）を実装する
  - _Requirements: 3.12, 3.13, 3.14, 3.15_

- [x] 4. 型変換メソッドと From 変換の実装
- [x] 4.1 (P) Tensor の分解・変換メソッドの実装
  - 対称部分（symm → SymmTensor）と 2 倍対称部分（two_symm → SymmTensor）を実装する
  - 反対称部分（skew → Tensor）を実装する（対角成分は常にゼロ）
  - 球面部分（sph → SphericalTensor）と偏差部分（dev → Tensor）を実装する
  - トレース（trace → f64）と行列式（det → f64、サルスの方法）を実装する
  - 転置（transpose → Tensor）を実装する
  - フロベニウスノルム（mag → f64、sqrt(T:T)）を実装する
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7, 4.8, 4.15_

- [x] 4.2 SymmTensor・Vector メソッドと From 変換の実装
  - SymmTensor に trace, det, dev, sph メソッドを実装する
  - Vector に mag（ユークリッドノルム）と mag_sqr（二乗マグニチュード）を実装する
  - SphericalTensor → SymmTensor の From 変換を実装する（対角成分に展開）
  - SphericalTensor → Tensor の From 変換を実装する（対角成分に展開）
  - SymmTensor → Tensor の From 変換を実装する（対称テンソルの完全 9 成分展開）
  - _Requirements: 4.9, 4.10, 4.11, 4.12, 4.13, 4.14, 5.1, 5.2, 5.3_

- [x] 5. (P) 特殊値コンストラクタの実装
  - Vector, Tensor, SymmTensor, SphericalTensor に zero() を const fn として実装する
  - Tensor, SymmTensor, SphericalTensor に identity() を const fn として実装する
  - 全公開 API にドキュメントコメントを付与する
  - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5, 6.6, 6.7, 7.8_

- [x] 6. テストスイートの実装と品質保証
- [x] 6.1 (P) 型定義と同型演算のテスト
  - 浮動小数点比較用テストヘルパー関数（assert_approx_eq）を定義する
  - コンストラクタ・アクセサ・Copy セマンティクスの正確性テストを実装する
  - 成分ごとの加減算・スカラー倍・符号反転の数値テストを実装する
  - 左スカラー倍の可換性テスト（2.0 * v == v * 2.0）を実装する
  - 複合代入演算子の等価性テスト（a += b と a = a + b の一致）を実装する
  - _Requirements: 7.1, 7.6_

- [x] 6.2 (P) 異型間演算のテスト
  - 異型間加算・減算の手計算検証テストを実装する
  - Vector 内積の数値テストと可換性検証を実装する
  - Tensor-Vector 積、Tensor 行列積、SymmTensor 行列積の手計算検証テストを実装する
  - 二重縮約の恒等式テスト（T.double_dot(identity) == T.trace()）を実装する
  - テンソル積・クロス積の手計算検証と代数的性質テスト（反可換性・直交性）を実装する
  - _Requirements: 7.2_

- [x] 6.3 (P) 型変換・From 変換・特殊値のテスト
  - symm + skew == T の分解完全性テストを実装する
  - dev().trace() ≈ 0.0 の偏差テンソル性質テストを実装する
  - det の既知値テストと transpose の検証テストを実装する
  - From 変換の往復一貫性テスト（SymmTensor → Tensor → symm() が元と一致）を実装する
  - 零テンソルの加法単位元性質テスト（A + zero == A）と単位テンソルの乗法単位元性質テスト（identity * v == v）を実装する
  - _Requirements: 7.3, 7.4, 7.5_

- [x] 6.4 品質チェック
  - cargo clippy で警告がないことを確認する
  - 全公開 API にドキュメントコメントが付与されていることを確認する
  - 全テストが通ることを確認する
  - _Requirements: 7.7, 7.8_
