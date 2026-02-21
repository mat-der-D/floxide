#[cfg(test)]
mod tests {
    use crate::tensor::*;

    const EPS: f64 = 1e-14;

    fn assert_approx_eq(a: f64, b: f64) {
        let diff = (a - b).abs();
        if b.abs() > 1.0 {
            assert!(diff / b.abs() < 1e-10, "relative: {a} vs {b}, diff={diff}");
        } else {
            assert!(diff < EPS, "absolute: {a} vs {b}, diff={diff}");
        }
    }

    fn assert_vec_approx_eq(a: Vector, b: Vector) {
        assert_approx_eq(a.x(), b.x());
        assert_approx_eq(a.y(), b.y());
        assert_approx_eq(a.z(), b.z());
    }

    fn assert_tensor_approx_eq(a: Tensor, b: Tensor) {
        for i in 0..9 {
            assert_approx_eq(a.as_array()[i], b.as_array()[i]);
        }
    }

    fn assert_symm_approx_eq(a: SymmTensor, b: SymmTensor) {
        for i in 0..6 {
            assert_approx_eq(a.as_array()[i], b.as_array()[i]);
        }
    }

    // ===== 6.1 型定義と同型演算のテスト =====

    #[test]
    fn test_vector_new_and_accessors() {
        let v = Vector::new(1.0, 2.0, 3.0);
        assert_eq!(v.x(), 1.0);
        assert_eq!(v.y(), 2.0);
        assert_eq!(v.z(), 3.0);
        assert_eq!(v.as_array(), &[1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_tensor_new_row_major_order() {
        let t = Tensor::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
        assert_eq!(t.xx(), 1.0);
        assert_eq!(t.xy(), 2.0);
        assert_eq!(t.xz(), 3.0);
        assert_eq!(t.yx(), 4.0);
        assert_eq!(t.yy(), 5.0);
        assert_eq!(t.yz(), 6.0);
        assert_eq!(t.zx(), 7.0);
        assert_eq!(t.zy(), 8.0);
        assert_eq!(t.zz(), 9.0);
        assert_eq!(t.as_array(), &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0]);
    }

    #[test]
    fn test_symm_tensor_component_mapping() {
        let s = SymmTensor::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);
        assert_eq!(s.xx(), 1.0);
        assert_eq!(s.xy(), 2.0);
        assert_eq!(s.xz(), 3.0);
        assert_eq!(s.yy(), 4.0);
        assert_eq!(s.yz(), 5.0);
        assert_eq!(s.zz(), 6.0);
        assert_eq!(s.as_array(), &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    }

    #[test]
    fn test_spherical_tensor_value() {
        let s = SphericalTensor::new(3.14);
        assert_eq!(s.value(), 3.14);
    }

    #[test]
    fn test_copy_semantics() {
        let v1 = Vector::new(1.0, 2.0, 3.0);
        let v2 = v1; // Copy
        assert_eq!(v1, v2); // v1 is still valid

        let t1 = Tensor::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
        let t2 = t1;
        assert_eq!(t1, t2);

        let s1 = SymmTensor::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);
        let s2 = s1;
        assert_eq!(s1, s2);

        let sp1 = SphericalTensor::new(5.0);
        let sp2 = sp1;
        assert_eq!(sp1, sp2);
    }

    #[test]
    fn test_vector_add_sub_component_wise() {
        let a = Vector::new(1.0, 2.0, 3.0);
        let b = Vector::new(4.0, 5.0, 6.0);
        assert_eq!(a + b, Vector::new(5.0, 7.0, 9.0));
        assert_eq!(a - b, Vector::new(-3.0, -3.0, -3.0));
    }

    #[test]
    fn test_tensor_add_sub() {
        let a = Tensor::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
        let b = Tensor::new(9.0, 8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0);
        let sum = a + b;
        assert_eq!(
            sum,
            Tensor::new(10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 10.0)
        );
    }

    #[test]
    fn test_tensor_scalar_mul_div() {
        let v = Vector::new(2.0, 4.0, 6.0);
        assert_eq!(v * 2.0, Vector::new(4.0, 8.0, 12.0));
        assert_eq!(v / 2.0, Vector::new(1.0, 2.0, 3.0));

        let t = Tensor::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
        assert_eq!(
            t * 3.0,
            Tensor::new(3.0, 6.0, 9.0, 12.0, 15.0, 18.0, 21.0, 24.0, 27.0)
        );
    }

    #[test]
    fn test_neg_inverts_all_components() {
        let v = Vector::new(1.0, -2.0, 3.0);
        assert_eq!(-v, Vector::new(-1.0, 2.0, -3.0));

        let t = Tensor::new(1.0, -2.0, 3.0, -4.0, 5.0, -6.0, 7.0, -8.0, 9.0);
        assert_eq!(
            -t,
            Tensor::new(-1.0, 2.0, -3.0, 4.0, -5.0, 6.0, -7.0, 8.0, -9.0)
        );

        let s = SymmTensor::new(1.0, -2.0, 3.0, -4.0, 5.0, -6.0);
        assert_eq!(-s, SymmTensor::new(-1.0, 2.0, -3.0, 4.0, -5.0, 6.0));

        let sp = SphericalTensor::new(7.0);
        assert_eq!(-sp, SphericalTensor::new(-7.0));
    }

    #[test]
    fn test_left_scalar_mul_commutativity() {
        let v = Vector::new(1.0, 2.0, 3.0);
        assert_eq!(2.0 * v, v * 2.0);

        let t = Tensor::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
        assert_eq!(3.0 * t, t * 3.0);

        let s = SymmTensor::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);
        assert_eq!(4.0 * s, s * 4.0);

        let sp = SphericalTensor::new(5.0);
        assert_eq!(2.0 * sp, sp * 2.0);
    }

    #[test]
    fn test_assign_ops_equivalence() {
        let a = Vector::new(1.0, 2.0, 3.0);
        let b = Vector::new(4.0, 5.0, 6.0);

        let mut c = a;
        c += b;
        assert_eq!(c, a + b);

        let mut c = a;
        c -= b;
        assert_eq!(c, a - b);

        let mut c = a;
        c *= 3.0;
        assert_eq!(c, a * 3.0);

        let mut c = a;
        c /= 2.0;
        assert_eq!(c, a / 2.0);
    }

    #[test]
    fn test_symm_tensor_ops() {
        let a = SymmTensor::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);
        let b = SymmTensor::new(6.0, 5.0, 4.0, 3.0, 2.0, 1.0);
        assert_eq!(a + b, SymmTensor::new(7.0, 7.0, 7.0, 7.0, 7.0, 7.0));
        assert_eq!(a - b, SymmTensor::new(-5.0, -3.0, -1.0, 1.0, 3.0, 5.0));
    }

    #[test]
    fn test_spherical_tensor_ops() {
        let a = SphericalTensor::new(3.0);
        let b = SphericalTensor::new(7.0);
        assert_eq!(a + b, SphericalTensor::new(10.0));
        assert_eq!(a - b, SphericalTensor::new(-4.0));
        assert_eq!(a * 2.0, SphericalTensor::new(6.0));
        assert_eq!(a / 3.0, SphericalTensor::new(1.0));
    }

    // ===== 6.2 異型間演算のテスト =====

    #[test]
    fn test_symm_tensor_plus_spherical_tensor() {
        let s = SymmTensor::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);
        let sp = SphericalTensor::new(10.0);
        let result = s + sp;
        assert_eq!(result, SymmTensor::new(11.0, 2.0, 3.0, 14.0, 5.0, 16.0));

        // 可換性
        assert_eq!(sp + s, s + sp);
    }

    #[test]
    fn test_symm_tensor_minus_spherical_tensor() {
        let s = SymmTensor::new(10.0, 2.0, 3.0, 20.0, 5.0, 30.0);
        let sp = SphericalTensor::new(5.0);
        assert_eq!(s - sp, SymmTensor::new(5.0, 2.0, 3.0, 15.0, 5.0, 25.0));
        assert_eq!(
            sp - s,
            SymmTensor::new(-5.0, -2.0, -3.0, -15.0, -5.0, -25.0)
        );
    }

    #[test]
    fn test_tensor_plus_symm_tensor() {
        let t = Tensor::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
        let s = SymmTensor::new(10.0, 20.0, 30.0, 40.0, 50.0, 60.0);
        // SymmTensor expands: [10, 20, 30, 20, 40, 50, 30, 50, 60]
        let result = t + s;
        assert_eq!(
            result,
            Tensor::new(11.0, 22.0, 33.0, 24.0, 45.0, 56.0, 37.0, 58.0, 69.0)
        );
    }

    #[test]
    fn test_tensor_plus_spherical_tensor() {
        let t = Tensor::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
        let sp = SphericalTensor::new(100.0);
        let result = t + sp;
        assert_eq!(
            result,
            Tensor::new(101.0, 2.0, 3.0, 4.0, 105.0, 6.0, 7.0, 8.0, 109.0)
        );
    }

    #[test]
    fn test_vector_inner_product() {
        let a = Vector::new(1.0, 2.0, 3.0);
        let b = Vector::new(4.0, 5.0, 6.0);
        // 1*4 + 2*5 + 3*6 = 4 + 10 + 18 = 32
        assert_approx_eq(a * b, 32.0);
        // 可換性
        assert_approx_eq(a * b, b * a);
    }

    #[test]
    fn test_tensor_vector_contraction() {
        let t = Tensor::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
        let v = Vector::new(1.0, 0.0, 0.0);
        // T * [1,0,0] = [1, 4, 7]
        assert_vec_approx_eq(t * v, Vector::new(1.0, 4.0, 7.0));

        let v2 = Vector::new(1.0, 2.0, 3.0);
        // T * v2 = [1+4+9, 4+10+18, 7+16+27] = [14, 32, 50]
        assert_vec_approx_eq(t * v2, Vector::new(14.0, 32.0, 50.0));
    }

    #[test]
    fn test_vector_tensor_contraction() {
        let t = Tensor::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
        let v = Vector::new(1.0, 0.0, 0.0);
        // v^T * T: [1*1+0*4+0*7, 1*2+0*5+0*8, 1*3+0*6+0*9] = [1, 2, 3]
        assert_vec_approx_eq(v * t, Vector::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_tensor_tensor_contraction() {
        // Identity * T = T
        let t = Tensor::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
        let id = Tensor::identity();
        assert_tensor_approx_eq(id * t, t);
        assert_tensor_approx_eq(t * id, t);
    }

    #[test]
    fn test_symm_tensor_vector_contraction() {
        // S = diag(1,2,3) => S * [1,1,1] = [1,2,3]
        let s = SymmTensor::new(1.0, 0.0, 0.0, 2.0, 0.0, 3.0);
        let v = Vector::new(1.0, 1.0, 1.0);
        assert_vec_approx_eq(s * v, Vector::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_symm_tensor_symm_tensor_contraction() {
        // Identity * Identity = Identity (as Tensor)
        let id = SymmTensor::identity();
        let result = id * id;
        assert_tensor_approx_eq(result, Tensor::identity());

        // Non-trivial: result is generally not symmetric
        let a = SymmTensor::new(1.0, 2.0, 0.0, 3.0, 0.0, 1.0);
        let b = SymmTensor::new(0.0, 1.0, 0.0, 0.0, 0.0, 1.0);
        let r = a * b;
        // Rxx = 1*0 + 2*1 + 0*0 = 2
        // Rxy = 1*1 + 2*0 + 0*0 = 1
        // Rxz = 1*0 + 2*0 + 0*1 = 0
        // Ryx = 2*0 + 3*1 + 0*0 = 3
        // Ryy = 2*1 + 3*0 + 0*0 = 2
        // Ryz = 2*0 + 3*0 + 0*1 = 0
        // Rzx = 0*0 + 0*1 + 1*0 = 0
        // Rzy = 0*1 + 0*0 + 1*0 = 0
        // Rzz = 0*0 + 0*0 + 1*1 = 1
        assert_tensor_approx_eq(r, Tensor::new(2.0, 1.0, 0.0, 3.0, 2.0, 0.0, 0.0, 0.0, 1.0));
        // Verify non-symmetric: Rxy=1 != Ryx=3
        assert!((r.xy() - r.yx()).abs() > 1.0);
    }

    #[test]
    fn test_double_dot_with_identity() {
        let t = Tensor::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
        let id = Tensor::identity();
        // T : I = T_xx*1 + T_yy*1 + T_zz*1 = trace(T) = 1+5+9 = 15
        assert_approx_eq(t.double_dot(&id), t.trace());
    }

    #[test]
    fn test_symm_double_dot() {
        let a = SymmTensor::new(1.0, 0.0, 0.0, 2.0, 0.0, 3.0);
        let b = SymmTensor::new(4.0, 0.0, 0.0, 5.0, 0.0, 6.0);
        // Diagonal only: 1*4 + 2*5 + 3*6 = 4+10+18 = 32
        assert_approx_eq(a.double_dot(&b), 32.0);

        // With off-diagonal
        let c = SymmTensor::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);
        let d = SymmTensor::new(1.0, 1.0, 1.0, 1.0, 1.0, 1.0);
        // 1*1 + 4*1 + 6*1 + 2*(2*1 + 3*1 + 5*1) = 11 + 2*10 = 31
        assert_approx_eq(c.double_dot(&d), 31.0);
    }

    #[test]
    fn test_outer_product() {
        let a = Vector::new(1.0, 2.0, 3.0);
        let b = Vector::new(4.0, 5.0, 6.0);
        let t = a.outer(&b);
        assert_tensor_approx_eq(
            t,
            Tensor::new(4.0, 5.0, 6.0, 8.0, 10.0, 12.0, 12.0, 15.0, 18.0),
        );
    }

    #[test]
    fn test_cross_product() {
        let a = Vector::new(1.0, 0.0, 0.0);
        let b = Vector::new(0.0, 1.0, 0.0);
        // i × j = k
        assert_vec_approx_eq(a.cross(&b), Vector::new(0.0, 0.0, 1.0));

        // 反可換性: a × b = -(b × a)
        let c = Vector::new(1.0, 2.0, 3.0);
        let d = Vector::new(4.0, 5.0, 6.0);
        assert_vec_approx_eq(c.cross(&d), -d.cross(&c));

        // 直交性: a · (a × b) ≈ 0
        assert_approx_eq(c * c.cross(&d), 0.0);
        assert_approx_eq(d * c.cross(&d), 0.0);
    }

    // ===== 6.3 型変換・From 変換・特殊値のテスト =====

    #[test]
    fn test_symm_plus_skew_equals_original() {
        let t = Tensor::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
        let symm_t = Tensor::from(t.symm());
        let skew_t = t.skew();
        assert_tensor_approx_eq(symm_t + skew_t, t);
    }

    #[test]
    fn test_two_symm() {
        let t = Tensor::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
        let ts = t.two_symm();
        let s = t.symm();
        // two_symm = 2 * symm
        assert_symm_approx_eq(ts, s * 2.0);
    }

    #[test]
    fn test_dev_trace_is_zero() {
        let t = Tensor::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
        assert_approx_eq(t.dev().trace(), 0.0);

        let s = SymmTensor::new(10.0, 2.0, 3.0, 20.0, 5.0, 30.0);
        assert_approx_eq(s.dev().trace(), 0.0);
    }

    #[test]
    fn test_trace() {
        let t = Tensor::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
        assert_approx_eq(t.trace(), 15.0);

        let s = SymmTensor::new(10.0, 2.0, 3.0, 20.0, 5.0, 30.0);
        assert_approx_eq(s.trace(), 60.0);
    }

    #[test]
    fn test_det_known_values() {
        // Identity det = 1
        assert_approx_eq(Tensor::identity().det(), 1.0);

        // Known 3x3:
        // | 1 2 3 |
        // | 0 1 4 |
        // | 5 6 0 |
        // det = 1*(0-24) - 2*(0-20) + 3*(0-5) = -24 + 40 - 15 = 1
        let t = Tensor::new(1.0, 2.0, 3.0, 0.0, 1.0, 4.0, 5.0, 6.0, 0.0);
        assert_approx_eq(t.det(), 1.0);
    }

    #[test]
    fn test_symm_tensor_det() {
        // Identity
        assert_approx_eq(SymmTensor::identity().det(), 1.0);

        // diag(2,3,4) => det = 24
        let s = SymmTensor::new(2.0, 0.0, 0.0, 3.0, 0.0, 4.0);
        assert_approx_eq(s.det(), 24.0);
    }

    #[test]
    fn test_transpose() {
        let t = Tensor::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
        let tt = t.transpose();
        assert_eq!(tt, Tensor::new(1.0, 4.0, 7.0, 2.0, 5.0, 8.0, 3.0, 6.0, 9.0));

        // Double transpose = original
        assert_eq!(tt.transpose(), t);
    }

    #[test]
    fn test_sph() {
        let t = Tensor::new(3.0, 0.0, 0.0, 0.0, 6.0, 0.0, 0.0, 0.0, 9.0);
        // trace = 18, sph = 6
        assert_approx_eq(t.sph().value(), 6.0);

        let s = SymmTensor::new(3.0, 0.0, 0.0, 6.0, 0.0, 9.0);
        assert_approx_eq(s.sph().value(), 6.0);
    }

    #[test]
    fn test_frobenius_norm() {
        let t = Tensor::new(1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        assert_approx_eq(t.mag(), 1.0);

        let t2 = Tensor::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
        // sum of squares: 1+4+9+16+25+36+49+64+81 = 285
        assert_approx_eq(t2.mag(), 285.0_f64.sqrt());
    }

    #[test]
    fn test_vector_mag() {
        let v = Vector::new(3.0, 4.0, 0.0);
        assert_approx_eq(v.mag(), 5.0);
        assert_approx_eq(v.mag_sqr(), 25.0);
    }

    #[test]
    fn test_from_spherical_to_symm() {
        let sp = SphericalTensor::new(5.0);
        let s: SymmTensor = sp.into();
        assert_eq!(s, SymmTensor::new(5.0, 0.0, 0.0, 5.0, 0.0, 5.0));
    }

    #[test]
    fn test_from_spherical_to_tensor() {
        let sp = SphericalTensor::new(3.0);
        let t: Tensor = sp.into();
        assert_eq!(t, Tensor::new(3.0, 0.0, 0.0, 0.0, 3.0, 0.0, 0.0, 0.0, 3.0));
    }

    #[test]
    fn test_from_symm_to_tensor() {
        let s = SymmTensor::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);
        let t: Tensor = s.into();
        assert_eq!(t, Tensor::new(1.0, 2.0, 3.0, 2.0, 4.0, 5.0, 3.0, 5.0, 6.0));
    }

    #[test]
    fn test_from_roundtrip() {
        // SymmTensor → Tensor → symm() should give back the original
        let s = SymmTensor::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);
        let t: Tensor = s.into();
        assert_symm_approx_eq(t.symm(), s);
    }

    #[test]
    fn test_zero_additive_identity() {
        let v = Vector::new(1.0, 2.0, 3.0);
        assert_eq!(v + Vector::zero(), v);

        let t = Tensor::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
        assert_eq!(t + Tensor::zero(), t);

        let s = SymmTensor::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);
        assert_eq!(s + SymmTensor::zero(), s);

        let sp = SphericalTensor::new(7.0);
        assert_eq!(sp + SphericalTensor::zero(), sp);
    }

    #[test]
    fn test_identity_multiplicative() {
        let v = Vector::new(1.0, 2.0, 3.0);
        assert_vec_approx_eq(Tensor::identity() * v, v);

        assert_approx_eq(Tensor::identity().trace(), 3.0);
    }

    #[test]
    fn test_identity_trace() {
        assert_approx_eq(Tensor::identity().trace(), 3.0);
        assert_approx_eq(SymmTensor::identity().trace(), 3.0);
    }

    #[test]
    fn test_symm_tensor_dev_sph() {
        let s = SymmTensor::new(10.0, 2.0, 3.0, 20.0, 5.0, 30.0);
        // dev + sph (as SymmTensor) should equal original
        let dev = s.dev();
        let sph: SymmTensor = s.sph().into();
        assert_symm_approx_eq(dev + sph, s);
    }
}
