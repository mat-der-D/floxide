/// テンソルの代数的分解・変換メソッドと `From` trait 実装を提供する。
use super::types::{SphericalTensor, SymmTensor, Tensor, Vector};

// ===== Tensor メソッド =====

impl Tensor {
    /// 対称部分: `(T + T^T) / 2`
    #[inline]
    pub fn symm(&self) -> SymmTensor {
        SymmTensor::new(
            self.xx(),
            (self.xy() + self.yx()) / 2.0,
            (self.xz() + self.zx()) / 2.0,
            self.yy(),
            (self.yz() + self.zy()) / 2.0,
            self.zz(),
        )
    }

    /// 2 倍対称部分: `T + T^T`
    #[inline]
    pub fn two_symm(&self) -> SymmTensor {
        SymmTensor::new(
            2.0 * self.xx(),
            self.xy() + self.yx(),
            self.xz() + self.zx(),
            2.0 * self.yy(),
            self.yz() + self.zy(),
            2.0 * self.zz(),
        )
    }

    /// 球面部分: `(trace / 3) * I`
    #[inline]
    pub fn sph(&self) -> SphericalTensor {
        SphericalTensor::new(self.trace() / 3.0)
    }

    /// 反対称部分: `(T - T^T) / 2`
    #[inline]
    pub fn skew(&self) -> Tensor {
        Tensor::new(
            0.0,
            (self.xy() - self.yx()) / 2.0,
            (self.xz() - self.zx()) / 2.0,
            (self.yx() - self.xy()) / 2.0,
            0.0,
            (self.yz() - self.zy()) / 2.0,
            (self.zx() - self.xz()) / 2.0,
            (self.zy() - self.yz()) / 2.0,
            0.0,
        )
    }

    /// 偏差部分: `T - (trace/3)*I`
    #[inline]
    pub fn dev(&self) -> Tensor {
        let tr3 = self.trace() / 3.0;
        Tensor::new(
            self.xx() - tr3,
            self.xy(),
            self.xz(),
            self.yx(),
            self.yy() - tr3,
            self.yz(),
            self.zx(),
            self.zy(),
            self.zz() - tr3,
        )
    }

    /// トレース: `T_xx + T_yy + T_zz`
    #[inline]
    pub fn trace(&self) -> f64 {
        self.xx() + self.yy() + self.zz()
    }

    /// 行列式（サルスの方法による 3×3 行列式の直接展開）
    #[inline]
    pub fn det(&self) -> f64 {
        self.xx() * (self.yy() * self.zz() - self.yz() * self.zy())
            - self.xy() * (self.yx() * self.zz() - self.yz() * self.zx())
            + self.xz() * (self.yx() * self.zy() - self.yy() * self.zx())
    }

    /// 転置: `T^T`
    #[inline]
    pub fn transpose(&self) -> Tensor {
        Tensor::new(
            self.xx(),
            self.yx(),
            self.zx(),
            self.xy(),
            self.yy(),
            self.zy(),
            self.xz(),
            self.yz(),
            self.zz(),
        )
    }

    /// フロベニウスノルム: `sqrt(T:T)`
    #[inline]
    pub fn mag(&self) -> f64 {
        self.double_dot(self).sqrt()
    }
}

// ===== SymmTensor メソッド =====

impl SymmTensor {
    /// トレース: `S_xx + S_yy + S_zz`
    #[inline]
    pub fn trace(&self) -> f64 {
        self.xx() + self.yy() + self.zz()
    }

    /// 行列式（対称行列の 3×3 展開）
    #[inline]
    pub fn det(&self) -> f64 {
        self.xx() * (self.yy() * self.zz() - self.yz() * self.yz())
            - self.xy() * (self.xy() * self.zz() - self.yz() * self.xz())
            + self.xz() * (self.xy() * self.yz() - self.yy() * self.xz())
    }

    /// 偏差部分: `S - (trace/3)*I`
    #[inline]
    pub fn dev(&self) -> SymmTensor {
        let tr3 = self.trace() / 3.0;
        SymmTensor::new(
            self.xx() - tr3,
            self.xy(),
            self.xz(),
            self.yy() - tr3,
            self.yz(),
            self.zz() - tr3,
        )
    }

    /// 球面部分: `(trace / 3) * I`
    #[inline]
    pub fn sph(&self) -> SphericalTensor {
        SphericalTensor::new(self.trace() / 3.0)
    }
}

// ===== Vector メソッド =====

impl Vector {
    /// ユークリッドノルム: `sqrt(x² + y² + z²)`
    #[inline]
    pub fn mag(&self) -> f64 {
        self.mag_sqr().sqrt()
    }

    /// 二乗マグニチュード: `x² + y² + z²`
    #[inline]
    pub fn mag_sqr(&self) -> f64 {
        self.x() * self.x() + self.y() * self.y() + self.z() * self.z()
    }
}

// ===== From 変換 =====

/// `SphericalTensor` → `SymmTensor`: 対角成分に `s` を設定する。
impl From<SphericalTensor> for SymmTensor {
    #[inline]
    fn from(sph: SphericalTensor) -> Self {
        let s = sph.value();
        SymmTensor::new(s, 0.0, 0.0, s, 0.0, s)
    }
}

/// `SphericalTensor` → `Tensor`: 対角成分に `s` を設定する。
impl From<SphericalTensor> for Tensor {
    #[inline]
    fn from(sph: SphericalTensor) -> Self {
        let s = sph.value();
        Tensor::new(s, 0.0, 0.0, 0.0, s, 0.0, 0.0, 0.0, s)
    }
}

/// `SymmTensor` → `Tensor`: 対称テンソルの完全 3×3 展開。
impl From<SymmTensor> for Tensor {
    #[inline]
    fn from(s: SymmTensor) -> Self {
        Tensor::new(
            s.xx(),
            s.xy(),
            s.xz(),
            s.xy(),
            s.yy(),
            s.yz(),
            s.xz(),
            s.yz(),
            s.zz(),
        )
    }
}
