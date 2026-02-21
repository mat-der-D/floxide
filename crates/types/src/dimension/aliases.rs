use typenum::{N1, N2, N3, P1, P2, Z0};

use crate::tensor::Vector;

use super::dim::Dim;

/// 圧力 (Pa = kg·m⁻¹·s⁻²)
///
/// SI 次元: M=1, L=-1, T=-2
pub type Pressure = Dim<f64, P1, N1, N2>;

/// 速度 (m·s⁻¹)。Vector 値を保持する。
///
/// SI 次元: M=0, L=1, T=-1
pub type Velocity = Dim<Vector, Z0, P1, N1>;

/// 密度 (kg·m⁻³)
///
/// SI 次元: M=1, L=-3, T=0
pub type Density = Dim<f64, P1, N3, Z0>;

/// 動粘性係数 (Pa·s = kg·m⁻¹·s⁻¹)
///
/// SI 次元: M=1, L=-1, T=-1
pub type DynamicViscosity = Dim<f64, P1, N1, N1>;

/// 動粘性率 (m²·s⁻¹)
///
/// SI 次元: M=0, L=2, T=-1
pub type KinematicViscosity = Dim<f64, Z0, P2, N1>;

/// 長さ (m)
///
/// SI 次元: M=0, L=1, T=0
pub type Length = Dim<f64, Z0, P1, Z0>;

/// 時間 (s)
///
/// SI 次元: M=0, L=0, T=1
pub type Time = Dim<f64, Z0, Z0, P1>;

/// 質量 (kg)
///
/// SI 次元: M=1, L=0, T=0
pub type Mass = Dim<f64, P1, Z0, Z0>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tensor::Vector;

    #[test]
    fn test_pressure_alias() {
        let p = Pressure::new(101325.0);
        assert_eq!(p.value(), 101325.0);
    }

    #[test]
    fn test_velocity_alias() {
        let v = Velocity::new(Vector::new(1.0, 0.0, 0.0));
        assert_eq!(v.value(), Vector::new(1.0, 0.0, 0.0));
    }

    #[test]
    fn test_density_alias() {
        let rho = Density::new(1000.0);
        assert_eq!(rho.value(), 1000.0);
    }
}
