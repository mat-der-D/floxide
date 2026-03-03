/// Named group of cell or point indices.
pub struct Zone {
    name: String,
    indices: Vec<usize>,
}

impl Zone {
    pub fn new(name: String, indices: Vec<usize>) -> Self {
        Self { name, indices }
    }

    /// Returns the zone name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the index list.
    pub fn indices(&self) -> &[usize] {
        &self.indices
    }
}

/// Named group of face indices with flip map.
pub struct FaceZone {
    name: String,
    indices: Vec<usize>,
    flip_map: Vec<bool>,
}

impl FaceZone {
    pub fn new(name: String, indices: Vec<usize>, flip_map: Vec<bool>) -> Self {
        Self {
            name,
            indices,
            flip_map,
        }
    }

    /// Returns the zone name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the face index list.
    pub fn indices(&self) -> &[usize] {
        &self.indices
    }

    /// Returns the flip map (true if face normal should be flipped).
    pub fn flip_map(&self) -> &[bool] {
        &self.flip_map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zone_name_and_indices() {
        let zone = Zone::new("fluid".into(), vec![0, 1, 2]);
        assert_eq!(zone.name(), "fluid");
        assert_eq!(zone.indices(), &[0, 1, 2]);
    }

    #[test]
    fn test_face_zone_name_indices_flip_map() {
        let fz = FaceZone::new("internal".into(), vec![3, 4], vec![false, true]);
        assert_eq!(fz.name(), "internal");
        assert_eq!(fz.indices(), &[3, 4]);
        assert_eq!(fz.flip_map(), &[false, true]);
    }

    #[test]
    fn test_zone_empty() {
        let zone = Zone::new("empty".into(), vec![]);
        assert_eq!(zone.indices().len(), 0);
    }
}
