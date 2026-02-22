use std::collections::BTreeSet;

use dugong_types::tensor::Vector;

/// 面の重心と面積ベクトルを計算する。
///
/// ファン三角形分割により、任意多角形の面に対応する。
/// 戻り値: `(face_centre, face_area_vector)`
pub(crate) fn compute_face_geometry(points: &[Vector], face: &[usize]) -> (Vector, Vector) {
    let n = face.len();

    // 参照点: 面頂点の単純平均
    let mut p_ref = Vector::zero();
    for &idx in face {
        p_ref += points[idx];
    }
    p_ref /= n as f64;

    let mut total_area_vec = Vector::zero();
    let mut weighted_centre = Vector::zero();

    for i in 0..n {
        let v_cur = points[face[i]];
        let v_next = points[face[(i + 1) % n]];

        // 三角形面積ベクトル: 0.5 * (v_next - p_ref) × (v_cur - p_ref)
        let tri_area_vec = (v_next - p_ref).cross(&(v_cur - p_ref)) * 0.5;
        let tri_area = tri_area_vec.mag();
        let tri_centre = (v_cur + v_next + p_ref) / 3.0;

        total_area_vec += tri_area_vec;
        weighted_centre += tri_centre * tri_area;
    }

    let total_area = total_area_vec.mag();
    let face_centre = if total_area > 1e-30 {
        weighted_centre / total_area
    } else {
        p_ref
    };

    (face_centre, total_area_vec)
}

/// 全セルの体積と重心を一括計算する。
///
/// 戻り値: `(cell_volumes, cell_centres)`
pub(crate) fn compute_cell_geometry(
    points: &[Vector],
    faces: &[Vec<usize>],
    owner: &[usize],
    neighbour: &[usize],
    n_internal_faces: usize,
    n_cells: usize,
) -> (Vec<f64>, Vec<Vector>) {
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
    for (fi, &n) in neighbour.iter().enumerate() {
        c_ref[n] += face_geom[fi].0;
        cell_face_count[n] += 1;
    }
    for ci in 0..n_cells {
        if cell_face_count[ci] > 0 {
            c_ref[ci] /= cell_face_count[ci] as f64;
        }
    }

    let mut cell_volumes = vec![0.0_f64; n_cells];
    let mut cell_centres_weighted = vec![Vector::zero(); n_cells];

    // owner 面の寄与（正の向き）
    for (fi, &o) in owner.iter().enumerate() {
        let (fc, fa) = face_geom[fi];
        let pyr_vol = fa * (fc - c_ref[o]) / 3.0;
        let pyr_centre = c_ref[o] * 0.75 + fc * 0.25;
        cell_volumes[o] += pyr_vol;
        cell_centres_weighted[o] += pyr_centre * pyr_vol;
    }

    // neighbour 面の寄与（負の向き → 面積ベクトルを反転）
    for fi in 0..n_internal_faces {
        let n = neighbour[fi];
        let (fc, fa) = face_geom[fi];
        let pyr_vol = (-fa) * (fc - c_ref[n]) / 3.0;
        let pyr_centre = c_ref[n] * 0.75 + fc * 0.25;
        cell_volumes[n] += pyr_vol;
        cell_centres_weighted[n] += pyr_centre * pyr_vol;
    }

    let mut cell_centres = vec![Vector::zero(); n_cells];
    for ci in 0..n_cells {
        if cell_volumes[ci].abs() > 1e-30 {
            cell_centres[ci] = cell_centres_weighted[ci] / cell_volumes[ci];
        } else {
            cell_centres[ci] = c_ref[ci];
        }
    }

    (cell_volumes, cell_centres)
}

/// 各セルの所属面インデックスリストを構築する。
pub(crate) fn compute_cell_faces(
    owner: &[usize],
    neighbour: &[usize],
    n_internal_faces: usize,
    n_cells: usize,
) -> Vec<Vec<usize>> {
    let mut result = vec![Vec::new(); n_cells];
    for (fi, &o) in owner.iter().enumerate() {
        result[o].push(fi);
    }
    for fi in 0..n_internal_faces {
        result[neighbour[fi]].push(fi);
    }
    result
}

/// 各セルの隣接セルリストを導出する。
pub(crate) fn compute_cell_cells(
    cell_faces: &[Vec<usize>],
    owner: &[usize],
    neighbour: &[usize],
    n_internal_faces: usize,
    n_cells: usize,
) -> Vec<Vec<usize>> {
    let mut result = vec![Vec::new(); n_cells];
    for fi in 0..n_internal_faces {
        let o = owner[fi];
        let n = neighbour[fi];
        result[o].push(n);
        result[n].push(o);
    }
    let _ = cell_faces; // signature kept per design
    result
}

/// 各セルの頂点インデックスを重複なく収集する。
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
