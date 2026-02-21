mod aliases;
/// 物理次元付き量のモジュール。
///
/// `typenum` クレートの型レベル整数算術を用いて、物理次元の整合性を
/// コンパイル時に保証する。
///
/// # モジュール構成
///
/// - [`Dim`][]: 物理次元付き量のコア型
/// - [`Quantity`][]: 次元付き量の統一インターフェース
/// - 型エイリアス: [`Pressure`]、[`Velocity`]、[`Density`] 等 8 種
mod dim;
mod field_value_impl;
mod ops;
mod quantity;

pub use aliases::{
    Density, DynamicViscosity, KinematicViscosity, Length, Mass, Pressure, Time, Velocity,
};
pub use dim::Dim;
pub use quantity::Quantity;
