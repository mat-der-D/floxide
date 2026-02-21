/// 同型テンソル間の基本算術演算を提供する。
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use super::types::{SphericalTensor, SymmTensor, Tensor, Vector};

// ===== Vector =====

impl Add for Vector {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self {
        let a = self.as_array();
        let b = rhs.as_array();
        Vector::new(a[0] + b[0], a[1] + b[1], a[2] + b[2])
    }
}

impl Sub for Vector {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self {
        let a = self.as_array();
        let b = rhs.as_array();
        Vector::new(a[0] - b[0], a[1] - b[1], a[2] - b[2])
    }
}

impl Neg for Vector {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self {
        let a = self.as_array();
        Vector::new(-a[0], -a[1], -a[2])
    }
}

impl Mul<f64> for Vector {
    type Output = Self;

    #[inline]
    fn mul(self, s: f64) -> Self {
        let a = self.as_array();
        Vector::new(a[0] * s, a[1] * s, a[2] * s)
    }
}

impl Mul<Vector> for f64 {
    type Output = Vector;

    #[inline]
    fn mul(self, v: Vector) -> Vector {
        v * self
    }
}

impl Div<f64> for Vector {
    type Output = Self;

    #[inline]
    fn div(self, s: f64) -> Self {
        let a = self.as_array();
        Vector::new(a[0] / s, a[1] / s, a[2] / s)
    }
}

impl AddAssign for Vector {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl SubAssign for Vector {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl MulAssign<f64> for Vector {
    #[inline]
    fn mul_assign(&mut self, s: f64) {
        *self = *self * s;
    }
}

impl DivAssign<f64> for Vector {
    #[inline]
    fn div_assign(&mut self, s: f64) {
        *self = *self / s;
    }
}

// ===== Tensor =====

impl Add for Tensor {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self {
        let a = self.as_array();
        let b = rhs.as_array();
        Tensor::new(
            a[0] + b[0],
            a[1] + b[1],
            a[2] + b[2],
            a[3] + b[3],
            a[4] + b[4],
            a[5] + b[5],
            a[6] + b[6],
            a[7] + b[7],
            a[8] + b[8],
        )
    }
}

impl Sub for Tensor {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self {
        let a = self.as_array();
        let b = rhs.as_array();
        Tensor::new(
            a[0] - b[0],
            a[1] - b[1],
            a[2] - b[2],
            a[3] - b[3],
            a[4] - b[4],
            a[5] - b[5],
            a[6] - b[6],
            a[7] - b[7],
            a[8] - b[8],
        )
    }
}

impl Neg for Tensor {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self {
        let a = self.as_array();
        Tensor::new(
            -a[0], -a[1], -a[2], -a[3], -a[4], -a[5], -a[6], -a[7], -a[8],
        )
    }
}

impl Mul<f64> for Tensor {
    type Output = Self;

    #[inline]
    fn mul(self, s: f64) -> Self {
        let a = self.as_array();
        Tensor::new(
            a[0] * s,
            a[1] * s,
            a[2] * s,
            a[3] * s,
            a[4] * s,
            a[5] * s,
            a[6] * s,
            a[7] * s,
            a[8] * s,
        )
    }
}

impl Mul<Tensor> for f64 {
    type Output = Tensor;

    #[inline]
    fn mul(self, t: Tensor) -> Tensor {
        t * self
    }
}

impl Div<f64> for Tensor {
    type Output = Self;

    #[inline]
    fn div(self, s: f64) -> Self {
        let a = self.as_array();
        Tensor::new(
            a[0] / s,
            a[1] / s,
            a[2] / s,
            a[3] / s,
            a[4] / s,
            a[5] / s,
            a[6] / s,
            a[7] / s,
            a[8] / s,
        )
    }
}

impl AddAssign for Tensor {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl SubAssign for Tensor {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl MulAssign<f64> for Tensor {
    #[inline]
    fn mul_assign(&mut self, s: f64) {
        *self = *self * s;
    }
}

impl DivAssign<f64> for Tensor {
    #[inline]
    fn div_assign(&mut self, s: f64) {
        *self = *self / s;
    }
}

// ===== SymmTensor =====

impl Add for SymmTensor {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self {
        let a = self.as_array();
        let b = rhs.as_array();
        SymmTensor::new(
            a[0] + b[0],
            a[1] + b[1],
            a[2] + b[2],
            a[3] + b[3],
            a[4] + b[4],
            a[5] + b[5],
        )
    }
}

impl Sub for SymmTensor {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self {
        let a = self.as_array();
        let b = rhs.as_array();
        SymmTensor::new(
            a[0] - b[0],
            a[1] - b[1],
            a[2] - b[2],
            a[3] - b[3],
            a[4] - b[4],
            a[5] - b[5],
        )
    }
}

impl Neg for SymmTensor {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self {
        let a = self.as_array();
        SymmTensor::new(-a[0], -a[1], -a[2], -a[3], -a[4], -a[5])
    }
}

impl Mul<f64> for SymmTensor {
    type Output = Self;

    #[inline]
    fn mul(self, s: f64) -> Self {
        let a = self.as_array();
        SymmTensor::new(a[0] * s, a[1] * s, a[2] * s, a[3] * s, a[4] * s, a[5] * s)
    }
}

impl Mul<SymmTensor> for f64 {
    type Output = SymmTensor;

    #[inline]
    fn mul(self, t: SymmTensor) -> SymmTensor {
        t * self
    }
}

impl Div<f64> for SymmTensor {
    type Output = Self;

    #[inline]
    fn div(self, s: f64) -> Self {
        let a = self.as_array();
        SymmTensor::new(a[0] / s, a[1] / s, a[2] / s, a[3] / s, a[4] / s, a[5] / s)
    }
}

impl AddAssign for SymmTensor {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl SubAssign for SymmTensor {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl MulAssign<f64> for SymmTensor {
    #[inline]
    fn mul_assign(&mut self, s: f64) {
        *self = *self * s;
    }
}

impl DivAssign<f64> for SymmTensor {
    #[inline]
    fn div_assign(&mut self, s: f64) {
        *self = *self / s;
    }
}

// ===== SphericalTensor =====

impl Add for SphericalTensor {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self {
        SphericalTensor::new(self.value() + rhs.value())
    }
}

impl Sub for SphericalTensor {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self {
        SphericalTensor::new(self.value() - rhs.value())
    }
}

impl Neg for SphericalTensor {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self {
        SphericalTensor::new(-self.value())
    }
}

impl Mul<f64> for SphericalTensor {
    type Output = Self;

    #[inline]
    fn mul(self, s: f64) -> Self {
        SphericalTensor::new(self.value() * s)
    }
}

impl Mul<SphericalTensor> for f64 {
    type Output = SphericalTensor;

    #[inline]
    fn mul(self, t: SphericalTensor) -> SphericalTensor {
        t * self
    }
}

impl Div<f64> for SphericalTensor {
    type Output = Self;

    #[inline]
    fn div(self, s: f64) -> Self {
        SphericalTensor::new(self.value() / s)
    }
}

impl AddAssign for SphericalTensor {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl SubAssign for SphericalTensor {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl MulAssign<f64> for SphericalTensor {
    #[inline]
    fn mul_assign(&mut self, s: f64) {
        *self = *self * s;
    }
}

impl DivAssign<f64> for SphericalTensor {
    #[inline]
    fn div_assign(&mut self, s: f64) {
        *self = *self / s;
    }
}
