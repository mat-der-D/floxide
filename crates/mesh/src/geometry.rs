use std::collections::BTreeSet;

use dugong_types::tensor::Vector;

/// 面の重心と面積ベクトルを計算する。
///
/// ファン三角形分割により、任意多角形の面に対応する。
/// 戻り値: `(face_center, face_area_vector)`
///
/// # Panics
///
/// `face` の各要素が `points` の有効なインデックスでない場合、実行時パニックとなる。
pub(crate) fn compute_face_geometry(points: &[Vector], face: &[usize]) -> (Vector, Vector) {
    let n = face.len();

    // 参照点: 面頂点の単純平均
    let mut p_ref = Vector::zero();
    for &idx in face {
        p_ref += points[idx];
    }
    p_ref /= n as f64;

    let mut total_area_vec = Vector::zero();
    let mut weighted_center = Vector::zero();

    for i in 0..n {
        let v_cur = points[face[i]];
        let v_next = points[face[(i + 1) % n]];

        // 三角形面積ベクトル: 0.5 * (v_cur - p_ref) × (v_next - p_ref)
        let tri_area_vec = (v_cur - p_ref).cross(&(v_next - p_ref)) * 0.5;
        let tri_area = tri_area_vec.mag();
        let tri_center = (v_cur + v_next + p_ref) / 3.0;

        total_area_vec += tri_area_vec;
        weighted_center += tri_center * tri_area;
    }

    let total_area = total_area_vec.mag();
    let face_center = if total_area > 1e-30 {
        weighted_center / total_area
    } else {
        p_ref
    };

    (face_center, total_area_vec)
}

/// 全セルの体積と重心を一括計算する。
///
/// 戻り値: `(cell_volumes, cell_centers)`
///
/// # Panics
///
/// 以下の前提条件に違反した場合、実行時パニックとなる。
/// - `faces` の各面に含まれる頂点インデックスが `points` の範囲内であること。
/// - `owner` の各要素が `n_cells` 未満であること。
/// - `neighbor` の各要素が `n_cells` 未満であること。
pub(crate) fn compute_cell_geometry(
    points: &[Vector],
    faces: &[Vec<usize>],
    owner: &[usize],
    neighbor: &[usize],
    n_cells: usize,
) -> (Vec<f64>, Vec<Vector>) {
    let n_internal_faces = neighbor.len();
    // まず全面のジオメトリを計算
    let face_geom: Vec<(Vector, Vector)> = faces
        .iter()
        .map(|f| compute_face_geometry(points, f))
        .collect();

    // 各セルの参照点（そのセルに属する全面の面中心の平均）
    let mut cell_face_count = vec![0usize; n_cells];
    let mut c_ref = vec![Vector::zero(); n_cells];

    for (fi, &o) in owner.iter().enumerate() {
        c_ref[o] += face_geom[fi].0;
        cell_face_count[o] += 1;
    }
    for (fi, &n) in neighbor.iter().enumerate() {
        c_ref[n] += face_geom[fi].0;
        cell_face_count[n] += 1;
    }
    for ci in 0..n_cells {
        if cell_face_count[ci] > 0 {
            c_ref[ci] /= cell_face_count[ci] as f64;
        }
    }

    let mut cell_volumes = vec![0.0_f64; n_cells];
    let mut cell_centers_weighted = vec![Vector::zero(); n_cells];

    // owner 面の寄与（正の向き）
    for (fi, &o) in owner.iter().enumerate() {
        let (fc, fa) = face_geom[fi];
        let pyr_vol = fa * (fc - c_ref[o]) / 3.0;
        let pyr_center = c_ref[o] * 0.75 + fc * 0.25;
        cell_volumes[o] += pyr_vol;
        cell_centers_weighted[o] += pyr_center * pyr_vol;
    }

    // neighbor 面の寄与（負の向き → 面積ベクトルを反転）
    for fi in 0..n_internal_faces {
        let n = neighbor[fi];
        let (fc, fa) = face_geom[fi];
        let pyr_vol = (-fa) * (fc - c_ref[n]) / 3.0;
        let pyr_center = c_ref[n] * 0.75 + fc * 0.25;
        cell_volumes[n] += pyr_vol;
        cell_centers_weighted[n] += pyr_center * pyr_vol;
    }

    let mut cell_centers = vec![Vector::zero(); n_cells];
    for ci in 0..n_cells {
        if cell_volumes[ci].abs() > 1e-30 {
            cell_centers[ci] = cell_centers_weighted[ci] / cell_volumes[ci];
        } else {
            cell_centers[ci] = c_ref[ci];
        }
    }

    (cell_volumes, cell_centers)
}

/// 各セルの所属面インデックスリストを構築する。
///
/// # Panics
///
/// 以下の前提条件に違反した場合、実行時パニックとなる。
/// - `owner` の各要素が `n_cells` 未満であること。
/// - `neighbor` の各要素が `n_cells` 未満であること。
pub(crate) fn compute_cell_faces(
    owner: &[usize],
    neighbor: &[usize],
    n_cells: usize,
) -> Vec<Vec<usize>> {
    let mut result = vec![Vec::new(); n_cells];
    for (fi, &o) in owner.iter().enumerate() {
        result[o].push(fi);
    }
    for (fi, &n) in neighbor.iter().enumerate() {
        result[n].push(fi);
    }
    result
}

/// 各セルの隣接セルリストを導出する。
///
/// # Panics
///
/// 以下の前提条件に違反した場合、実行時パニックとなる。
/// - `owner[0..neighbor.len()]` および `neighbor` の各要素が `n_cells` 未満であること。
pub(crate) fn compute_cell_cells(
    cell_faces: &[Vec<usize>],
    owner: &[usize],
    neighbor: &[usize],
    n_cells: usize,
) -> Vec<Vec<usize>> {
    let mut result = vec![Vec::new(); n_cells];
    for (fi, &n) in neighbor.iter().enumerate() {
        let o = owner[fi];
        result[o].push(n);
        result[n].push(o);
    }
    let _ = cell_faces; // signature kept per design
    result
}

/// 各セルの頂点インデックスを重複なく収集する。
///
/// # Panics
///
/// 以下の前提条件に違反した場合、実行時パニックとなる。
/// - `cell_faces` の各面インデックスが `faces` の範囲内であること。
pub(crate) fn compute_cell_points(
    cell_faces: &[Vec<usize>],
    faces: &[Vec<usize>],
    n_cells: usize,
) -> Vec<Vec<usize>> {
    let mut result = Vec::with_capacity(n_cells);
    for cell_face_indices in cell_faces.iter().take(n_cells) {
        let mut pts = BTreeSet::new();
        for &fi in cell_face_indices {
            for &pi in &faces[fi] {
                pts.insert(pi);
            }
        }
        result.push(pts.into_iter().collect());
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use dugong_types::tensor::Vector;

    /// 単位正方形面（z=0 平面、owner 側 (-z) から見て反時計回り → 法線 -z）
    fn square_points() -> Vec<Vector> {
        vec![
            Vector::new(0.0, 0.0, 0.0),
            Vector::new(1.0, 0.0, 0.0),
            Vector::new(1.0, 1.0, 0.0),
            Vector::new(0.0, 1.0, 0.0),
        ]
    }

    /// 単位立方体の 8 点
    fn cube_points() -> Vec<Vector> {
        vec![
            Vector::new(0.0, 0.0, 0.0), // 0
            Vector::new(1.0, 0.0, 0.0), // 1
            Vector::new(1.0, 1.0, 0.0), // 2
            Vector::new(0.0, 1.0, 0.0), // 3
            Vector::new(0.0, 0.0, 1.0), // 4
            Vector::new(1.0, 0.0, 1.0), // 5
            Vector::new(1.0, 1.0, 1.0), // 6
            Vector::new(0.0, 1.0, 1.0), // 7
        ]
    }

    /// 単位立方体の 6 面（owner 側から見て反時計回り → 右手の法則で外向き法線）
    fn cube_faces() -> Vec<Vec<usize>> {
        vec![
            vec![0, 3, 2, 1], // f0: z- (normal -z)
            vec![4, 5, 6, 7], // f1: z+ (normal +z)
            vec![0, 1, 5, 4], // f2: y- (normal -y)
            vec![3, 7, 6, 2], // f3: y+ (normal +y)
            vec![0, 4, 7, 3], // f4: x- (normal -x)
            vec![1, 2, 6, 5], // f5: x+ (normal +x)
        ]
    }

    // ===== compute_face_geometry =====

    #[test]
    fn face_geometry_square_area_magnitude() {
        let pts = square_points();
        let (_, area_vec) = compute_face_geometry(&pts, &[0, 3, 2, 1]);
        let area = area_vec.mag();
        assert!((area - 1.0).abs() < 1e-12, "expected area 1.0, got {area}");
    }

    #[test]
    fn face_geometry_square_area_direction() {
        let pts = square_points();
        let (_, area_vec) = compute_face_geometry(&pts, &[0, 3, 2, 1]);
        // 設計通り: 0.5 * (v_cur - p_ref) × (v_next - p_ref) で owner 外向き法線を生成
        // 頂点は owner 側 (-z) から見て反時計回り → 右手の法則で -z 方向
        assert!(
            area_vec.z() < 0.0,
            "expected -z normal from cross product convention"
        );
        assert!(area_vec.x().abs() < 1e-12);
        assert!(area_vec.y().abs() < 1e-12);
    }

    #[test]
    fn face_geometry_square_centroid() {
        let pts = square_points();
        let (center, _) = compute_face_geometry(&pts, &[0, 3, 2, 1]);
        let expected = Vector::new(0.5, 0.5, 0.0);
        let diff = (center - expected).mag();
        assert!(diff < 1e-12, "centroid error {diff}");
    }

    #[test]
    fn face_geometry_triangle() {
        let pts = vec![
            Vector::new(0.0, 0.0, 0.0),
            Vector::new(2.0, 0.0, 0.0),
            Vector::new(0.0, 2.0, 0.0),
        ];
        let (center, area_vec) = compute_face_geometry(&pts, &[0, 1, 2]);
        // 三角形面積 = 2.0
        let area = area_vec.mag();
        assert!((area - 2.0).abs() < 1e-12, "expected area 2.0, got {area}");
        // 重心 = (2/3, 2/3, 0)
        let expected = Vector::new(2.0 / 3.0, 2.0 / 3.0, 0.0);
        let diff = (center - expected).mag();
        assert!(diff < 1e-12, "triangle centroid error {diff}");
    }

    // ===== compute_cell_geometry =====

    #[test]
    fn cell_geometry_single_cube_volume() {
        let pts = cube_points();
        let faces = cube_faces();
        let owner = vec![0; 6];
        let neighbor: Vec<usize> = vec![];
        let (vols, _) = compute_cell_geometry(&pts, &faces, &owner, &neighbor, 1);
        assert_eq!(vols.len(), 1);
        assert!(
            (vols[0] - 1.0).abs() < 1e-10,
            "expected volume 1.0, got {}",
            vols[0]
        );
    }

    #[test]
    fn cell_geometry_single_cube_center() {
        let pts = cube_points();
        let faces = cube_faces();
        let owner = vec![0; 6];
        let neighbor: Vec<usize> = vec![];
        let (_, centers) = compute_cell_geometry(&pts, &faces, &owner, &neighbor, 1);
        let expected = Vector::new(0.5, 0.5, 0.5);
        let diff = (centers[0] - expected).mag();
        assert!(diff < 1e-10, "center error {diff}");
    }

    #[test]
    fn cell_geometry_two_cells() {
        // セル0: x=0..1, セル1: x=1..2
        let pts = vec![
            Vector::new(0.0, 0.0, 0.0), // 0
            Vector::new(1.0, 0.0, 0.0), // 1
            Vector::new(1.0, 1.0, 0.0), // 2
            Vector::new(0.0, 1.0, 0.0), // 3
            Vector::new(0.0, 0.0, 1.0), // 4
            Vector::new(1.0, 0.0, 1.0), // 5
            Vector::new(1.0, 1.0, 1.0), // 6
            Vector::new(0.0, 1.0, 1.0), // 7
            Vector::new(2.0, 0.0, 0.0), // 8
            Vector::new(2.0, 1.0, 0.0), // 9
            Vector::new(2.0, 0.0, 1.0), // 10
            Vector::new(2.0, 1.0, 1.0), // 11
        ];
        let faces = vec![
            vec![1, 2, 6, 5],   // f0:  internal (owner=0→neighbor=1, normal +x)
            vec![0, 3, 2, 1],   // f1:  cell0 z- (normal -z)
            vec![4, 5, 6, 7],   // f2:  cell0 z+ (normal +z)
            vec![0, 1, 5, 4],   // f3:  cell0 y- (normal -y)
            vec![3, 7, 6, 2],   // f4:  cell0 y+ (normal +y)
            vec![0, 4, 7, 3],   // f5:  cell0 x- (normal -x)
            vec![8, 9, 11, 10], // f6:  cell1 x+ (normal +x)
            vec![1, 8, 10, 5],  // f7:  cell1 y- (normal -y)
            vec![2, 6, 11, 9],  // f8:  cell1 y+ (normal +y)
            vec![1, 2, 9, 8],   // f9:  cell1 z- (normal -z)
            vec![5, 10, 11, 6], // f10: cell1 z+ (normal +z)
        ];
        let owner = vec![0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1];
        let neighbor = vec![1];
        let (vols, centers) = compute_cell_geometry(&pts, &faces, &owner, &neighbor, 2);
        for i in 0..2 {
            assert!(
                (vols[i] - 1.0).abs() < 1e-10,
                "cell {i} volume error, got {}",
                vols[i]
            );
        }
        let expected = [Vector::new(0.5, 0.5, 0.5), Vector::new(1.5, 0.5, 0.5)];
        for i in 0..2 {
            let diff = (centers[i] - expected[i]).mag();
            assert!(diff < 1e-10, "cell {i} center error {diff}");
        }
    }

    // ===== compute_cell_faces =====

    #[test]
    fn cell_faces_single_cell() {
        let owner = vec![0, 0, 0, 0, 0, 0];
        let neighbor: Vec<usize> = vec![];
        let cf = compute_cell_faces(&owner, &neighbor, 1);
        assert_eq!(cf.len(), 1);
        assert_eq!(cf[0], vec![0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn cell_faces_two_cells_internal_face_in_both() {
        // f0: internal (owner=0, neighbor=1), f1..f5: cell0, f6..f10: cell1
        let owner = vec![0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1];
        let neighbor = vec![1];
        let cf = compute_cell_faces(&owner, &neighbor, 2);
        assert!(cf[0].contains(&0), "cell 0 should contain internal face 0");
        assert!(cf[1].contains(&0), "cell 1 should contain internal face 0");
        assert_eq!(cf[0].len(), 6);
        assert_eq!(cf[1].len(), 6);
    }

    // ===== compute_cell_cells =====

    #[test]
    fn cell_cells_no_internal_faces() {
        let cf = vec![vec![0, 1, 2, 3, 4, 5]];
        let owner = vec![0; 6];
        let neighbor: Vec<usize> = vec![];
        let cc = compute_cell_cells(&cf, &owner, &neighbor, 1);
        assert_eq!(cc.len(), 1);
        assert!(cc[0].is_empty());
    }

    #[test]
    fn cell_cells_two_cells_symmetric() {
        let cf = vec![vec![0, 1, 2, 3, 4, 5], vec![0, 6, 7, 8, 9, 10]];
        let owner = vec![0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1];
        let neighbor = vec![1];
        let cc = compute_cell_cells(&cf, &owner, &neighbor, 2);
        assert_eq!(cc[0], vec![1]);
        assert_eq!(cc[1], vec![0]);
    }

    // ===== compute_cell_points =====

    #[test]
    fn cell_points_single_cube() {
        let faces = cube_faces();
        let cf = vec![vec![0, 1, 2, 3, 4, 5]];
        let cp = compute_cell_points(&cf, &faces, 1);
        assert_eq!(cp.len(), 1);
        assert_eq!(cp[0], vec![0, 1, 2, 3, 4, 5, 6, 7]);
    }

    #[test]
    fn cell_points_no_duplicates() {
        let faces = cube_faces();
        let cf = vec![vec![0, 1, 2, 3, 4, 5]];
        let cp = compute_cell_points(&cf, &faces, 1);
        let mut sorted = cp[0].clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(cp[0].len(), sorted.len());
    }

    #[test]
    fn cell_points_sorted() {
        // BTreeSet を使用しているため、結果はソート済みであるべき
        let faces = cube_faces();
        let cf = vec![vec![0, 1, 2, 3, 4, 5]];
        let cp = compute_cell_points(&cf, &faces, 1);
        let mut sorted = cp[0].clone();
        sorted.sort();
        assert_eq!(cp[0], sorted);
    }
}
