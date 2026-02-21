use typenum::Integer;

use super::dim::Dim;

/// 次元付き量が保持する値の型を公開するトレイト。
///
/// `fvm` 演算子が `T: Quantity where T::Value: FieldValue` として
/// 次元付きフィールドを受け取り、次元なし `FvMatrix` を返す境界を定義する。
pub trait Quantity {
    /// 内部値の型。`FieldValue` bound と組み合わせて上位レイヤーと接続する。
    type Value;
}

impl<V, M: Integer, L: Integer, T: Integer> Quantity for Dim<V, M, L, T> {
    type Value = V;
}

#[cfg(test)]
mod tests {
    use super::*;
    use typenum::{N1, N2, P1};

    #[test]
    fn test_quantity_value_type() {
        // Dim<f64, P1, N1, N2> の Value 型が f64 であることを型アノテーションで検証
        let _: <Dim<f64, P1, N1, N2> as Quantity>::Value = 0.0_f64;
    }
}
