/// 次元付き量の算術演算子実装。
///
/// - 同次元演算（`Add`, `Sub`, `Neg`, `Mul<f64>`, `Div<f64>`）: 型パラメータ M, L, T が完全一致する場合のみコンパイル成功
/// - 異次元乗算（`Mul<Dim<...>>`）: typenum の型レベル加算（`std::ops::Add`）で次元指数を合成
/// - 異次元除算（`Div<Dim<...>>`）: typenum の型レベル減算（`std::ops::Sub`）で次元指数を合成
///
/// **注意**: `typenum` は型レベル整数に対して `std::ops::Add`/`Sub` を実装しており、
/// 値レベルの算術と同じトレイト名を使用する。混乱を避けるため、異次元演算の where 節では
/// 型レベル演算の bound (`M1: Add<M2>`) と値レベルの bound (`V: Add<Output=V>`) を
/// 文脈から区別できるよう型引数で明示する。
use std::ops::{Add, Div, Mul, Neg, Sub};

use typenum::{Diff, Integer, Sum};

use super::dim::Dim;

// ── 同次元演算（M, L, T の型が完全一致する場合のみコンパイル成功） ──

impl<V: Add<Output = V>, M: Integer, L: Integer, T: Integer> Add for Dim<V, M, L, T> {
    type Output = Dim<V, M, L, T>;

    fn add(self, rhs: Self) -> Self::Output {
        Dim::new(self.into_value() + rhs.into_value())
    }
}

impl<V: Sub<Output = V>, M: Integer, L: Integer, T: Integer> Sub for Dim<V, M, L, T> {
    type Output = Dim<V, M, L, T>;

    fn sub(self, rhs: Self) -> Self::Output {
        Dim::new(self.into_value() - rhs.into_value())
    }
}

impl<V: Neg<Output = V>, M: Integer, L: Integer, T: Integer> Neg for Dim<V, M, L, T> {
    type Output = Dim<V, M, L, T>;

    fn neg(self) -> Self::Output {
        Dim::new(-self.into_value())
    }
}

impl<V: Mul<f64, Output = V>, M: Integer, L: Integer, T: Integer> Mul<f64> for Dim<V, M, L, T> {
    type Output = Dim<V, M, L, T>;

    fn mul(self, rhs: f64) -> Self::Output {
        Dim::new(self.into_value() * rhs)
    }
}

impl<V: Div<f64, Output = V>, M: Integer, L: Integer, T: Integer> Div<f64> for Dim<V, M, L, T> {
    type Output = Dim<V, M, L, T>;

    fn div(self, rhs: f64) -> Self::Output {
        Dim::new(self.into_value() / rhs)
    }
}

// ── 異次元乗算（typenum が std::ops::Add を型レベル整数に実装 → 次元指数を型レベルで加算） ──

impl<V1, V2, M1, M2, L1, L2, T1, T2> Mul<Dim<V2, M2, L2, T2>> for Dim<V1, M1, L1, T1>
where
    V1: Mul<V2>,
    M1: Integer + Add<M2>,
    M2: Integer,
    L1: Integer + Add<L2>,
    L2: Integer,
    T1: Integer + Add<T2>,
    T2: Integer,
    Sum<M1, M2>: Integer,
    Sum<L1, L2>: Integer,
    Sum<T1, T2>: Integer,
{
    type Output = Dim<<V1 as Mul<V2>>::Output, Sum<M1, M2>, Sum<L1, L2>, Sum<T1, T2>>;

    fn mul(self, rhs: Dim<V2, M2, L2, T2>) -> Self::Output {
        Dim::new(self.into_value() * rhs.into_value())
    }
}

// ── 異次元除算（typenum が std::ops::Sub を型レベル整数に実装 → 次元指数を型レベルで減算） ──

impl<V1, V2, M1, M2, L1, L2, T1, T2> Div<Dim<V2, M2, L2, T2>> for Dim<V1, M1, L1, T1>
where
    V1: Div<V2>,
    M1: Integer + Sub<M2>,
    M2: Integer,
    L1: Integer + Sub<L2>,
    L2: Integer,
    T1: Integer + Sub<T2>,
    T2: Integer,
    Diff<M1, M2>: Integer,
    Diff<L1, L2>: Integer,
    Diff<T1, T2>: Integer,
{
    type Output = Dim<<V1 as Div<V2>>::Output, Diff<M1, M2>, Diff<L1, L2>, Diff<T1, T2>>;

    fn div(self, rhs: Dim<V2, M2, L2, T2>) -> Self::Output {
        Dim::new(self.into_value() / rhs.into_value())
    }
}

#[cfg(test)]
mod tests {
    use typenum::{N1, N2, N3, P1, P3, Z0};

    use super::*;

    // Pressure 相当: Dim<f64, P1, N1, N2>
    // Density 相当: Dim<f64, P1, N3, Z0>
    // Volume 相当: Dim<f64, Z0, P3, Z0>
    // Mass 相当: Dim<f64, P1, Z0, Z0>

    #[test]
    fn test_pressure_add_same_dimension_succeeds() {
        let p1: Dim<f64, P1, N1, N2> = Dim::new(100.0);
        let p2: Dim<f64, P1, N1, N2> = Dim::new(200.0);
        let p3 = p1 + p2;
        assert_eq!(p3.value(), 300.0);
    }

    #[test]
    fn test_dim_sub_same_dimension_succeeds() {
        let p1: Dim<f64, P1, N1, N2> = Dim::new(300.0);
        let p2: Dim<f64, P1, N1, N2> = Dim::new(100.0);
        let p3 = p1 - p2;
        assert_eq!(p3.value(), 200.0);
    }

    #[test]
    fn test_dim_neg_returns_same_dimension() {
        let p: Dim<f64, P1, N1, N2> = Dim::new(101325.0);
        let neg_p = -p;
        assert_eq!(neg_p.value(), -101325.0);
    }

    #[test]
    fn test_dim_mul_f64_preserves_dimension() {
        let p: Dim<f64, P1, N1, N2> = Dim::new(100.0);
        let scaled = p * 2.0;
        assert_eq!(scaled.value(), 200.0);
    }

    #[test]
    fn test_dim_div_f64_preserves_dimension() {
        let p: Dim<f64, P1, N1, N2> = Dim::new(100.0);
        let scaled = p / 2.0;
        assert_eq!(scaled.value(), 50.0);
    }

    #[test]
    fn test_density_mul_volume_gives_mass() {
        // Dim<f64, P1, N3, Z0> * Dim<f64, Z0, P3, Z0>
        // → Dim<f64, Sum<P1,Z0>=P1, Sum<N3,P3>=Z0, Sum<Z0,Z0>=Z0>
        // = Dim<f64, P1, Z0, Z0> = Mass
        let density: Dim<f64, P1, N3, Z0> = Dim::new(1000.0);
        let volume: Dim<f64, Z0, P3, Z0> = Dim::new(2.0);
        let mass: Dim<f64, P1, Z0, Z0> = density * volume;
        assert_eq!(mass.value(), 2000.0);
    }
}
