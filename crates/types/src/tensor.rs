/// CFD フレームワークの基盤となるテンソル型群とその演算を提供する。
///
/// 5 種のテンソル型（`Scalar`, `Vector`, `Tensor`, `SymmTensor`, `SphericalTensor`）
/// の定義と、同型・異型間の算術演算、型変換メソッド、特殊値コンストラクタを含む。
mod convert;
mod cross_ops;
mod ops;
mod special;
#[cfg(test)]
mod tests;
mod types;

pub use types::{Scalar, SphericalTensor, SymmTensor, Tensor, Vector};
