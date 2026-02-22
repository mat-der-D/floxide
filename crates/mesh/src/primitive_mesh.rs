use std::sync::OnceLock;

use dugong_types::tensor::Vector;

use crate::error::MeshError;
use crate::geometry;

pub struct PrimitiveMesh {
    points: Vec<Vector>,
    faces: Vec<Vec<usize>>,
    owner: Vec<usize>,
    neighbour: Vec<usize>,
    n_internal_faces: usize,
    n_cells: usize,

    cell_centres: OnceLock<Vec<Vector>>,
    cell_volumes: OnceLock<Vec<f64>>,
    face_centres: OnceLock<Vec<Vector>>,
    face_areas: OnceLock<Vec<Vector>>,

    cell_cells: OnceLock<Vec<Vec<usize>>>,
    cell_faces: OnceLock<Vec<Vec<usize>>>,
    cell_points: OnceLock<Vec<Vec<usize>>>,
}

impl PrimitiveMesh {
    pub fn new(
        points: Vec<Vector>,
        faces: Vec<Vec<usize>>,
        owner: Vec<usize>,
        neighbour: Vec<usize>,
        n_internal_faces: usize,
        n_cells: usize,
    ) -> Result<Self, MeshError> {
        // owner length check
        if owner.len() != faces.len() {
            return Err(MeshError::OwnerLengthMismatch {
                expected: faces.len(),
                got: owner.len(),
            });
        }

        // neighbour length check
        if neighbour.len() != n_internal_faces {
            return Err(MeshError::NeighbourLengthMismatch {
                expected: n_internal_faces,
                got: neighbour.len(),
            });
        }

        // owner index range check
        for (face, &cell) in owner.iter().enumerate() {
            if cell >= n_cells {
                return Err(MeshError::OwnerIndexOutOfRange {
                    face,
                    cell,
                    n_cells,
                });
            }
        }

        // neighbour index range check
        for (face, &cell) in neighbour.iter().enumerate() {
            if cell >= n_cells {
                return Err(MeshError::NeighbourIndexOutOfRange {
                    face,
                    cell,
                    n_cells,
                });
            }
        }

        // point index range check
        let n_points = points.len();
        for (face, f) in faces.iter().enumerate() {
            for &point in f {
                if point >= n_points {
                    return Err(MeshError::PointIndexOutOfRange {
                        face,
                        point,
                        n_points,
                    });
                }
            }
        }

        Ok(Self {
            points,
            faces,
            owner,
            neighbour,
            n_internal_faces,
            n_cells,
            cell_centres: OnceLock::new(),
            cell_volumes: OnceLock::new(),
            face_centres: OnceLock::new(),
            face_areas: OnceLock::new(),
            cell_cells: OnceLock::new(),
            cell_faces: OnceLock::new(),
            cell_points: OnceLock::new(),
        })
    }

    // Basic accessors

    pub fn points(&self) -> &[Vector] {
        &self.points
    }

    pub fn faces(&self) -> &[Vec<usize>] {
        &self.faces
    }

    pub fn owner(&self) -> &[usize] {
        &self.owner
    }

    pub fn neighbour(&self) -> &[usize] {
        &self.neighbour
    }

    pub fn n_internal_faces(&self) -> usize {
        self.n_internal_faces
    }

    pub fn n_cells(&self) -> usize {
        self.n_cells
    }

    pub fn n_faces(&self) -> usize {
        self.faces.len()
    }

    pub fn n_points(&self) -> usize {
        self.points.len()
    }

    // Lazy geometry accessors

    fn ensure_face_geometry(&self) {
        self.face_centres.get_or_init(|| {
            let mut centres = Vec::with_capacity(self.faces.len());
            let mut areas = Vec::with_capacity(self.faces.len());
            for f in &self.faces {
                let (fc, fa) = geometry::compute_face_geometry(&self.points, f);
                centres.push(fc);
                areas.push(fa);
            }
            // Initialize face_areas too
            let _ = self.face_areas.set(areas);
            centres
        });
        // If face_areas wasn't set (race condition won't happen with OnceLock, but be safe)
        self.face_areas.get_or_init(|| {
            self.faces
                .iter()
                .map(|f| geometry::compute_face_geometry(&self.points, f).1)
                .collect()
        });
    }

    pub fn face_centres(&self) -> &[Vector] {
        self.ensure_face_geometry();
        self.face_centres.get().unwrap()
    }

    pub fn face_areas(&self) -> &[Vector] {
        self.ensure_face_geometry();
        self.face_areas.get().unwrap()
    }

    fn ensure_cell_geometry(&self) {
        self.cell_volumes.get_or_init(|| {
            let (volumes, centres) = geometry::compute_cell_geometry(
                &self.points,
                &self.faces,
                &self.owner,
                &self.neighbour,
                self.n_internal_faces,
                self.n_cells,
            );
            let _ = self.cell_centres.set(centres);
            volumes
        });
        self.cell_centres.get_or_init(|| {
            let (_, centres) = geometry::compute_cell_geometry(
                &self.points,
                &self.faces,
                &self.owner,
                &self.neighbour,
                self.n_internal_faces,
                self.n_cells,
            );
            centres
        });
    }

    pub fn cell_volumes(&self) -> &[f64] {
        self.ensure_cell_geometry();
        self.cell_volumes.get().unwrap()
    }

    pub fn cell_centres(&self) -> &[Vector] {
        self.ensure_cell_geometry();
        self.cell_centres.get().unwrap()
    }

    // Lazy connectivity accessors

    fn ensure_cell_faces(&self) -> &[Vec<usize>] {
        self.cell_faces.get_or_init(|| {
            geometry::compute_cell_faces(
                &self.owner,
                &self.neighbour,
                self.n_internal_faces,
                self.n_cells,
            )
        })
    }

    pub fn cell_faces(&self) -> &[Vec<usize>] {
        self.ensure_cell_faces()
    }

    pub fn cell_cells(&self) -> &[Vec<usize>] {
        self.cell_cells.get_or_init(|| {
            let cf = self.ensure_cell_faces();
            geometry::compute_cell_cells(
                cf,
                &self.owner,
                &self.neighbour,
                self.n_internal_faces,
                self.n_cells,
            )
        })
    }

    pub fn cell_points(&self) -> &[Vec<usize>] {
        self.cell_points.get_or_init(|| {
            let cf = self.ensure_cell_faces();
            geometry::compute_cell_points(cf, &self.faces, self.n_cells)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    /// 単一立方体セル（8点・6面・n_internal_faces=0・n_cells=1）
    fn make_unit_cube_mesh() -> PrimitiveMesh {
        let points = vec![
            Vector::new(0.0, 0.0, 0.0), // 0
            Vector::new(1.0, 0.0, 0.0), // 1
            Vector::new(1.0, 1.0, 0.0), // 2
            Vector::new(0.0, 1.0, 0.0), // 3
            Vector::new(0.0, 0.0, 1.0), // 4
            Vector::new(1.0, 0.0, 1.0), // 5
            Vector::new(1.0, 1.0, 1.0), // 6
            Vector::new(0.0, 1.0, 1.0), // 7
        ];
        let faces = vec![
            vec![0, 1, 2, 3], // f0: z- face (outward normal: -z)
            vec![4, 7, 6, 5], // f1: z+ face (outward normal: +z)
            vec![0, 4, 5, 1], // f2: y- face (outward normal: -y)
            vec![2, 6, 7, 3], // f3: y+ face (outward normal: +y)
            vec![0, 3, 7, 4], // f4: x- face (outward normal: -x)
            vec![1, 5, 6, 2], // f5: x+ face (outward normal: +x)
        ];
        let owner = vec![0, 0, 0, 0, 0, 0];
        let neighbour = vec![];
        PrimitiveMesh::new(points, faces, owner, neighbour, 0, 1).unwrap()
    }

    /// 2セルメッシュ（内部面1つ）
    /// セル0: x=0..1, セル1: x=1..2, 共有面: x=1
    fn make_two_cell_mesh() -> PrimitiveMesh {
        let points = vec![
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
        // Internal face first (OpenFOAM convention)
        let faces = vec![
            vec![1, 5, 6, 2],   // f0: internal face at x=1 (owner=0→neighbour=1, normal +x)
            vec![0, 1, 2, 3],   // f1: cell0 z- boundary (normal -z)
            vec![4, 7, 6, 5],   // f2: cell0 z+ boundary (normal +z)
            vec![0, 4, 5, 1],   // f3: cell0 y- boundary (normal -y)
            vec![2, 6, 7, 3],   // f4: cell0 y+ boundary (normal +y)
            vec![0, 3, 7, 4],   // f5: cell0 x- boundary (normal -x)
            vec![8, 10, 11, 9], // f6: cell1 x+ boundary (normal +x)
            vec![1, 5, 10, 8],  // f7: cell1 y- boundary (normal -y)
            vec![2, 9, 11, 6],  // f8: cell1 y+ boundary (normal +y)
            vec![1, 8, 9, 2],   // f9: cell1 z- boundary (normal -z)
            vec![5, 6, 11, 10], // f10: cell1 z+ boundary (normal +z)
        ];
        let owner = vec![0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1];
        let neighbour = vec![1]; // face 0 connects cell 0 and cell 1
        PrimitiveMesh::new(points, faces, owner, neighbour, 1, 2).unwrap()
    }

    // ===== Task 7.1: Test helpers =====

    #[test]
    fn test_make_unit_cube_mesh_succeeds() {
        let mesh = make_unit_cube_mesh();
        assert_eq!(mesh.n_cells(), 1);
        assert_eq!(mesh.n_faces(), 6);
        assert_eq!(mesh.n_points(), 8);
    }

    #[test]
    fn test_make_two_cell_mesh_succeeds() {
        let mesh = make_two_cell_mesh();
        assert_eq!(mesh.n_cells(), 2);
        assert_eq!(mesh.n_internal_faces(), 1);
    }

    // ===== Task 7.2: Constructor error tests =====

    #[test]
    fn test_new_owner_length_mismatch_returns_err() {
        let points = vec![Vector::zero(); 4];
        let faces = vec![vec![0, 1, 2]];
        let owner = vec![0, 0]; // wrong length
        let result = PrimitiveMesh::new(points, faces, owner, vec![], 0, 1);
        assert!(matches!(result, Err(MeshError::OwnerLengthMismatch { .. })));
    }

    #[test]
    fn test_new_neighbour_length_mismatch_returns_err() {
        let points = vec![Vector::zero(); 4];
        let faces = vec![vec![0, 1, 2]];
        let owner = vec![0];
        let result = PrimitiveMesh::new(points, faces, owner, vec![0], 0, 1); // neighbour len=1 but n_internal=0
        assert!(matches!(
            result,
            Err(MeshError::NeighbourLengthMismatch { .. })
        ));
    }

    #[test]
    fn test_new_owner_index_out_of_range_returns_err() {
        let points = vec![Vector::zero(); 4];
        let faces = vec![vec![0, 1, 2]];
        let owner = vec![5]; // out of range for n_cells=1
        let result = PrimitiveMesh::new(points, faces, owner, vec![], 0, 1);
        assert!(matches!(
            result,
            Err(MeshError::OwnerIndexOutOfRange { .. })
        ));
    }

    #[test]
    fn test_new_neighbour_index_out_of_range_returns_err() {
        let points = vec![Vector::zero(); 4];
        let faces = vec![vec![0, 1, 2], vec![1, 2, 3]];
        let owner = vec![0, 1];
        let neighbour = vec![5]; // out of range
        let result = PrimitiveMesh::new(points, faces, owner, neighbour, 1, 2);
        assert!(matches!(
            result,
            Err(MeshError::NeighbourIndexOutOfRange { .. })
        ));
    }

    #[test]
    fn test_new_point_index_out_of_range_returns_err() {
        let points = vec![Vector::zero(); 3];
        let faces = vec![vec![0, 1, 99]]; // 99 out of range
        let owner = vec![0];
        let result = PrimitiveMesh::new(points, faces, owner, vec![], 0, 1);
        assert!(matches!(
            result,
            Err(MeshError::PointIndexOutOfRange { .. })
        ));
    }

    #[test]
    fn test_new_valid_single_cube_succeeds() {
        let mesh = make_unit_cube_mesh();
        assert_eq!(mesh.n_cells(), 1);
    }

    // ===== Task 7.3: Cell geometry precision tests =====

    #[test]
    fn test_cell_volumes_single_cube_returns_one() {
        let mesh = make_unit_cube_mesh();
        let vols = mesh.cell_volumes();
        assert_eq!(vols.len(), 1);
        let rel_err = (vols[0] - 1.0).abs() / 1.0;
        assert!(
            rel_err < 1e-10,
            "cell volume relative error {rel_err} >= 1e-10, got {}",
            vols[0]
        );
    }

    #[test]
    fn test_cell_centres_single_cube_returns_half() {
        let mesh = make_unit_cube_mesh();
        let centres = mesh.cell_centres();
        assert_eq!(centres.len(), 1);
        let expected = Vector::new(0.5, 0.5, 0.5);
        let diff = (centres[0] - expected).mag();
        assert!(
            diff < 1e-10,
            "cell centre error {diff} >= 1e-10, got ({}, {}, {})",
            centres[0].x(),
            centres[0].y(),
            centres[0].z()
        );
    }

    #[test]
    fn test_cell_volumes_cached_on_second_call() {
        let mesh = make_unit_cube_mesh();
        let ptr1 = mesh.cell_volumes().as_ptr();
        let ptr2 = mesh.cell_volumes().as_ptr();
        assert_eq!(ptr1, ptr2, "cell_volumes should return same pointer");
    }

    // ===== Task 7.4: Face geometry precision tests =====

    #[test]
    fn test_face_areas_single_cube_norm_equals_one() {
        let mesh = make_unit_cube_mesh();
        let areas = mesh.face_areas();
        assert_eq!(areas.len(), 6);
        for (i, a) in areas.iter().enumerate() {
            let norm = a.mag();
            let rel_err = (norm - 1.0).abs() / 1.0;
            assert!(
                rel_err < 1e-10,
                "face {i} area norm relative error {rel_err} >= 1e-10, got {norm}"
            );
        }
    }

    #[test]
    fn test_face_areas_sum_zero_for_closed_cell() {
        // For a single closed cell, the sum of all face area vectors (outward) should be zero
        let mesh = make_unit_cube_mesh();
        let areas = mesh.face_areas();
        let mut sum = Vector::zero();
        for a in areas {
            sum = sum + *a;
        }
        let mag = sum.mag();
        assert!(mag < 1e-12, "face area vector sum magnitude {mag} >= 1e-12");
    }

    // ===== Task 7.5: Connectivity and Send/Sync tests =====

    #[test]
    fn test_cell_faces_contains_all_adjacent_faces() {
        let mesh = make_unit_cube_mesh();
        let cf = mesh.cell_faces();
        assert_eq!(cf.len(), 1);
        assert_eq!(cf[0].len(), 6, "unit cube cell should have 6 faces");
    }

    #[test]
    fn test_cell_cells_correct_neighbors() {
        let mesh = make_two_cell_mesh();
        let cc = mesh.cell_cells();
        assert_eq!(cc.len(), 2);
        assert!(cc[0].contains(&1), "cell 0 should neighbor cell 1");
        assert!(cc[1].contains(&0), "cell 1 should neighbor cell 0");
    }

    #[test]
    fn test_cell_points_no_duplicates() {
        let mesh = make_unit_cube_mesh();
        let cp = mesh.cell_points();
        assert_eq!(cp.len(), 1);
        let pts = &cp[0];
        let mut sorted = pts.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(
            pts.len(),
            sorted.len(),
            "cell_points should have no duplicates"
        );
        assert_eq!(pts.len(), 8, "unit cube should have 8 points");
    }

    #[test]
    fn test_primitive_mesh_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<PrimitiveMesh>();
    }

    // ===== Two-cell geometry tests =====

    #[test]
    fn test_two_cell_volumes() {
        let mesh = make_two_cell_mesh();
        let vols = mesh.cell_volumes();
        for (i, &v) in vols.iter().enumerate() {
            let rel_err = (v - 1.0).abs() / 1.0;
            assert!(
                rel_err < 1e-10,
                "cell {i} volume relative error {rel_err}, got {v}"
            );
        }
    }

    #[test]
    fn test_two_cell_centres() {
        let mesh = make_two_cell_mesh();
        let centres = mesh.cell_centres();
        let expected = [Vector::new(0.5, 0.5, 0.5), Vector::new(1.5, 0.5, 0.5)];
        for (i, (c, e)) in centres.iter().zip(expected.iter()).enumerate() {
            let diff = (*c - *e).mag();
            assert!(diff < 1e-10, "cell {i} centre error {diff}");
        }
    }
}
