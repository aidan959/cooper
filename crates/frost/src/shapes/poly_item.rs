use glam::Vec3;

use super::poly_primitives::WrappedPrimitiveId;
#[derive(Debug, Clone)]
pub struct PolygonPrimitive {
    pub vertices: [Vec3; 4],
    pub num_vertices: usize,
    pub vertex_ids: [WrappedPrimitiveId; 4],
    pub edge_ids: [WrappedPrimitiveId; 4],
    pub face_id: WrappedPrimitiveId,
}

impl Default for PolygonPrimitive {
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

impl PolygonPrimitive {
    pub fn new() -> Self{
        Self::default()
    }
    pub fn transorm(&mut self, pos: &Vec3) {
        for point in &mut self.vertices[0..self.num_vertices] {
            *point = *pos * (*point);
        }
    }
}