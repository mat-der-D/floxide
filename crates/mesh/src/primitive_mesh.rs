use std::sync::OnceLock;

use dugong_types::tensor::Vector;

use crate::error::MeshError;
use crate::geometry;

/// The topology engine for polyhedral meshes.
///
/// Stores the minimal set of mesh data — point coordinates, face-vertex
/// connectivity, and owner/neighbor cell indices — and lazily derives
/// geometry (cell volumes, cell centers, face area vectors, face centers)
/// and connectivity (cell-cells, cell-faces, cell-points) on first access.
///
/// # Mesh topology conventions (OpenFOAM-compatible)
///
/// Faces are divided into two groups, stored contiguously in `faces`:
///
/// - **Internal faces** (`faces[0..n_internal_faces()]`): shared by two cells.
///   Each has an entry in both `owner` and `neighbor`.
/// - **Boundary faces** (`faces[n_internal_faces()..n_faces()]`): on the mesh
///   boundary with only one adjacent cell. Each has an entry in `owner` only.
///
/// The `neighbor` slice contains exactly one entry per internal face, so
/// `neighbor.len()` defines the number of internal faces.
///
/// All data is immutable after construction. Lazy fields use [`OnceLock`] so
/// the struct is `Send + Sync` without `unsafe`.
pub struct PrimitiveMesh {
    points: Vec<Vector>,
    faces: Vec<Vec<usize>>,
    owner: Vec<usize>,
    neighbor: Vec<usize>,
    n_cells: usize,

    cell_centers: OnceLock<Vec<Vector>>,
    cell_volumes: OnceLock<Vec<f64>>,
    face_centers: OnceLock<Vec<Vector>>,
    face_areas: OnceLock<Vec<Vector>>,

    cell_cells: OnceLock<Vec<Vec<usize>>>,
    cell_faces: OnceLock<Vec<Vec<usize>>>,
    cell_points: OnceLock<Vec<Vec<usize>>>,
}

impl PrimitiveMesh {
    /// Constructs a new `PrimitiveMesh` after validating all topology invariants.
    ///
    /// The number of cells is derived as `max(owner) + 1` (or 0 if `owner`
    /// is empty) and cached for O(1) access.
    ///
    /// # Arguments
    ///
    /// * `points` — Vertex coordinates.
    /// * `faces` — Each face is a list of point indices defining a polygon.
    ///   Internal faces must come first, followed by boundary faces.
    /// * `owner` — The owner cell index for each face. `owner.len()` must equal
    ///   `faces.len()`.
    /// * `neighbor` — The neighbor cell index for each internal face.
    ///   `neighbor.len()` implicitly defines the number of internal faces.
    ///
    /// # Errors
    ///
    /// Returns `Err` if any invariant is violated:
    /// - `owner.len() != faces.len()`
    /// - Any `neighbor` index `>= n_cells`
    /// - Any point index in `faces` `>= points.len()`
    pub fn new(
        points: Vec<Vector>,
        faces: Vec<Vec<usize>>,
        owner: Vec<usize>,
        neighbor: Vec<usize>,
    ) -> Result<Self, MeshError> {
        // owner length check
        if owner.len() != faces.len() {
            return Err(MeshError::OwnerLengthMismatch {
                expected: faces.len(),
                got: owner.len(),
            });
        }

        // n_cells is derived from owner
        let n_cells = owner.iter().copied().max().map_or(0, |m| m + 1);

        // neighbor index range check
        for (face, &cell) in neighbor.iter().enumerate() {
            if cell >= n_cells {
                return Err(MeshError::NeighborIndexOutOfRange {
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
            neighbor,
            n_cells,
            cell_centers: OnceLock::new(),
            cell_volumes: OnceLock::new(),
            face_centers: OnceLock::new(),
            face_areas: OnceLock::new(),
            cell_cells: OnceLock::new(),
            cell_faces: OnceLock::new(),
            cell_points: OnceLock::new(),
        })
    }

    // Basic accessors

    /// Returns the vertex coordinates.
    pub fn points(&self) -> &[Vector] {
        &self.points
    }

    /// Returns the face definitions. Each face is a list of point indices
    /// forming a polygon. Internal faces occupy `faces[0..n_internal_faces()]`,
    /// boundary faces occupy the remainder.
    pub fn faces(&self) -> &[Vec<usize>] {
        &self.faces
    }

    /// Returns the owner cell index for each face.
    ///
    /// Every face has exactly one owner cell. The area vector of a face points
    /// outward from its owner cell.
    pub fn owner(&self) -> &[usize] {
        &self.owner
    }

    /// Returns the neighbor cell index for each internal face.
    ///
    /// Only internal faces (those shared by two cells) have a neighbor.
    /// `neighbor[i]` is the cell on the opposite side of `faces[i]` from
    /// `owner[i]`. Boundary faces have no entry in this slice.
    pub fn neighbor(&self) -> &[usize] {
        &self.neighbor
    }

    /// Returns the number of internal faces.
    ///
    /// Internal faces are shared by two cells and appear at the beginning
    /// of the face list (`faces[0..n_internal_faces()]`). The remaining faces
    /// are boundary faces. Equal to `neighbor().len()`.
    pub fn n_internal_faces(&self) -> usize {
        self.neighbor.len()
    }

    /// Returns the total number of cells.
    pub fn n_cells(&self) -> usize {
        self.n_cells
    }

    /// Returns the total number of faces (internal + boundary).
    pub fn n_faces(&self) -> usize {
        self.faces.len()
    }

    /// Returns the total number of points (vertices).
    pub fn n_points(&self) -> usize {
        self.points.len()
    }

    // Lazy geometry accessors

    /// Computes and caches both face centers and face area vectors.
    ///
    /// # Safety (logical preconditions)
    ///
    /// All point indices in `faces` have been validated to be within
    /// `points` bounds by `new()`.
    fn ensure_face_geometry(&self) {
        self.face_centers.get_or_init(|| {
            let mut centers = Vec::with_capacity(self.faces.len());
            let mut areas = Vec::with_capacity(self.faces.len());
            for f in &self.faces {
                let (fc, fa) = geometry::compute_face_geometry(&self.points, f);
                centers.push(fc);
                areas.push(fa);
            }
            let _ = self.face_areas.set(areas);
            centers
        });
    }

    /// Returns the centroid of each face. Lazily computed on first access.
    ///
    /// The returned slice has length `n_faces()`.
    pub fn face_centers(&self) -> &[Vector] {
        self.ensure_face_geometry();
        // Safety: ensure_face_geometry() initializes face_centers via get_or_init(), so it is never None.
        self.face_centers.get().unwrap()
    }

    /// Returns the area vector of each face. Lazily computed on first access.
    ///
    /// Each area vector's direction is the outward normal of the face relative
    /// to its owner cell, and its magnitude equals the face area.
    /// The returned slice has length `n_faces()`.
    pub fn face_areas(&self) -> &[Vector] {
        self.ensure_face_geometry();
        // Safety: ensure_face_geometry() sets face_areas as a side effect and guarantees it is initialized.
        self.face_areas.get().unwrap()
    }

    /// Computes and caches both cell volumes and cell centers.
    ///
    /// # Safety (logical preconditions)
    ///
    /// Validated by `new()`:
    /// - All point indices in `faces` are within `points` bounds.
    /// - All `owner` elements are less than `n_cells`.
    /// - All `neighbor` elements are less than `n_cells`.
    fn ensure_cell_geometry(&self) {
        self.cell_volumes.get_or_init(|| {
            let (volumes, centers) = geometry::compute_cell_geometry(
                &self.points,
                &self.faces,
                &self.owner,
                &self.neighbor,
                self.n_cells,
            );
            let _ = self.cell_centers.set(centers);
            volumes
        });
    }

    /// Returns the volume of each cell. Lazily computed on first access.
    ///
    /// Computed via tetrahedral (pyramid) decomposition of each cell.
    /// The returned slice has length `n_cells()`.
    pub fn cell_volumes(&self) -> &[f64] {
        self.ensure_cell_geometry();
        // Safety: ensure_cell_geometry() initializes cell_volumes via get_or_init(), so it is never None.
        self.cell_volumes.get().unwrap()
    }

    /// Returns the centroid of each cell. Lazily computed on first access.
    ///
    /// Computed as the volume-weighted average of pyramid centroids.
    /// The returned slice has length `n_cells()`.
    pub fn cell_centers(&self) -> &[Vector] {
        self.ensure_cell_geometry();
        // Safety: ensure_cell_geometry() sets cell_centers as a side effect and guarantees it is initialized.
        self.cell_centers.get().unwrap()
    }

    // Lazy connectivity accessors

    /// Computes and caches the cell-to-face connectivity.
    ///
    /// # Safety (logical preconditions)
    ///
    /// Validated by `new()`:
    /// - All `owner` elements are less than `n_cells`.
    /// - All `neighbor` elements are less than `n_cells`.
    fn ensure_cell_faces(&self) -> &[Vec<usize>] {
        self.cell_faces
            .get_or_init(|| geometry::compute_cell_faces(&self.owner, &self.neighbor, self.n_cells))
    }

    /// Returns the face indices adjacent to each cell. Lazily computed on
    /// first access.
    ///
    /// `cell_faces()[c]` contains the indices (into `faces()`) of all faces
    /// that border cell `c`, including both internal and boundary faces.
    /// The returned slice has length `n_cells()`.
    pub fn cell_faces(&self) -> &[Vec<usize>] {
        self.ensure_cell_faces()
    }

    /// Returns the neighboring cell indices for each cell. Lazily computed on
    /// first access.
    ///
    /// `cell_cells()[c]` contains the indices of all cells that share an
    /// internal face with cell `c`. Boundary faces do not contribute neighbors.
    /// The returned slice has length `n_cells()`.
    ///
    /// # Safety (logical preconditions)
    ///
    /// Validated by `new()`:
    /// - All `owner` elements are less than `n_cells`.
    /// - All `neighbor` elements are less than `n_cells`.
    pub fn cell_cells(&self) -> &[Vec<usize>] {
        self.cell_cells.get_or_init(|| {
            let cf = self.ensure_cell_faces();
            geometry::compute_cell_cells(cf, &self.owner, &self.neighbor, self.n_cells)
        })
    }

    /// Returns the point indices belonging to each cell. Lazily computed on
    /// first access.
    ///
    /// `cell_points()[c]` contains all vertex indices that form cell `c`,
    /// collected from its adjacent faces with duplicates removed and sorted
    /// in ascending order. The returned slice has length `n_cells()`.
    ///
    /// # Safety (logical preconditions)
    ///
    /// Validated by `new()`:
    /// - All `owner` elements are less than `n_cells` (for `ensure_cell_faces`).
    /// - All `neighbor` elements are less than `n_cells` (for `ensure_cell_faces`).
    /// - Face indices returned by `cell_faces` are derived from `owner`/`neighbor`
    ///   and therefore within `faces` bounds.
    pub fn cell_points(&self) -> &[Vec<usize>] {
        self.cell_points.get_or_init(|| {
            let cf = self.ensure_cell_faces();
            geometry::compute_cell_points(cf, &self.faces)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    /// Single unit-cube cell (8 points, 6 faces, 0 internal faces, 1 cell).
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
            vec![0, 3, 2, 1], // f0: z- face (outward normal: -z)
            vec![4, 5, 6, 7], // f1: z+ face (outward normal: +z)
            vec![0, 1, 5, 4], // f2: y- face (outward normal: -y)
            vec![3, 7, 6, 2], // f3: y+ face (outward normal: +y)
            vec![0, 4, 7, 3], // f4: x- face (outward normal: -x)
            vec![1, 2, 6, 5], // f5: x+ face (outward normal: +x)
        ];
        let owner = vec![0, 0, 0, 0, 0, 0];
        let neighbor = vec![];
        PrimitiveMesh::new(points, faces, owner, neighbor).unwrap()
    }

    /// Two-cell mesh with one internal face.
    /// Cell 0: x=0..1, Cell 1: x=1..2, shared face at x=1.
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
        let neighbor = vec![1]; // face 0 connects cell 0 and cell 1
        PrimitiveMesh::new(points, faces, owner, neighbor).unwrap()
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
        let result = PrimitiveMesh::new(points, faces, owner, vec![]);
        assert!(matches!(result, Err(MeshError::OwnerLengthMismatch { .. })));
    }

    #[test]
    fn test_new_neighbor_index_out_of_range_returns_err() {
        let points = vec![Vector::zero(); 4];
        let faces = vec![vec![0, 1, 2], vec![1, 2, 3]];
        let owner = vec![0, 1];
        let neighbor = vec![5]; // out of range: n_cells = max(owner)+1 = 2
        let result = PrimitiveMesh::new(points, faces, owner, neighbor);
        assert!(matches!(
            result,
            Err(MeshError::NeighborIndexOutOfRange { .. })
        ));
    }

    #[test]
    fn test_new_point_index_out_of_range_returns_err() {
        let points = vec![Vector::zero(); 3];
        let faces = vec![vec![0, 1, 99]]; // 99 out of range
        let owner = vec![0];
        let result = PrimitiveMesh::new(points, faces, owner, vec![]);
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
    fn test_cell_centers_single_cube_returns_half() {
        let mesh = make_unit_cube_mesh();
        let centers = mesh.cell_centers();
        assert_eq!(centers.len(), 1);
        let expected = Vector::new(0.5, 0.5, 0.5);
        let diff = (centers[0] - expected).mag();
        assert!(
            diff < 1e-10,
            "cell center error {diff} >= 1e-10, got ({}, {}, {})",
            centers[0].x(),
            centers[0].y(),
            centers[0].z()
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
    fn test_two_cell_centers() {
        let mesh = make_two_cell_mesh();
        let centers = mesh.cell_centers();
        let expected = [Vector::new(0.5, 0.5, 0.5), Vector::new(1.5, 0.5, 0.5)];
        for (i, (c, e)) in centers.iter().zip(expected.iter()).enumerate() {
            let diff = (*c - *e).mag();
            assert!(diff < 1e-10, "cell {i} center error {diff}");
        }
    }
}
