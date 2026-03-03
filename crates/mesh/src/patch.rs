use dugong_types::tensor::Vector;

// ---------------------------------------------------------------------------
// PatchKind
// ---------------------------------------------------------------------------

/// Patch kind identifier for runtime dispatch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatchKind {
    Wall,
    Cyclic,
    Processor,
    Empty,
    Symmetry,
    Wedge,
}

// ---------------------------------------------------------------------------
// Transform
// ---------------------------------------------------------------------------

/// Geometric transform for cyclic patches.
#[derive(Debug, Clone, PartialEq)]
pub enum Transform {
    /// Translational periodicity with a separation vector.
    Translational { separation: Vector },
    /// Rotational periodicity around an axis.
    Rotational {
        axis: Vector,
        angle: f64,
        center: Vector,
    },
}

// ---------------------------------------------------------------------------
// PolyPatch trait
// ---------------------------------------------------------------------------

/// Common interface for all boundary patches.
///
/// Object-safe: can be used as `Box<dyn PolyPatch>`.
/// Requires `Send + Sync` for thread safety.
pub trait PolyPatch: Send + Sync {
    /// Returns the patch name.
    fn name(&self) -> &str;

    /// Returns the start face index in the global face list.
    fn start(&self) -> usize;

    /// Returns the number of faces in this patch.
    fn size(&self) -> usize;

    /// Returns the patch kind.
    fn kind(&self) -> PatchKind;

    /// Attempts to downcast to a coupled patch (immutable).
    /// Default returns `None` for non-coupled patches.
    fn as_coupled(&self) -> Option<&dyn CoupledPatch> {
        None
    }

    /// Attempts to downcast to a coupled patch (mutable).
    /// Default returns `None` for non-coupled patches.
    fn as_coupled_mut(&mut self) -> Option<&mut dyn CoupledPatch> {
        None
    }

    /// Hook called when mesh points are moved.
    /// Default is a no-op.
    fn move_points(&mut self, _points: &[Vector]) {}
}

// ---------------------------------------------------------------------------
// CoupledPatch trait
// ---------------------------------------------------------------------------

/// Sub-trait for coupled patches (patches with neighbor cell info).
///
/// Object-safe: can be used as `&dyn CoupledPatch`.
pub trait CoupledPatch: PolyPatch {
    /// Returns the face-to-cell mapping for this patch.
    fn face_cells(&self) -> &[usize];

    /// Returns the neighbor cell centers.
    fn neighbor_cell_centers(&self) -> &[Vector];

    /// Returns the neighbor rank number.
    /// `Some(rank)` for processor patches, `None` for cyclic patches.
    fn neighbor_rank(&self) -> Option<i32>;

    /// Returns the geometric transform (for cyclic patches).
    /// `None` for processor patches.
    fn transform(&self) -> Option<&Transform>;
}

// ---------------------------------------------------------------------------
// PatchSpec
// ---------------------------------------------------------------------------

/// Lightweight patch specification for PolyMesh construction.
#[derive(Debug, Clone)]
pub enum PatchSpec {
    Wall {
        name: String,
        start: usize,
        size: usize,
    },
    Cyclic {
        name: String,
        start: usize,
        size: usize,
        transform: Transform,
        face_cells: Vec<usize>,
    },
    Processor {
        name: String,
        start: usize,
        size: usize,
        neighbor_rank: i32,
        face_cells: Vec<usize>,
    },
    Empty {
        name: String,
        start: usize,
        size: usize,
    },
    Symmetry {
        name: String,
        start: usize,
        size: usize,
    },
    Wedge {
        name: String,
        start: usize,
        size: usize,
    },
}

impl PatchSpec {
    /// Returns the patch name.
    pub fn name(&self) -> &str {
        match self {
            Self::Wall { name, .. }
            | Self::Cyclic { name, .. }
            | Self::Processor { name, .. }
            | Self::Empty { name, .. }
            | Self::Symmetry { name, .. }
            | Self::Wedge { name, .. } => name,
        }
    }

    /// Returns the start face index.
    pub fn start(&self) -> usize {
        match self {
            Self::Wall { start, .. }
            | Self::Cyclic { start, .. }
            | Self::Processor { start, .. }
            | Self::Empty { start, .. }
            | Self::Symmetry { start, .. }
            | Self::Wedge { start, .. } => *start,
        }
    }

    /// Returns the number of faces.
    pub fn size(&self) -> usize {
        match self {
            Self::Wall { size, .. }
            | Self::Cyclic { size, .. }
            | Self::Processor { size, .. }
            | Self::Empty { size, .. }
            | Self::Symmetry { size, .. }
            | Self::Wedge { size, .. } => *size,
        }
    }

    /// Returns true if this is a coupled patch (Cyclic or Processor).
    pub fn is_coupled(&self) -> bool {
        matches!(self, Self::Cyclic { .. } | Self::Processor { .. })
    }
}

// ---------------------------------------------------------------------------
// Concrete patch types — non-coupled
// ---------------------------------------------------------------------------

macro_rules! simple_patch {
    ($name:ident, $kind:expr) => {
        pub struct $name {
            name: String,
            start: usize,
            size: usize,
        }

        impl $name {
            pub fn new(name: String, start: usize, size: usize) -> Self {
                Self { name, start, size }
            }
        }

        impl PolyPatch for $name {
            fn name(&self) -> &str {
                &self.name
            }
            fn start(&self) -> usize {
                self.start
            }
            fn size(&self) -> usize {
                self.size
            }
            fn kind(&self) -> PatchKind {
                $kind
            }
        }
    };
}

simple_patch!(WallPolyPatch, PatchKind::Wall);
simple_patch!(EmptyPolyPatch, PatchKind::Empty);
simple_patch!(SymmetryPolyPatch, PatchKind::Symmetry);
simple_patch!(WedgePolyPatch, PatchKind::Wedge);

// ---------------------------------------------------------------------------
// CyclicPolyPatch
// ---------------------------------------------------------------------------

pub struct CyclicPolyPatch {
    name: String,
    start: usize,
    size: usize,
    transform: Transform,
    face_cells: Vec<usize>,
    neighbor_cell_centers: Vec<Vector>,
}

impl CyclicPolyPatch {
    pub fn new(
        name: String,
        start: usize,
        size: usize,
        transform: Transform,
        face_cells: Vec<usize>,
        neighbor_cell_centers: Vec<Vector>,
    ) -> Self {
        Self {
            name,
            start,
            size,
            transform,
            face_cells,
            neighbor_cell_centers,
        }
    }
}

impl PolyPatch for CyclicPolyPatch {
    fn name(&self) -> &str {
        &self.name
    }
    fn start(&self) -> usize {
        self.start
    }
    fn size(&self) -> usize {
        self.size
    }
    fn kind(&self) -> PatchKind {
        PatchKind::Cyclic
    }
    fn as_coupled(&self) -> Option<&dyn CoupledPatch> {
        Some(self)
    }
    fn as_coupled_mut(&mut self) -> Option<&mut dyn CoupledPatch> {
        Some(self)
    }
}

impl CoupledPatch for CyclicPolyPatch {
    fn face_cells(&self) -> &[usize] {
        &self.face_cells
    }
    fn neighbor_cell_centers(&self) -> &[Vector] {
        &self.neighbor_cell_centers
    }
    fn neighbor_rank(&self) -> Option<i32> {
        None
    }
    fn transform(&self) -> Option<&Transform> {
        Some(&self.transform)
    }
}

// ---------------------------------------------------------------------------
// ProcessorPolyPatch
// ---------------------------------------------------------------------------

pub struct ProcessorPolyPatch {
    name: String,
    start: usize,
    size: usize,
    neighbor_rank: i32,
    face_cells: Vec<usize>,
    neighbor_cell_centers: Vec<Vector>,
}

impl ProcessorPolyPatch {
    pub fn new(
        name: String,
        start: usize,
        size: usize,
        neighbor_rank: i32,
        face_cells: Vec<usize>,
        neighbor_cell_centers: Vec<Vector>,
    ) -> Self {
        Self {
            name,
            start,
            size,
            neighbor_rank,
            face_cells,
            neighbor_cell_centers,
        }
    }
}

impl PolyPatch for ProcessorPolyPatch {
    fn name(&self) -> &str {
        &self.name
    }
    fn start(&self) -> usize {
        self.start
    }
    fn size(&self) -> usize {
        self.size
    }
    fn kind(&self) -> PatchKind {
        PatchKind::Processor
    }
    fn as_coupled(&self) -> Option<&dyn CoupledPatch> {
        Some(self)
    }
    fn as_coupled_mut(&mut self) -> Option<&mut dyn CoupledPatch> {
        Some(self)
    }
}

impl CoupledPatch for ProcessorPolyPatch {
    fn face_cells(&self) -> &[usize] {
        &self.face_cells
    }
    fn neighbor_cell_centers(&self) -> &[Vector] {
        &self.neighbor_cell_centers
    }
    fn neighbor_rank(&self) -> Option<i32> {
        Some(self.neighbor_rank)
    }
    fn transform(&self) -> Option<&Transform> {
        None
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- PatchKind --

    #[test]
    fn test_patch_kind_eq() {
        assert_eq!(PatchKind::Wall, PatchKind::Wall);
        assert_ne!(PatchKind::Wall, PatchKind::Cyclic);
    }

    // -- Transform --

    #[test]
    fn test_transform_translational() {
        let t = Transform::Translational {
            separation: Vector::new(1.0, 0.0, 0.0),
        };
        if let Transform::Translational { separation } = &t {
            assert_eq!(separation.x(), 1.0);
        } else {
            panic!("expected Translational");
        }
    }

    #[test]
    fn test_transform_rotational() {
        let t = Transform::Rotational {
            axis: Vector::new(0.0, 0.0, 1.0),
            angle: std::f64::consts::PI,
            center: Vector::zero(),
        };
        if let Transform::Rotational {
            axis,
            angle,
            center,
        } = &t
        {
            assert_eq!(axis.z(), 1.0);
            assert!((angle - std::f64::consts::PI).abs() < 1e-15);
            assert_eq!(center.x(), 0.0);
        } else {
            panic!("expected Rotational");
        }
    }

    #[test]
    fn test_transform_clone_eq() {
        let t = Transform::Translational {
            separation: Vector::new(1.0, 2.0, 3.0),
        };
        assert_eq!(t.clone(), t);
    }

    // -- PatchSpec --

    #[test]
    fn test_patch_spec_wall_accessors() {
        let spec = PatchSpec::Wall {
            name: "inlet".into(),
            start: 10,
            size: 5,
        };
        assert_eq!(spec.name(), "inlet");
        assert_eq!(spec.start(), 10);
        assert_eq!(spec.size(), 5);
        assert!(!spec.is_coupled());
    }

    #[test]
    fn test_patch_spec_cyclic_is_coupled() {
        let spec = PatchSpec::Cyclic {
            name: "cyc".into(),
            start: 0,
            size: 2,
            transform: Transform::Translational {
                separation: Vector::zero(),
            },
            face_cells: vec![0, 1],
        };
        assert!(spec.is_coupled());
    }

    #[test]
    fn test_patch_spec_processor_is_coupled() {
        let spec = PatchSpec::Processor {
            name: "proc0".into(),
            start: 0,
            size: 3,
            neighbor_rank: 1,
            face_cells: vec![0, 1, 2],
        };
        assert!(spec.is_coupled());
    }

    #[test]
    fn test_patch_spec_non_coupled_variants() {
        for spec in [
            PatchSpec::Empty {
                name: "e".into(),
                start: 0,
                size: 0,
            },
            PatchSpec::Symmetry {
                name: "s".into(),
                start: 0,
                size: 0,
            },
            PatchSpec::Wedge {
                name: "w".into(),
                start: 0,
                size: 0,
            },
        ] {
            assert!(!spec.is_coupled());
        }
    }

    // -- Non-coupled patch types --

    #[test]
    fn test_wall_poly_patch() {
        let p = WallPolyPatch::new("wall".into(), 4, 6);
        assert_eq!(p.name(), "wall");
        assert_eq!(p.start(), 4);
        assert_eq!(p.size(), 6);
        assert_eq!(p.kind(), PatchKind::Wall);
        assert!(p.as_coupled().is_none());
    }

    #[test]
    fn test_empty_poly_patch() {
        let p = EmptyPolyPatch::new("empty".into(), 0, 0);
        assert_eq!(p.kind(), PatchKind::Empty);
        assert!(p.as_coupled().is_none());
    }

    #[test]
    fn test_symmetry_poly_patch() {
        let p = SymmetryPolyPatch::new("sym".into(), 2, 3);
        assert_eq!(p.kind(), PatchKind::Symmetry);
        assert!(p.as_coupled().is_none());
    }

    #[test]
    fn test_wedge_poly_patch() {
        let p = WedgePolyPatch::new("wedge".into(), 5, 1);
        assert_eq!(p.kind(), PatchKind::Wedge);
        assert!(p.as_coupled().is_none());
    }

    // -- CyclicPolyPatch --

    #[test]
    fn test_cyclic_poly_patch_as_coupled() {
        let p = CyclicPolyPatch::new(
            "cyclic0".into(),
            10,
            2,
            Transform::Translational {
                separation: Vector::new(1.0, 0.0, 0.0),
            },
            vec![0, 1],
            vec![Vector::new(0.5, 0.5, 0.5), Vector::new(1.5, 0.5, 0.5)],
        );
        assert_eq!(p.kind(), PatchKind::Cyclic);
        let coupled = p.as_coupled().expect("CyclicPolyPatch should be coupled");
        assert!(coupled.neighbor_rank().is_none());
        assert!(coupled.transform().is_some());
        assert_eq!(coupled.face_cells(), &[0, 1]);
        assert_eq!(coupled.neighbor_cell_centers().len(), 2);
    }

    #[test]
    fn test_cyclic_poly_patch_as_coupled_mut() {
        let mut p = CyclicPolyPatch::new(
            "cyclic0".into(),
            10,
            2,
            Transform::Translational {
                separation: Vector::zero(),
            },
            vec![0, 1],
            vec![Vector::zero(), Vector::zero()],
        );
        assert!(p.as_coupled_mut().is_some());
    }

    // -- ProcessorPolyPatch --

    #[test]
    fn test_processor_poly_patch_as_coupled() {
        let p = ProcessorPolyPatch::new(
            "proc0to1".into(),
            20,
            3,
            1,
            vec![5, 6, 7],
            vec![
                Vector::new(1.0, 0.0, 0.0),
                Vector::new(2.0, 0.0, 0.0),
                Vector::new(3.0, 0.0, 0.0),
            ],
        );
        assert_eq!(p.kind(), PatchKind::Processor);
        let coupled = p
            .as_coupled()
            .expect("ProcessorPolyPatch should be coupled");
        assert_eq!(coupled.neighbor_rank(), Some(1));
        assert!(coupled.transform().is_none());
        assert_eq!(coupled.face_cells(), &[5, 6, 7]);
        assert_eq!(coupled.neighbor_cell_centers().len(), 3);
    }

    // -- Object safety / trait object tests --

    #[test]
    fn test_poly_patch_trait_object() {
        let p: Box<dyn PolyPatch> = Box::new(WallPolyPatch::new("wall".into(), 0, 5));
        assert_eq!(p.name(), "wall");
        assert!(p.as_coupled().is_none());
    }

    #[test]
    fn test_coupled_patch_trait_object() {
        let p = CyclicPolyPatch::new(
            "c".into(),
            0,
            1,
            Transform::Translational {
                separation: Vector::zero(),
            },
            vec![0],
            vec![Vector::zero()],
        );
        let coupled: &dyn CoupledPatch = p.as_coupled().unwrap();
        assert_eq!(coupled.name(), "c");
        assert_eq!(coupled.face_cells().len(), 1);
    }

    // -- Send + Sync --

    #[test]
    fn test_patch_types_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<WallPolyPatch>();
        assert_send_sync::<EmptyPolyPatch>();
        assert_send_sync::<SymmetryPolyPatch>();
        assert_send_sync::<WedgePolyPatch>();
        assert_send_sync::<CyclicPolyPatch>();
        assert_send_sync::<ProcessorPolyPatch>();
    }

    #[test]
    fn test_box_dyn_poly_patch_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Box<dyn PolyPatch>>();
    }
}
