//! Mesh structure and topology
//!
//! Provides finite volume mesh representation with cells, faces, and points.

mod error;
mod geometry;
mod global_mesh_data;
mod patch;
mod poly_mesh;
mod primitive_mesh;
mod zone;

pub use error::MeshError;
pub use global_mesh_data::GlobalMeshData;
pub use patch::{
    CoupledPatch, CyclicPolyPatch, EmptyPolyPatch, PatchKind, PatchSpec, PolyPatch,
    ProcessorPolyPatch, SymmetryPolyPatch, Transform, WallPolyPatch, WedgePolyPatch,
};
pub use poly_mesh::PolyMesh;
pub use primitive_mesh::PrimitiveMesh;
pub use zone::{FaceZone, Zone};
