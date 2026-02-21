/// 零テンソル・単位テンソルなどの特殊値コンストラクタを提供する。
use super::types::{SphericalTensor, SymmTensor, Tensor, Vector};

impl Vector {
    /// 全成分ゼロのベクトルを返す。
    #[inline]
    pub const fn zero() -> Self {
        Self([0.0, 0.0, 0.0])
    }
}

impl Tensor {
    /// 全成分ゼロのテンソルを返す。
    #[inline]
    pub const fn zero() -> Self {
        Self([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])
    }

    /// 3×3 単位行列を返す。
    #[inline]
    pub const fn identity() -> Self {
        Self([1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0])
    }
}

impl SymmTensor {
    /// 全成分ゼロの対称テンソルを返す。
    #[inline]
    pub const fn zero() -> Self {
        Self([0.0, 0.0, 0.0, 0.0, 0.0, 0.0])
    }

    /// 対角成分 1 の対称テンソルを返す。
    #[inline]
    pub const fn identity() -> Self {
        Self([1.0, 0.0, 0.0, 1.0, 0.0, 1.0])
    }
}

impl SphericalTensor {
    /// 値ゼロの球面テンソルを返す。
    #[inline]
    pub const fn zero() -> Self {
        Self(0.0)
    }

    /// 値 1 の球面テンソル（単位球面テンソル）を返す。
    #[inline]
    pub const fn identity() -> Self {
        Self(1.0)
    }
}
