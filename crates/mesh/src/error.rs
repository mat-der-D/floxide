#[derive(Debug, thiserror::Error)]
pub enum MeshError {
    #[error("owner length mismatch: expected {expected}, got {got}")]
    OwnerLengthMismatch { expected: usize, got: usize },
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
    #[error(
        "patch face range mismatch: patches cover [{patch_start}, {patch_end}), \
         expected [{expected_start}, {expected_end})"
    )]
    PatchFaceRangeMismatch {
        patch_start: usize,
        patch_end: usize,
        expected_start: usize,
        expected_end: usize,
    },
    #[error(
        "patch face overlap or gap at face {face}: \
         patch '{patch_a}' ends at {end_a}, patch '{patch_b}' starts at {start_b}"
    )]
    PatchFaceOverlapOrGap {
        face: usize,
        patch_a: String,
        patch_b: String,
        end_a: usize,
        start_b: usize,
    },
    #[error("missing neighbor cell centers for coupled patch '{patch_name}'")]
    MissingNeighborCenters { patch_name: String },
    #[error(
        "neighbor cell centers length mismatch for patch '{patch_name}': \
         expected {expected}, got {got}"
    )]
    NeighborCentersLengthMismatch {
        patch_name: String,
        expected: usize,
        got: usize,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_patch_face_range_mismatch_message() {
        let err = MeshError::PatchFaceRangeMismatch {
            patch_start: 4,
            patch_end: 8,
            expected_start: 4,
            expected_end: 10,
        };
        let msg = err.to_string();
        assert!(msg.contains("patches cover [4, 8)"));
        assert!(msg.contains("expected [4, 10)"));
    }

    #[test]
    fn test_patch_face_overlap_or_gap_message() {
        let err = MeshError::PatchFaceOverlapOrGap {
            face: 6,
            patch_a: "inlet".into(),
            patch_b: "outlet".into(),
            end_a: 6,
            start_b: 8,
        };
        let msg = err.to_string();
        assert!(msg.contains("inlet"));
        assert!(msg.contains("outlet"));
    }

    #[test]
    fn test_missing_neighbor_centers_message() {
        let err = MeshError::MissingNeighborCenters {
            patch_name: "proc0".into(),
        };
        assert!(err.to_string().contains("proc0"));
    }

    #[test]
    fn test_neighbor_centers_length_mismatch_message() {
        let err = MeshError::NeighborCentersLengthMismatch {
            patch_name: "cyclic0".into(),
            expected: 10,
            got: 5,
        };
        let msg = err.to_string();
        assert!(msg.contains("cyclic0"));
        assert!(msg.contains("expected 10"));
        assert!(msg.contains("got 5"));
    }
}
