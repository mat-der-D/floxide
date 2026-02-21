/// `f64` の型エイリアス。スカラー量を表す。
pub type Scalar = f64;

/// 3 次元ベクトル。内部は `[x, y, z]` の順で格納する。
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vector(pub(super) [f64; 3]);

impl Vector {
    /// 成分を指定して生成する。
    #[inline]
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self([x, y, z])
    }

    /// 内部配列への参照を返す。
    #[inline]
    pub fn as_array(&self) -> &[f64; 3] {
        &self.0
    }

    /// x 成分を返す。
    #[inline]
    pub fn x(&self) -> f64 {
        self.0[0]
    }

    /// y 成分を返す。
    #[inline]
    pub fn y(&self) -> f64 {
        self.0[1]
    }

    /// z 成分を返す。
    #[inline]
    pub fn z(&self) -> f64 {
        self.0[2]
    }
}

/// 3×3 テンソル。内部は row-major 順 `[xx, xy, xz, yx, yy, yz, zx, zy, zz]` で格納する。
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Tensor(pub(super) [f64; 9]);

impl Tensor {
    /// 9 成分を row-major 順で指定して生成する。
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        xx: f64,
        xy: f64,
        xz: f64,
        yx: f64,
        yy: f64,
        yz: f64,
        zx: f64,
        zy: f64,
        zz: f64,
    ) -> Self {
        Self([xx, xy, xz, yx, yy, yz, zx, zy, zz])
    }

    /// 内部配列への参照を返す。
    #[inline]
    pub fn as_array(&self) -> &[f64; 9] {
        &self.0
    }

    /// xx 成分を返す。
    #[inline]
    pub fn xx(&self) -> f64 {
        self.0[0]
    }

    /// xy 成分を返す。
    #[inline]
    pub fn xy(&self) -> f64 {
        self.0[1]
    }

    /// xz 成分を返す。
    #[inline]
    pub fn xz(&self) -> f64 {
        self.0[2]
    }

    /// yx 成分を返す。
    #[inline]
    pub fn yx(&self) -> f64 {
        self.0[3]
    }

    /// yy 成分を返す。
    #[inline]
    pub fn yy(&self) -> f64 {
        self.0[4]
    }

    /// yz 成分を返す。
    #[inline]
    pub fn yz(&self) -> f64 {
        self.0[5]
    }

    /// zx 成分を返す。
    #[inline]
    pub fn zx(&self) -> f64 {
        self.0[6]
    }

    /// zy 成分を返す。
    #[inline]
    pub fn zy(&self) -> f64 {
        self.0[7]
    }

    /// zz 成分を返す。
    #[inline]
    pub fn zz(&self) -> f64 {
        self.0[8]
    }
}

/// 対称テンソル。上三角 row-major 順 `[xx, xy, xz, yy, yz, zz]` の 6 独立成分で格納する。
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SymmTensor(pub(super) [f64; 6]);

impl SymmTensor {
    /// 6 独立成分を指定して生成する。
    #[inline]
    pub fn new(xx: f64, xy: f64, xz: f64, yy: f64, yz: f64, zz: f64) -> Self {
        Self([xx, xy, xz, yy, yz, zz])
    }

    /// 内部配列への参照を返す。
    #[inline]
    pub fn as_array(&self) -> &[f64; 6] {
        &self.0
    }

    /// xx 成分を返す。
    #[inline]
    pub fn xx(&self) -> f64 {
        self.0[0]
    }

    /// xy 成分を返す。
    #[inline]
    pub fn xy(&self) -> f64 {
        self.0[1]
    }

    /// xz 成分を返す。
    #[inline]
    pub fn xz(&self) -> f64 {
        self.0[2]
    }

    /// yy 成分を返す。
    #[inline]
    pub fn yy(&self) -> f64 {
        self.0[3]
    }

    /// yz 成分を返す。
    #[inline]
    pub fn yz(&self) -> f64 {
        self.0[4]
    }

    /// zz 成分を返す。
    #[inline]
    pub fn zz(&self) -> f64 {
        self.0[5]
    }
}

/// 球面テンソル。スカラー値 `s` で `sI`（単位テンソルのスカラー倍）を表す。
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SphericalTensor(pub(super) f64);

impl SphericalTensor {
    /// スカラー値を指定して生成する。
    #[inline]
    pub fn new(s: f64) -> Self {
        Self(s)
    }

    /// 内部値を返す。
    #[inline]
    pub fn value(&self) -> f64 {
        self.0
    }
}
