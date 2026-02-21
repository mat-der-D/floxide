use typenum::Integer;

use crate::traits::{FieldValue, HasDiv, HasGrad};

use super::dim::Dim;

/// `V: FieldValue` のとき `Dim<V, M, L, T>` も `FieldValue` を実装する。
///
/// `FieldValue` のスーパートレイト（`Copy`, `Add`, `Sub`, `Mul<f64>`, `Neg`）は
/// `V: FieldValue` と `ops.rs` の演算子実装によって自動的に充足される。
impl<V: FieldValue, M: Integer, L: Integer, T: Integer> FieldValue for Dim<V, M, L, T> {
    fn zero() -> Self {
        Dim::new(V::zero())
    }

    fn mag(&self) -> f64 {
        self.value_ref().mag()
    }
}

/// `V: FieldValue + HasGrad` のとき `HasGrad` を実装する。
///
/// 次元指数は変わらず、値のテンソルランクが昇格する。
///
/// 例: `Dim<f64, M, L, T>` の grad → `Dim<Vector, M, L, T>`
impl<V: FieldValue + HasGrad, M: Integer, L: Integer, T: Integer> HasGrad for Dim<V, M, L, T>
where
    V::GradOutput: FieldValue,
{
    type GradOutput = Dim<V::GradOutput, M, L, T>;
}

/// `V: FieldValue + HasDiv` のとき `HasDiv` を実装する。
///
/// 例: `Dim<Vector, M, L, T>` の div → `Dim<f64, M, L, T>`
impl<V: FieldValue + HasDiv, M: Integer, L: Integer, T: Integer> HasDiv for Dim<V, M, L, T>
where
    V::DivOutput: FieldValue,
{
    type DivOutput = Dim<V::DivOutput, M, L, T>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tensor::{Tensor, Vector};
    use typenum::{N1, P1, Z0};

    // Pressure: Dim<f64, P1, N1, N2>
    // Velocity: Dim<Vector, Z0, P1, N1>

    #[test]
    fn test_dim_fieldvalue_zero_mag_is_zero() {
        type Pressure = Dim<f64, P1, N1, N1>;
        assert!(Pressure::zero().mag() < 1e-14);
    }

    #[test]
    fn test_velocity_zero_mag_is_zero() {
        type Velocity = Dim<Vector, Z0, P1, N1>;
        assert!(Velocity::zero().mag() < 1e-14);
    }

    #[test]
    fn test_velocity_hasgrad_gradoutput_type() {
        // <Velocity as HasGrad>::GradOutput = Dim<Tensor, Z0, P1, N1>
        // 型アノテーションで検証
        type Velocity = Dim<Vector, Z0, P1, N1>;
        type VelocityGrad = <Velocity as HasGrad>::GradOutput;
        let _: VelocityGrad = Dim::new(Tensor::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0));
    }

    #[test]
    fn test_velocity_hasdiv_divoutput_type() {
        // <Dim<Vector, M, L, T> as HasDiv>::DivOutput = Dim<f64, M, L, T>
        type Velocity = Dim<Vector, Z0, P1, N1>;
        type VelocityDiv = <Velocity as HasDiv>::DivOutput;
        let _: VelocityDiv = Dim::new(0.0_f64);
    }
}
