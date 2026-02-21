use std::marker::PhantomData;

use typenum::Integer;

/// 物理次元付き量。M: 質量, L: 長さ, T: 時間の SI 次元指数を型パラメータで保持する。
///
/// 次元指数は `typenum` の型レベル整数（`P1`, `N1`, `Z0` 等）で表現される。
/// 直接使用するよりも型エイリアス（[`Pressure`][crate::Pressure]、
/// [`Velocity`][crate::Velocity] 等）を推奨する。
///
/// # Examples
///
/// ```
/// use dugong_types::{Pressure, Velocity};
/// use dugong_types::tensor::Vector;
///
/// let p = Pressure::new(101325.0);
/// let v = Velocity::new(Vector::new(1.0, 0.0, 0.0));
/// assert_eq!(p.value(), 101325.0);
/// ```
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Dim<V, M: Integer, L: Integer, T: Integer> {
    value: V,
    _phantom: PhantomData<(M, L, T)>,
}

impl<V, M: Integer, L: Integer, T: Integer> Dim<V, M, L, T> {
    /// 値を包んで次元付き量を生成する。
    pub fn new(value: V) -> Self {
        Self {
            value,
            _phantom: PhantomData,
        }
    }

    /// 内部の生の値を返す。
    pub fn value(&self) -> V
    where
        V: Copy,
    {
        self.value
    }

    /// 内部値への参照を返す（同モジュール内の演算実装に使用）。
    pub(super) fn value_ref(&self) -> &V {
        &self.value
    }

    /// 値を消費して内部値を返す（同モジュール内の演算実装に使用）。
    pub(super) fn into_value(self) -> V {
        self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use typenum::{N1, N2, P1};

    #[test]
    fn test_dim_new_value_roundtrip() {
        let v = 42.0_f64;
        let d: Dim<f64, P1, N1, N2> = Dim::new(v);
        assert_eq!(d.value(), v);
    }

    #[test]
    fn test_dim_new_integer_value() {
        let d: Dim<i32, P1, N1, N2> = Dim::new(100);
        assert_eq!(d.value(), 100);
    }
}
