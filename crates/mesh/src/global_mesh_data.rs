/// Global mesh topology data for parallel computation.
pub struct GlobalMeshData {
    n_total_cells: usize,
    n_total_points: usize,
    n_total_faces: usize,
    n_total_internal_faces: usize,
}

impl GlobalMeshData {
    pub fn new(
        n_total_cells: usize,
        n_total_points: usize,
        n_total_faces: usize,
        n_total_internal_faces: usize,
    ) -> Self {
        Self {
            n_total_cells,
            n_total_points,
            n_total_faces,
            n_total_internal_faces,
        }
    }

    pub fn n_total_cells(&self) -> usize {
        self.n_total_cells
    }

    pub fn n_total_points(&self) -> usize {
        self.n_total_points
    }

    pub fn n_total_faces(&self) -> usize {
        self.n_total_faces
    }

    pub fn n_total_internal_faces(&self) -> usize {
        self.n_total_internal_faces
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_mesh_data_accessors() {
        let gmd = GlobalMeshData::new(100, 200, 300, 150);
        assert_eq!(gmd.n_total_cells(), 100);
        assert_eq!(gmd.n_total_points(), 200);
        assert_eq!(gmd.n_total_faces(), 300);
        assert_eq!(gmd.n_total_internal_faces(), 150);
    }
}
