use std::default;

use glam::Vec3;

use super::poly_primitives::WrappedPrimitiveId;
#[derive(Debug, Clone, Copy )]
pub struct PolygonPrimitive {
    pub num_vertices: usize,
    pub vertex_ids: [WrappedPrimitiveId; 4],
    pub edge_ids: [WrappedPrimitiveId; 4],
    pub face_id: WrappedPrimitiveId,
}

impl PolygonPrimitive{
    pub fn new() -> Self{
        Self{
            num_vertices: 0,
            vertex_ids: [Default::default(); 4],
            edge_ids: [Default::default(); 4],
            face_id: Default::default(),
        }
    }
    pub fn normal(&self) -> Vec3{
        Vec3::new(0.0, 0.0, 0.0)
    }
}
impl default::Default for PolygonPrimitive {
    fn default() -> Self {
        Self{
            num_vertices: 0,
            vertex_ids: [Default::default(); 4],
            edge_ids: [Default::default(); 4],
            face_id: Default::default(),
        }
    }
}




pub struct PolygonPrimitiveOld{
    pub vertices: [ Vec3; 4],
    pub num_vertices: usize,
    pub vertex_ids: [WrappedPrimitiveId; 4],
    pub edge_ids: [WrappedPrimitiveId; 4],
    pub face_id: WrappedPrimitiveId,
}

impl Default for PolygonPrimitiveOld {
    fn default() -> Self {
        Self {
            vertices: [Default::default(); 4],
            vertex_ids: [Default::default(); 4],
            edge_ids: [Default::default(); 4],
            face_id: Default::default(),
            num_vertices: Default::default(),
        }
    }
}

impl PolygonPrimitiveOld {
    pub fn new() -> Self{
        Self::default()
    }

}