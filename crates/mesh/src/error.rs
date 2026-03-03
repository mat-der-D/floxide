#[derive(Debug, thiserror::Error)]
pub enum MeshError {
    #[error("owner length mismatch: expected {expected}, got {got}")]
    OwnerLengthMismatch { expected: usize, got: usize },
    #[error("owner index out of range: face {face}, cell {cell}, n_cells {n_cells}")]
    OwnerIndexOutOfRange {
        face: usize,
        cell: usize,
        n_cells: usize,
    },
    #[error("neighbor index out of range: face {face}, cell {cell}, n_cells {n_cells}")]
    NeighborIndexOutOfRange {
        face: usize,
        cell: usize,
        n_cells: usize,
    },
    #[error("point index out of range: face {face}, point {point}, n_points {n_points}")]
    PointIndexOutOfRange {
        face: usize,
        point: usize,
        n_points: usize,
    },
}
