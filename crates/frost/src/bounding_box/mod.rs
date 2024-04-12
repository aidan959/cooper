use glam::Vec3;

use crate::math::Vec3Tools;
mod bounding_box;
mod bounding_box_cuboid;

pub use bounding_box::BoundingVolume;

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct BoundingBox {
    pub min_coord: Vec3,
    pub max_coord: Vec3
}
// Vertex id layout 
//           7 - 6 
//  y      4 − 5 |
//    x    |   | 2
// z       0 - 1  
impl BoundingBox {
    pub const EDGES_VERTEX_IDS: [(usize, usize); 12] = [
        (0, 1),
        (1, 2),
        (3, 2),
        (0, 3),
        (4, 5),
        (5, 6),
        (7, 6),
        (4, 7),
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ];
    pub const FACES_VERTEX_IDS: [(usize,usize,usize,usize); 6] = [
        (1, 2, 6, 5),
        (0, 3, 7, 4),
        (2, 3, 7, 6),
        (1, 0, 4, 5),
        (4, 5, 6, 7),
        (0, 1, 2, 3),
    ];

    pub fn new(min_coord: Vec3, max_coord: Vec3)-> Self {
        BoundingBox {min_coord, max_coord}
    }

    pub fn from_he(center: Vec3, he: Vec3) -> Self {
        {
            let min_coord = center - he;
            let max_coord = center + he;
            BoundingBox {min_coord, max_coord}
        }
    }

    pub fn center(&self) -> Vec3 {
        Vec3::mid_point(&self.min_coord, &self.max_coord)
    }

    pub fn he(&self) -> Vec3 {
        (self.max_coord - self.min_coord) * 0.5 as f32
    }
    pub fn extents(&self) -> Vec3 {
        self.max_coord - self.min_coord
    }
    pub fn volume(&self) -> f32 {
        let e = self.extents();
        e.x * e.y * e.z
    }
    pub fn vertices(&self) -> [Vec3; 8] {
        [
            Vec3::new(self.min_coord.x, self.min_coord.y, self.min_coord.z),
            Vec3::new(self.max_coord.x, self.min_coord.y, self.min_coord.z),
            Vec3::new(self.max_coord.x, self.max_coord.y, self.min_coord.z),
            Vec3::new(self.min_coord.x, self.max_coord.y, self.min_coord.z),
            Vec3::new(self.min_coord.x, self.min_coord.y, self.max_coord.z),
            Vec3::new(self.max_coord.x, self.min_coord.y, self.max_coord.z),
            Vec3::new(self.max_coord.x, self.max_coord.y, self.max_coord.z),
            Vec3::new(self.min_coord.x, self.max_coord.y, self.max_coord.z),

        ]
    }
} 
impl BoundingVolume for BoundingBox {
    fn center(&self) -> Vec3 {
        self.center()
    }

    fn intersects(&self, other: &Self) -> bool {
        *self.min_coord <= *other.max_coord && *self.max_coord >= *other.min_coord
    }

    fn contains(&self, other: &Self) -> bool {
        *self.min_coord <= *other.min_coord && *self.max_coord >= *other.max_coord
    }
}


#[cfg(test)]
mod test {
    use glam::{DMat4, DVec4, Vec3};

    use crate::shapes::Cuboid;

    use super::{BoundingBox, BoundingVolume};

    #[test]
    fn bb_test1() {
        let cube1 = Cuboid::new(Vec3::new(0.5,0.5,0.5));
        let cube2 = Cuboid::new(Vec3::new(0.5,0.5,0.5));
        let cube1_box = cube1.bounding_box(&DMat4::from_euler(glam::EulerRot::XYZ, 0.0, 0., 0.));
        let cube2_box = cube2.bounding_box(&DMat4::from_euler(glam::EulerRot::XYZ, 0.0, 0., 0.));

        assert_eq!(cube1_box.intersects(&cube2_box), true);
    }
}