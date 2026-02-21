# 実装計画

- [x] 1. 依存関係の追加と `Dim` 構造体のコア実装
- [x] 1.1 Cargo.toml への依存関係追加
  - `[dependencies]` に `typenum = "1"` を追加する
  - `[dev-dependencies]` に `trybuild = "1"` を追加する
  - ワークスペースの共通設定（`version.workspace = true` 等）を維持する
  - _Requirements: 1.1, 4.1_

- [x] 1.2 `dimension` モジュール骨格の構築
  - `src/dimension.rs` を新規作成し `mod dim`・`mod quantity`・`mod ops`・`mod aliases`・`mod field_value_impl` を宣言し各 `pub use` を記述する
  - `src/lib.rs` に `pub mod dimension;` を追加し `Dim`・`Quantity`・全 8 種型エイリアスを再エクスポートする
  - 後続タスクが各サブモジュールファイルを独立して作成できるようスタブ状態で完成させる
  - _Requirements: 1.5, 2.3, 6.3_

- [x] 1.3 `Dim` 構造体の実装
  - `src/dimension/dim.rs` に `Dim<V, M: Integer, L: Integer, T: Integer>` を定義し、`value: V` フィールドと `_phantom: PhantomData<(M, L, T)>` で構成する
  - `#[repr(transparent)]` を付与してゼロコストレイアウトを保証し、`#[derive(Debug, Clone, Copy, PartialEq)]` を付与する
  - `pub fn new(value: V) -> Self` コンストラクタと `pub fn value(&self) -> V where V: Copy` メソッドを実装する
  - `#[cfg(test)]` 内に `test_dim_new_value_roundtrip`（`Dim::new(v).value() == v`）を追加する
  - _Requirements: 1.1, 1.2, 1.3, 1.4_

- [x] 2. (P) `Quantity` トレイトの実装
  - `src/dimension/quantity.rs` に `trait Quantity { type Value; }` を定義する
  - `impl<V, M: Integer, L: Integer, T: Integer> Quantity for Dim<V, M, L, T>` で `type Value = V` を実装する
  - 上位レイヤーが `T: Quantity where T::Value: FieldValue` という bound で使用できることを確認する
  - _Requirements: 2.1, 2.2, 2.3, 2.4_

- [x] 3. 算術演算子の実装
- [x] 3.1 (P) 同次元演算子の実装
  - `src/dimension/ops.rs` を新規作成し、同次元 `Add`・`Sub`・`Neg`・`Mul<f64>`・`Div<f64>` を実装する
  - 型パラメータ `M`・`L`・`T` が完全一致する場合のみコンパイル成功となる設計を維持し、異次元加減算は `Add` impl が存在しないためコンパイルエラーになることを確認する
  - `typenum::Add`（型レベル）と `std::ops::Add`（値レベル）を混同しないよう `use` で明示的に分離する
  - `#[cfg(test)]` 内に `test_pressure_add_same_dimension_succeeds`・`test_dim_neg_returns_same_dimension` を追加する
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 4.3, 4.4_

- [x] 3.2 異次元乗除算演算子の実装
  - 同じ `ops.rs` に `Mul<Dim<V2, M2, L2, T2>>` を実装し、出力型を `Dim<..., Sum<M1,M2>, Sum<L1,L2>, Sum<T1,T2>>` とする
  - `Div<Dim<V2, M2, L2, T2>>` を実装し `Diff<M1,M2>` 等で次元指数を減算する
  - `Mul<f64>` と `Mul<Dim<...>>` は `Rhs` 型が異なるため impl 競合なしであることを確認する
  - `#[cfg(test)]` 内に `test_density_mul_volume_gives_mass` を追加し、`Dim<f64,P1,N3,Z0> * Dim<f64,Z0,P3,Z0>` の型が `Dim<f64,P1,Z0,Z0>` になることを検証する
  - _Requirements: 4.1, 4.2_

- [x] 4. (P) CFD 型エイリアスの定義
  - `src/dimension/aliases.rs` に `Pressure`・`Velocity`・`Density`・`DynamicViscosity`・`KinematicViscosity`・`Length`・`Time`・`Mass` の 8 種を型エイリアスとして定義する
  - `use dugong_types::{Pressure, Velocity, Density};` で直接インポートできることを確認する
  - _Requirements: 6.1, 6.2, 6.3_

- [x] 5. (P) `FieldValue`・`HasGrad`・`HasDiv` 統合実装
  - `src/dimension/field_value_impl.rs` に `V: FieldValue` を bound とするブランケット impl を定義し、`zero()` は `Dim::new(V::zero())`、`mag()` は内部値の `mag()` に委譲する
  - `V: FieldValue + HasGrad` のとき `HasGrad for Dim<V, M, L, T>` を実装し `type GradOutput = Dim<V::GradOutput, M, L, T>` とする
  - `V: FieldValue + HasDiv` のとき `HasDiv for Dim<V, M, L, T>` を実装し `type DivOutput = Dim<V::DivOutput, M, L, T>` とする
  - `#[cfg(test)]` 内に `test_dim_fieldvalue_zero_mag_is_zero` と `test_velocity_hasgrad_gradoutput_type` を追加する
  - `VolumeField<Dim<V, M, L, T>, State>` が型として成立することを型レベルで確認する
  - _Requirements: 7.1, 7.2, 7.3, 7.4_

- [x] 6. compile_fail テストの実装
  - `tests/compile_fail_dimension.rs` に trybuild テストランナーを実装し、`tests/compile_fail/` 配下のファイルを `t.compile_fail(...)` で参照する
  - `tests/compile_fail/add_different_dims.rs` に `Dim<f64,P1,N1,N2>` と `Dim<f64,P1,N3,Z0>` を加算するコードを記述し、コンパイルエラーになることを検証する
  - `tests/compile_fail/sub_different_dims.rs` に異次元減算コードを記述する
  - タスク 3 の ops.rs（`Add` impl が同次元のみ）完了後に実施する
  - _Requirements: 5.1, 5.2, 5.3_

- [x] 7. ドキュメント・品質検証
- [x] 7.1 公開 API へのドキュメントコメント付与
  - `Dim` 構造体・`Quantity` トレイト・全型エイリアスに `///` コメントを付与する
  - `Dim` もしくは代表的な公開 API に `# Examples` セクションを設け、`Pressure`・`Velocity` の使用例を含める
  - _Requirements: 8.1, 8.4_

- [x] 7.2 品質チェックと全テスト実行
  - `cargo clippy -p dugong-types -- -D warnings` を実行し警告ゼロを確認する
  - `cargo build -p dugong-types` が正常終了することを確認する
  - `cargo test -p dugong-types` を実行し compile_fail テストを含む全テストが通過することを確認する
  - _Requirements: 8.2, 8.3_
