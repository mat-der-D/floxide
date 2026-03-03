use std::collections::HashMap;

use dugong_types::tensor::Vector;

use crate::error::MeshError;
use crate::global_mesh_data::GlobalMeshData;
use crate::patch::{
    CyclicPolyPatch, EmptyPolyPatch, PatchSpec, PolyPatch, ProcessorPolyPatch, SymmetryPolyPatch,
    WallPolyPatch, WedgePolyPatch,
};
use crate::primitive_mesh::PrimitiveMesh;
use crate::zone::{FaceZone, Zone};

/// Polyhedral mesh with boundary patches, zones, and parallel metadata.
///
/// Owns a `PrimitiveMesh` and augments it with:
/// - Boundary patches (`Vec<Box<dyn PolyPatch>>`)
/// - Cell/face/point zones
/// - Optional global parallel topology data
/// - Optional old-time point coordinates for dynamic meshes
pub struct PolyMesh {
    primitive: PrimitiveMesh,
    patches: Vec<Box<dyn PolyPatch>>,
    cell_zones: Vec<Zone>,
    face_zones: Vec<FaceZone>,
    point_zones: Vec<Zone>,
    global_data: Option<GlobalMeshData>,
    old_points: Option<Vec<Vector>>,
}

impl PolyMesh {
    /// Constructs a new PolyMesh from patch specifications.
    ///
    /// Internally converts each `PatchSpec` into a concrete patch type,
    /// injecting neighbor cell centers for coupled patches from the
    /// provided map.
    ///
    /// # Errors
    ///
    /// Returns an error if patch face ranges are invalid or coupled patch
    /// neighbor data is missing/mismatched.
    pub fn new(
        primitive: PrimitiveMesh,
        mut patch_specs: Vec<PatchSpec>,
        mut neighbor_centers: HashMap<String, Vec<Vector>>,
        cell_zones: Vec<Zone>,
        face_zones: Vec<FaceZone>,
        point_zones: Vec<Zone>,
    ) -> Result<Self, MeshError> {
        // Sort by start index
        patch_specs.sort_by_key(|s| s.start());

        let boundary_start = primitive.n_internal_faces();
        let boundary_end = primitive.n_faces();

        // Validate face range coverage
        if !patch_specs.is_empty() {
            let patch_start = patch_specs.first().unwrap().start();
            let patch_end =
                patch_specs.last().unwrap().start() + patch_specs.last().unwrap().size();

            if patch_start != boundary_start || patch_end != boundary_end {
                return Err(MeshError::PatchFaceRangeMismatch {
                    patch_start,
                    patch_end,
                    expected_start: boundary_start,
                    expected_end: boundary_end,
                });
            }

            // Check for overlaps/gaps between adjacent patches
            for w in patch_specs.windows(2) {
                let end_a = w[0].start() + w[0].size();
                let start_b = w[1].start();
                if end_a != start_b {
                    return Err(MeshError::PatchFaceOverlapOrGap {
                        face: end_a.min(start_b),
                        patch_a: w[0].name().to_string(),
                        patch_b: w[1].name().to_string(),
                        end_a,
                        start_b,
                    });
                }
            }
        } else if boundary_start != boundary_end {
            // No patches but there are boundary faces
            return Err(MeshError::PatchFaceRangeMismatch {
                patch_start: 0,
                patch_end: 0,
                expected_start: boundary_start,
                expected_end: boundary_end,
            });
        }

        // Convert PatchSpec to concrete patch types
        let mut patches: Vec<Box<dyn PolyPatch>> = Vec::with_capacity(patch_specs.len());

        for spec in patch_specs {
            let patch: Box<dyn PolyPatch> = match spec {
                PatchSpec::Wall { name, start, size } => {
                    Box::new(WallPolyPatch::new(name, start, size))
                }
                PatchSpec::Empty { name, start, size } => {
                    Box::new(EmptyPolyPatch::new(name, start, size))
                }
                PatchSpec::Symmetry { name, start, size } => {
                    Box::new(SymmetryPolyPatch::new(name, start, size))
                }
                PatchSpec::Wedge { name, start, size } => {
                    Box::new(WedgePolyPatch::new(name, start, size))
                }
                PatchSpec::Cyclic {
                    name,
                    start,
                    size,
                    transform,
                    face_cells,
                } => {
                    let centers = neighbor_centers.remove(&name).ok_or_else(|| {
                        MeshError::MissingNeighborCenters {
                            patch_name: name.clone(),
                        }
                    })?;
                    if centers.len() != size {
                        return Err(MeshError::NeighborCentersLengthMismatch {
                            patch_name: name,
                            expected: size,
                            got: centers.len(),
                        });
                    }
                    Box::new(CyclicPolyPatch::new(
                        name, start, size, transform, face_cells, centers,
                    ))
                }
                PatchSpec::Processor {
                    name,
                    start,
                    size,
                    neighbor_rank,
                    face_cells,
                } => {
                    let centers = neighbor_centers.remove(&name).ok_or_else(|| {
                        MeshError::MissingNeighborCenters {
                            patch_name: name.clone(),
                        }
                    })?;
                    if centers.len() != size {
                        return Err(MeshError::NeighborCentersLengthMismatch {
                            patch_name: name,
                            expected: size,
                            got: centers.len(),
                        });
                    }
                    Box::new(ProcessorPolyPatch::new(
                        name,
                        start,
                        size,
                        neighbor_rank,
                        face_cells,
                        centers,
                    ))
                }
            };
            patches.push(patch);
        }

        Ok(Self {
            primitive,
            patches,
            cell_zones,
            face_zones,
            point_zones,
            global_data: None,
            old_points: None,
        })
    }

    // --- PrimitiveMesh delegation ---

    /// Returns the underlying PrimitiveMesh.
    pub fn primitive(&self) -> &PrimitiveMesh {
        &self.primitive
    }

    pub fn points(&self) -> &[Vector] {
        self.primitive.points()
    }

    pub fn faces(&self) -> &[Vec<usize>] {
        self.primitive.faces()
    }

    pub fn owner(&self) -> &[usize] {
        self.primitive.owner()
    }

    pub fn neighbor(&self) -> &[usize] {
        self.primitive.neighbor()
    }

    pub fn n_internal_faces(&self) -> usize {
        self.primitive.n_internal_faces()
    }

    pub fn n_cells(&self) -> usize {
        self.primitive.n_cells()
    }

    pub fn n_faces(&self) -> usize {
        self.primitive.n_faces()
    }

    pub fn n_points(&self) -> usize {
        self.primitive.n_points()
    }

    pub fn cell_volumes(&self) -> &[f64] {
        self.primitive.cell_volumes()
    }

    pub fn cell_centers(&self) -> &[Vector] {
        self.primitive.cell_centers()
    }

    pub fn face_centers(&self) -> &[Vector] {
        self.primitive.face_centers()
    }

    pub fn face_areas(&self) -> &[Vector] {
        self.primitive.face_areas()
    }

    pub fn cell_cells(&self) -> &[Vec<usize>] {
        self.primitive.cell_cells()
    }

    pub fn cell_faces(&self) -> &[Vec<usize>] {
        self.primitive.cell_faces()
    }

    pub fn cell_points(&self) -> &[Vec<usize>] {
        self.primitive.cell_points()
    }

    // --- PolyMesh specific ---

    /// Returns the patch list.
    pub fn patches(&self) -> &[Box<dyn PolyPatch>] {
        &self.patches
    }

    /// Returns the cell zones.
    pub fn cell_zones(&self) -> &[Zone] {
        &self.cell_zones
    }

    /// Returns the face zones.
    pub fn face_zones(&self) -> &[FaceZone] {
        &self.face_zones
    }

    /// Returns the point zones.
    pub fn point_zones(&self) -> &[Zone] {
        &self.point_zones
    }

    /// Returns global mesh data (None in serial runs).
    pub fn global_data(&self) -> Option<&GlobalMeshData> {
        self.global_data.as_ref()
    }

    /// Returns old-time point coordinates (None for static meshes).
    pub fn old_points(&self) -> Option<&[Vector]> {
        self.old_points.as_deref()
    }

    /// Sets the global mesh data.
    pub fn set_global_data(&mut self, data: GlobalMeshData) {
        self.global_data = Some(data);
    }

    /// Sets old-time point coordinates.
    pub fn set_old_points(&mut self, points: Vec<Vector>) {
        self.old_points = Some(points);
    }
}

// Static assertion: PolyMesh is Send + Sync
const _: fn() = || {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<PolyMesh>();
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::patch::{PatchKind, Transform};

    // -- Test helper: create a two-cell mesh with 1 internal face and 10 boundary faces --

    fn make_two_cell_primitive() -> PrimitiveMesh {
        let points = vec![
            Vector::new(0.0, 0.0, 0.0),
            Vector::new(1.0, 0.0, 0.0),
            Vector::new(1.0, 1.0, 0.0),
            Vector::new(0.0, 1.0, 0.0),
            Vector::new(0.0, 0.0, 1.0),
            Vector::new(1.0, 0.0, 1.0),
            Vector::new(1.0, 1.0, 1.0),
            Vector::new(0.0, 1.0, 1.0),
            Vector::new(2.0, 0.0, 0.0),
            Vector::new(2.0, 1.0, 0.0),
            Vector::new(2.0, 0.0, 1.0),
            Vector::new(2.0, 1.0, 1.0),
        ];
        let faces = vec![
            vec![1, 2, 6, 5],   // f0: internal
            vec![0, 3, 2, 1],   // f1: boundary
            vec![4, 5, 6, 7],   // f2: boundary
            vec![0, 1, 5, 4],   // f3: boundary
            vec![3, 7, 6, 2],   // f4: boundary
            vec![0, 4, 7, 3],   // f5: boundary
            vec![8, 9, 11, 10], // f6: boundary
            vec![1, 8, 10, 5],  // f7: boundary
            vec![2, 6, 11, 9],  // f8: boundary
            vec![1, 2, 9, 8],   // f9: boundary
            vec![5, 10, 11, 6], // f10: boundary
        ];
        let owner = vec![0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1];
        let neighbor = vec![1];
        PrimitiveMesh::new(points, faces, owner, neighbor).unwrap()
    }

    /// Test utility: orthogonal box PolyMesh (Req 8.8)
    fn make_test_poly_mesh() -> PolyMesh {
        let prim = make_two_cell_primitive();
        // 10 boundary faces: f1..f10, starting at index 1
        let specs = vec![PatchSpec::Wall {
            name: "defaultWall".into(),
            start: 1,
            size: 10,
        }];
        PolyMesh::new(prim, specs, HashMap::new(), vec![], vec![], vec![]).unwrap()
    }

    // -- Construction success --

    #[test]
    fn test_poly_mesh_construction_wall_only() {
        let mesh = make_test_poly_mesh();
        assert_eq!(mesh.patches().len(), 1);
        assert_eq!(mesh.patches()[0].name(), "defaultWall");
        assert_eq!(mesh.patches()[0].kind(), PatchKind::Wall);
        assert_eq!(mesh.patches()[0].size(), 10);
    }

    #[test]
    fn test_poly_mesh_construction_with_coupled_patch() {
        let prim = make_two_cell_primitive();
        let specs = vec![
            PatchSpec::Wall {
                name: "wall".into(),
                start: 1,
                size: 8,
            },
            PatchSpec::Processor {
                name: "proc0".into(),
                start: 9,
                size: 2,
                neighbor_rank: 1,
                face_cells: vec![1, 1],
            },
        ];
        let mut centers = HashMap::new();
        centers.insert(
            "proc0".into(),
            vec![Vector::new(3.0, 0.5, 0.5), Vector::new(3.0, 0.5, 0.5)],
        );
        let mesh = PolyMesh::new(prim, specs, centers, vec![], vec![], vec![]).unwrap();
        assert_eq!(mesh.patches().len(), 2);
        assert!(mesh.patches()[1].as_coupled().is_some());
    }

    #[test]
    fn test_poly_mesh_construction_empty_boundary() {
        // Mesh with 0 boundary faces (all faces internal — unusual but valid)
        let points = vec![
            Vector::new(0.0, 0.0, 0.0),
            Vector::new(1.0, 0.0, 0.0),
            Vector::new(0.0, 1.0, 0.0),
        ];
        let faces = vec![vec![0, 1, 2]];
        let owner = vec![0];
        let neighbor = vec![0]; // 1 internal face, 0 boundary
        // This PrimitiveMesh has n_internal_faces == 1, n_faces == 1 => boundary is empty
        let prim = PrimitiveMesh::new(points, faces, owner, neighbor).unwrap();
        let mesh = PolyMesh::new(prim, vec![], HashMap::new(), vec![], vec![], vec![]).unwrap();
        assert_eq!(mesh.patches().len(), 0);
    }

    // -- Construction error cases --

    #[test]
    fn test_poly_mesh_face_range_mismatch() {
        let prim = make_two_cell_primitive();
        let specs = vec![PatchSpec::Wall {
            name: "wall".into(),
            start: 1,
            size: 5, // only covers 5 of 10 boundary faces
        }];
        let result = PolyMesh::new(prim, specs, HashMap::new(), vec![], vec![], vec![]);
        assert!(matches!(
            result,
            Err(MeshError::PatchFaceRangeMismatch { .. })
        ));
    }

    #[test]
    fn test_poly_mesh_face_overlap_or_gap() {
        let prim = make_two_cell_primitive();
        let specs = vec![
            PatchSpec::Wall {
                name: "a".into(),
                start: 1,
                size: 4,
            },
            PatchSpec::Wall {
                name: "b".into(),
                start: 6, // gap at face 5
                size: 5,
            },
        ];
        let result = PolyMesh::new(prim, specs, HashMap::new(), vec![], vec![], vec![]);
        assert!(matches!(
            result,
            Err(MeshError::PatchFaceOverlapOrGap { .. })
        ));
    }

    #[test]
    fn test_poly_mesh_missing_neighbor_centers() {
        let prim = make_two_cell_primitive();
        let specs = vec![
            PatchSpec::Wall {
                name: "wall".into(),
                start: 1,
                size: 8,
            },
            PatchSpec::Cyclic {
                name: "cyc".into(),
                start: 9,
                size: 2,
                transform: Transform::Translational {
                    separation: Vector::new(2.0, 0.0, 0.0),
                },
                face_cells: vec![1, 1],
            },
        ];
        // No neighbor centers provided
        let result = PolyMesh::new(prim, specs, HashMap::new(), vec![], vec![], vec![]);
        assert!(matches!(
            result,
            Err(MeshError::MissingNeighborCenters { .. })
        ));
    }

    #[test]
    fn test_poly_mesh_neighbor_centers_length_mismatch() {
        let prim = make_two_cell_primitive();
        let specs = vec![
            PatchSpec::Wall {
                name: "wall".into(),
                start: 1,
                size: 8,
            },
            PatchSpec::Processor {
                name: "proc0".into(),
                start: 9,
                size: 2,
                neighbor_rank: 1,
                face_cells: vec![1, 1],
            },
        ];
        let mut centers = HashMap::new();
        centers.insert("proc0".into(), vec![Vector::zero()]); // 1 instead of 2
        let result = PolyMesh::new(prim, specs, centers, vec![], vec![], vec![]);
        assert!(matches!(
            result,
            Err(MeshError::NeighborCentersLengthMismatch { .. })
        ));
    }

    // -- PrimitiveMesh delegation --

    #[test]
    fn test_poly_mesh_delegates_to_primitive() {
        let mesh = make_test_poly_mesh();
        let prim = mesh.primitive();
        assert_eq!(mesh.n_cells(), prim.n_cells());
        assert_eq!(mesh.n_faces(), prim.n_faces());
        assert_eq!(mesh.n_points(), prim.n_points());
        assert_eq!(mesh.n_internal_faces(), prim.n_internal_faces());
        assert_eq!(mesh.points().len(), prim.points().len());
        assert_eq!(mesh.faces().len(), prim.faces().len());
        assert_eq!(mesh.owner().len(), prim.owner().len());
        assert_eq!(mesh.neighbor().len(), prim.neighbor().len());
        assert_eq!(mesh.cell_volumes().len(), prim.cell_volumes().len());
        assert_eq!(mesh.cell_centers().len(), prim.cell_centers().len());
        assert_eq!(mesh.face_centers().len(), prim.face_centers().len());
        assert_eq!(mesh.face_areas().len(), prim.face_areas().len());
        assert_eq!(mesh.cell_cells().len(), prim.cell_cells().len());
        assert_eq!(mesh.cell_faces().len(), prim.cell_faces().len());
        assert_eq!(mesh.cell_points().len(), prim.cell_points().len());
    }

    // -- Zones --

    #[test]
    fn test_poly_mesh_zones() {
        let prim = make_two_cell_primitive();
        let specs = vec![PatchSpec::Wall {
            name: "wall".into(),
            start: 1,
            size: 10,
        }];
        let cell_zones = vec![Zone::new("fluid".into(), vec![0, 1])];
        let face_zones = vec![FaceZone::new("fz".into(), vec![0], vec![false])];
        let point_zones = vec![Zone::new("pz".into(), vec![0, 1, 2])];
        let mesh = PolyMesh::new(
            prim,
            specs,
            HashMap::new(),
            cell_zones,
            face_zones,
            point_zones,
        )
        .unwrap();
        assert_eq!(mesh.cell_zones().len(), 1);
        assert_eq!(mesh.cell_zones()[0].name(), "fluid");
        assert_eq!(mesh.face_zones().len(), 1);
        assert_eq!(mesh.point_zones().len(), 1);
    }

    // -- GlobalMeshData and old_points --

    #[test]
    fn test_poly_mesh_global_data_default_none() {
        let mesh = make_test_poly_mesh();
        assert!(mesh.global_data().is_none());
    }

    #[test]
    fn test_poly_mesh_set_global_data() {
        let mut mesh = make_test_poly_mesh();
        mesh.set_global_data(GlobalMeshData::new(100, 200, 300, 50));
        let gd = mesh.global_data().unwrap();
        assert_eq!(gd.n_total_cells(), 100);
    }

    #[test]
    fn test_poly_mesh_old_points_default_none() {
        let mesh = make_test_poly_mesh();
        assert!(mesh.old_points().is_none());
    }

    #[test]
    fn test_poly_mesh_set_old_points() {
        let mut mesh = make_test_poly_mesh();
        let old = vec![Vector::zero(); mesh.n_points()];
        mesh.set_old_points(old);
        assert_eq!(mesh.old_points().unwrap().len(), mesh.n_points());
    }

    // -- Send + Sync --

    #[test]
    fn test_poly_mesh_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<PolyMesh>();
    }

    // -- Downcast via as_coupled on PolyMesh patches --

    #[test]
    fn test_poly_mesh_patch_downcast() {
        let prim = make_two_cell_primitive();
        let specs = vec![
            PatchSpec::Wall {
                name: "wall".into(),
                start: 1,
                size: 8,
            },
            PatchSpec::Processor {
                name: "proc0".into(),
                start: 9,
                size: 2,
                neighbor_rank: 1,
                face_cells: vec![1, 1],
            },
        ];
        let mut centers = HashMap::new();
        centers.insert(
            "proc0".into(),
            vec![Vector::new(3.0, 0.5, 0.5), Vector::new(3.0, 0.5, 0.5)],
        );
        let mesh = PolyMesh::new(prim, specs, centers, vec![], vec![], vec![]).unwrap();
        // Wall patch: as_coupled returns None
        assert!(mesh.patches()[0].as_coupled().is_none());
        // Processor patch: as_coupled returns Some
        let coupled = mesh.patches()[1].as_coupled().unwrap();
        assert_eq!(coupled.neighbor_rank(), Some(1));
    }

    // -- Patches sorted by start --

    #[test]
    fn test_poly_mesh_patches_sorted_by_start() {
        let prim = make_two_cell_primitive();
        // Provide patches in reverse order
        let specs = vec![
            PatchSpec::Wall {
                name: "b".into(),
                start: 6,
                size: 5,
            },
            PatchSpec::Wall {
                name: "a".into(),
                start: 1,
                size: 5,
            },
        ];
        let mesh = PolyMesh::new(prim, specs, HashMap::new(), vec![], vec![], vec![]).unwrap();
        assert_eq!(mesh.patches()[0].name(), "a");
        assert_eq!(mesh.patches()[1].name(), "b");
    }
}
