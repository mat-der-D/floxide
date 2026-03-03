//! Mesh structure and topology
//!
//! Provides finite volume mesh representation with cells, faces, and points.

mod error;
mod geometry;
mod primitive_mesh;

pub use error::MeshError;
pub use primitive_mesh::PrimitiveMesh;
