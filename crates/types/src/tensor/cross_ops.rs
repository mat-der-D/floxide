/// 異なるテンソルランク間の演算を提供する。
use std::ops::{Add, Mul, Sub};

use super::types::{SphericalTensor, SymmTensor, Tensor, Vector};

// ===== 異型間加算・減算 =====

// SymmTensor + SphericalTensor → SymmTensor
impl Add<SphericalTensor> for SymmTensor {
    type Output = SymmTensor;

    #[inline]
    fn add(self, rhs: SphericalTensor) -> SymmTensor {
        let s = rhs.value();
        SymmTensor::new(
            self.xx() + s,
            self.xy(),
            self.xz(),
            self.yy() + s,
            self.yz(),
            self.zz() + s,
        )
    }
}

// SphericalTensor + SymmTensor → SymmTensor
impl Add<SymmTensor> for SphericalTensor {
    type Output = SymmTensor;

    #[inline]
    fn add(self, rhs: SymmTensor) -> SymmTensor {
        rhs + self
    }
}

// SymmTensor - SphericalTensor → SymmTensor
impl Sub<SphericalTensor> for SymmTensor {
    type Output = SymmTensor;

    #[inline]
    fn sub(self, rhs: SphericalTensor) -> SymmTensor {
        let s = rhs.value();
        SymmTensor::new(
            self.xx() - s,
            self.xy(),
            self.xz(),
            self.yy() - s,
            self.yz(),
            self.zz() - s,
        )
    }
}

// SphericalTensor - SymmTensor → SymmTensor
impl Sub<SymmTensor> for SphericalTensor {
    type Output = SymmTensor;

    #[inline]
    fn sub(self, rhs: SymmTensor) -> SymmTensor {
        let s = self.value();
        SymmTensor::new(
            s - rhs.xx(),
            -rhs.xy(),
            -rhs.xz(),
            s - rhs.yy(),
            -rhs.yz(),
            s - rhs.zz(),
        )
    }
}

// Tensor + SymmTensor → Tensor
impl Add<SymmTensor> for Tensor {
    type Output = Tensor;

    #[inline]
    fn add(self, rhs: SymmTensor) -> Tensor {
        Tensor::new(
            self.xx() + rhs.xx(),
            self.xy() + rhs.xy(),
            self.xz() + rhs.xz(),
            self.yx() + rhs.xy(),
            self.yy() + rhs.yy(),
            self.yz() + rhs.yz(),
            self.zx() + rhs.xz(),
            self.zy() + rhs.yz(),
            self.zz() + rhs.zz(),
        )
    }
}

// Tensor - SymmTensor → Tensor
impl Sub<SymmTensor> for Tensor {
    type Output = Tensor;

    #[inline]
    fn sub(self, rhs: SymmTensor) -> Tensor {
        Tensor::new(
            self.xx() - rhs.xx(),
            self.xy() - rhs.xy(),
            self.xz() - rhs.xz(),
            self.yx() - rhs.xy(),
            self.yy() - rhs.yy(),
            self.yz() - rhs.yz(),
            self.zx() - rhs.xz(),
            self.zy() - rhs.yz(),
            self.zz() - rhs.zz(),
        )
    }
}

// Tensor + SphericalTensor → Tensor
impl Add<SphericalTensor> for Tensor {
    type Output = Tensor;

    #[inline]
    fn add(self, rhs: SphericalTensor) -> Tensor {
        let s = rhs.value();
        Tensor::new(
            self.xx() + s,
            self.xy(),
            self.xz(),
            self.yx(),
            self.yy() + s,
            self.yz(),
            self.zx(),
            self.zy(),
            self.zz() + s,
        )
    }
}

// Tensor - SphericalTensor → Tensor
impl Sub<SphericalTensor> for Tensor {
    type Output = Tensor;

    #[inline]
    fn sub(self, rhs: SphericalTensor) -> Tensor {
        let s = rhs.value();
        Tensor::new(
            self.xx() - s,
            self.xy(),
            self.xz(),
            self.yx(),
            self.yy() - s,
            self.yz(),
            self.zx(),
            self.zy(),
            self.zz() - s,
        )
    }
}

// ===== 単縮約（Mul trait: rank(A) + rank(B) - 2）=====

// Vector * Vector → f64（内積）
impl Mul<Vector> for Vector {
    type Output = f64;

    #[inline]
    fn mul(self, rhs: Vector) -> f64 {
        self.x() * rhs.x() + self.y() * rhs.y() + self.z() * rhs.z()
    }
}

// Tensor * Vector → Vector（行列・ベクトル積）
impl Mul<Vector> for Tensor {
    type Output = Vector;

    #[inline]
    fn mul(self, v: Vector) -> Vector {
        Vector::new(
            self.xx() * v.x() + self.xy() * v.y() + self.xz() * v.z(),
            self.yx() * v.x() + self.yy() * v.y() + self.yz() * v.z(),
            self.zx() * v.x() + self.zy() * v.y() + self.zz() * v.z(),
        )
    }
}

// Vector * Tensor → Vector（ベクトル・行列積: v^T * T）
impl Mul<Tensor> for Vector {
    type Output = Vector;

    #[inline]
    fn mul(self, t: Tensor) -> Vector {
        Vector::new(
            self.x() * t.xx() + self.y() * t.yx() + self.z() * t.zx(),
            self.x() * t.xy() + self.y() * t.yy() + self.z() * t.zy(),
            self.x() * t.xz() + self.y() * t.yz() + self.z() * t.zz(),
        )
    }
}

// Tensor * Tensor → Tensor（行列積）
impl Mul<Tensor> for Tensor {
    type Output = Tensor;

    #[inline]
    fn mul(self, b: Tensor) -> Tensor {
        Tensor::new(
            self.xx() * b.xx() + self.xy() * b.yx() + self.xz() * b.zx(),
            self.xx() * b.xy() + self.xy() * b.yy() + self.xz() * b.zy(),
            self.xx() * b.xz() + self.xy() * b.yz() + self.xz() * b.zz(),
            self.yx() * b.xx() + self.yy() * b.yx() + self.yz() * b.zx(),
            self.yx() * b.xy() + self.yy() * b.yy() + self.yz() * b.zy(),
            self.yx() * b.xz() + self.yy() * b.yz() + self.yz() * b.zz(),
            self.zx() * b.xx() + self.zy() * b.yx() + self.zz() * b.zx(),
            self.zx() * b.xy() + self.zy() * b.yy() + self.zz() * b.zy(),
            self.zx() * b.xz() + self.zy() * b.yz() + self.zz() * b.zz(),
        )
    }
}

// SymmTensor * Vector → Vector
impl Mul<Vector> for SymmTensor {
    type Output = Vector;

    #[inline]
    fn mul(self, v: Vector) -> Vector {
        Vector::new(
            self.xx() * v.x() + self.xy() * v.y() + self.xz() * v.z(),
            self.xy() * v.x() + self.yy() * v.y() + self.yz() * v.z(),
            self.xz() * v.x() + self.yz() * v.y() + self.zz() * v.z(),
        )
    }
}

// SymmTensor * SymmTensor → Tensor（行列積、結果は一般に非対称）
impl Mul<SymmTensor> for SymmTensor {
    type Output = Tensor;

    #[inline]
    fn mul(self, b: SymmTensor) -> Tensor {
        Tensor::new(
            self.xx() * b.xx() + self.xy() * b.xy() + self.xz() * b.xz(),
            self.xx() * b.xy() + self.xy() * b.yy() + self.xz() * b.yz(),
            self.xx() * b.xz() + self.xy() * b.yz() + self.xz() * b.zz(),
            self.xy() * b.xx() + self.yy() * b.xy() + self.yz() * b.xz(),
            self.xy() * b.xy() + self.yy() * b.yy() + self.yz() * b.yz(),
            self.xy() * b.xz() + self.yy() * b.yz() + self.yz() * b.zz(),
            self.xz() * b.xx() + self.yz() * b.xy() + self.zz() * b.xz(),
            self.xz() * b.xy() + self.yz() * b.yy() + self.zz() * b.yz(),
            self.xz() * b.xz() + self.yz() * b.yz() + self.zz() * b.zz(),
        )
    }
}

// ===== 二重縮約・テンソル積・クロス積 =====

impl Tensor {
    /// テンソル二重縮約: `A:B = Σ_ij A_ij * B_ij`
    #[inline]
    pub fn double_dot(&self, other: &Tensor) -> f64 {
        let a = self.as_array();
        let b = other.as_array();
        a[0] * b[0]
            + a[1] * b[1]
            + a[2] * b[2]
            + a[3] * b[3]
            + a[4] * b[4]
            + a[5] * b[5]
            + a[6] * b[6]
            + a[7] * b[7]
            + a[8] * b[8]
    }
}

impl SymmTensor {
    /// 対称テンソル二重縮約: `A:B = Σ_ij A_ij * B_ij`
    ///
    /// 対称性により非対角成分を 2 倍する。
    #[inline]
    pub fn double_dot(&self, other: &SymmTensor) -> f64 {
        self.xx() * other.xx()
            + self.yy() * other.yy()
            + self.zz() * other.zz()
            + 2.0 * (self.xy() * other.xy() + self.xz() * other.xz() + self.yz() * other.yz())
    }
}

impl Vector {
    /// テンソル積: `a ⊗ b → Tensor`（`T_ij = a_i * b_j`）
    #[inline]
    pub fn outer(&self, other: &Vector) -> Tensor {
        Tensor::new(
            self.x() * other.x(),
            self.x() * other.y(),
            self.x() * other.z(),
            self.y() * other.x(),
            self.y() * other.y(),
            self.y() * other.z(),
            self.z() * other.x(),
            self.z() * other.y(),
            self.z() * other.z(),
        )
    }

    /// クロス積: `a × b → Vector`
    #[inline]
    pub fn cross(&self, other: &Vector) -> Vector {
        Vector::new(
            self.y() * other.z() - self.z() * other.y(),
            self.z() * other.x() - self.x() * other.z(),
            self.x() * other.y() - self.y() * other.x(),
        )
    }
}
