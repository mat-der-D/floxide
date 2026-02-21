// Dim<f64, P1, N1, N2>（Pressure 型相当）と Dim<f64, P1, N3, Z0>（Density 型相当）の減算
// 同じ V=f64 だが L と T が異なる → Sub impl が存在しない → コンパイルエラー
use dugong_types::dimension::Dim;
use typenum::{N1, N2, N3, P1, Z0};

fn main() {
    let p: Dim<f64, P1, N1, N2> = Dim::new(101325.0);
    let d: Dim<f64, P1, N3, Z0> = Dim::new(1000.0);
    let _ = p - d;
}
