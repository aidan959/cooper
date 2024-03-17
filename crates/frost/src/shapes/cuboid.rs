use std::{default, ops::Mul};

use glam::Vec3;

use super::{
    poly_item::PolygonPrimitiveOld,
    poly_primitives::{PrimitiveId, WrappedPrimitiveId},
};
use crate::math::*;

pub struct Cuboid {
    pub half_extents: Vec3,
}

impl Cuboid {
    pub fn new(half_extents: Vec3) -> Cuboid {
        Cuboid { half_extents }
    }

    pub fn scaled(self, scale: Vec3) -> Self {
        Self {
            half_extents: Vec3::new(
                self.half_extents[X] * scale[X],
                self.half_extents[Y] * scale[Y],
                self.half_extents[Z] * scale[Z],
            ),
        }
    }

    pub fn support_face(&self, dir: Vec3) -> PolygonPrimitiveOld {
        let i_max = dir.max_element_index();
        let sign = (1.0 as f32).copysign(dir[i_max]);
        let he = self.half_extents;
        let vertices = match i_max {
            X => [
                Vec3::new(he.x * sign, he.y, he.z),
                Vec3::new(he.x * sign, -he.y, he.z),
                Vec3::new(he.x * sign, -he.y, -he.z),
                Vec3::new(he.x * sign, he.y, -he.z),
            ],
            Y => [
                Vec3::new(he.x, he.y * sign, he.z),
                Vec3::new(-he.x, he.y * sign, he.z),
                Vec3::new(-he.x, he.y * sign, -he.z),
                Vec3::new(he.x, he.y * sign, -he.z),
            ],
            Z => [
                Vec3::new(he.x, he.y, he.z * sign),
                Vec3::new(he.x, -he.y, he.z * sign),
                Vec3::new(-he.x, -he.y, he.z * sign),
                Vec3::new(-he.x, he.y, he.z * sign),
            ],
            _ => unreachable!(),
        };
        let sign_i = ((sign as i8 + 1) / 2) as usize;

        let vertex_ids = WrappedPrimitiveId::vertices(match i_max {
            X => [
                [0b0000, 0b0100, 0b0110, 0b0010],
                [0b1000, 0b1100, 0b1110, 0b1010],
            ][sign_i],
            Y => [
                [0b0000, 0b1000, 0b1010, 0b0010],
                [0b0100, 0b1100, 0b1110, 0b0110],
            ][sign_i],
            Z => [
                [0b0000, 0b0100, 0b1100, 0b1000],
                [0b0010, 0b0110, 0b1110, 0b1010],
            ][sign_i],
            _ => unreachable!(),
        });
        let edge_ids = WrappedPrimitiveId::edges(match i_max {
            0 => [
                [0b11_010_000, 0b11_011_010, 0b11_011_001, 0b11_001_000],
                [0b11_110_100, 0b11_111_110, 0b11_111_101, 0b11_101_100],
            ][sign_i],
            1 => [
                [0b11_100_000, 0b11_101_100, 0b11_101_001, 0b11_001_000],
                [0b11_110_010, 0b11_111_110, 0b11_111_011, 0b11_011_010],
            ][sign_i],
            2 => [
                [0b11_010_000, 0b11_110_010, 0b11_110_100, 0b11_100_000],
                [0b11_011_001, 0b11_111_011, 0b11_111_101, 0b11_101_001],
            ][sign_i],
            _ => unreachable!(),
        });
        let face_id: WrappedPrimitiveId = (i_max + sign_i * 3 + 10).into();

        PolygonPrimitiveOld {
            vertices,
            vertex_ids,
            edge_ids,
            face_id,
            num_vertices: 4,
        }
    }

    pub fn primitive_normal(&self, primitive: PrimitiveId) -> Option<Vec3> {
        match primitive {
            PrimitiveId::Vertex(id) => Some({
                let mut dir: Vec3 = Vec3::ZERO;
                for i in X..Z {
                    if id & (1 << i) != 0 {
                        dir[i] = -1.0;
                    } else {
                        dir[i] = 1.0
                    }
                }
                dir.normalize()
            }),
            PrimitiveId::Edge(id) => Some({
                let edge = id & 0b011;
                let face1 = (edge + 1) % 3;
                let face2 = (edge + 2) % 3;
                let signs = id >> 2;

                let mut dir: Vec3 = Vec3::ZERO;

                if signs & (1 << face1) != 0 {
                    dir[face1 as usize] = -1.0
                } else {
                    dir[face1 as usize] = 1.0
                }

                if signs & (1 << face2) != 0 {
                    dir[face2 as usize] = -1.0
                } else {
                    dir[face2 as usize] = 1.0;
                }
                dir.normalize()
            }),
            PrimitiveId::Face(id) => Some({
                let mut dir: Vec3 = Vec3::ZERO;

                if id < 3 {
                    dir[id as usize] = 1.0;
                } else {
                    dir[id as usize - 3] = -1.0;
                }
                dir.normalize()
            }),
            _ => None,
        }
    }
}
#[cfg(test)]
pub mod tests {
    #[test]
    fn iamax() {
        let dir = [0.0 as f32, -1.0, -3.0];
        let max_i = dir
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.total_cmp(b))
            .map(|(index, _)| index)
            .unwrap();

        println!("{}", max_i);
    }
}
