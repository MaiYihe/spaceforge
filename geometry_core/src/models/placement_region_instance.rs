#[derive(Debug, Clone, Copy)]
pub struct PlacementTransform {
    pub matrix: [[f32; 4]; 4],
}

impl PlacementTransform {
    pub fn identity() -> Self {
        Self {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn apply_to_point(&self, p: [f32; 3]) -> [f32; 3] {
        let x = p[0];
        let y = p[1];
        let z = p[2];
        let m = &self.matrix;
        [
            x * m[0][0] + y * m[1][0] + z * m[2][0] + m[3][0],
            x * m[0][1] + y * m[1][1] + z * m[2][1] + m[3][1],
            x * m[0][2] + y * m[1][2] + z * m[2][2] + m[3][2],
        ]
    }
}

#[derive(Debug, Clone, Default)]
pub struct PlacementRegionDerived {
    pub samples: Vec<[f32; 3]>,
    pub hull: Vec<[f32; 3]>,
}

#[derive(Debug, Clone)]
pub struct PlacementRegionInstance {
    pub region_index: usize,
    pub transform: PlacementTransform,
}
