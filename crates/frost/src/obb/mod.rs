use glam::{Mat3, Quat, Vec3};

use crate::{shapes::{self, PolygonPrimitive, PrimitiveId, WrappedPrimitiveId}, Transform};

pub trait OBB {
    fn is_colliding(&self, other: &DynamicOBB) -> bool;
    fn get_collision_point_normal(&self, other: &DynamicOBB) -> Option<CollisionPoint>;
    fn center(&self) -> Vec3;
    fn half_extents(&self) -> Vec3;
    fn orientation(&self) -> Quat;
    fn update_faces(&mut self);
    fn get_axes(&self) -> [Vec3; 3] {
        let mat = Mat3::from_quat(self.orientation());
        [mat.x_axis, mat.y_axis, mat.z_axis]
    }
    fn project_to_axis(&self, axis: Vec3) -> (f32, f32) {
        let corners = self.get_vertices();

        let mut min = f32::INFINITY;
        let mut max = f32::NEG_INFINITY;

        for corner in corners.iter() {
            let projection = corner.dot(axis);
            if projection < min {
                min = projection;
            }
            if projection > max {
                max = projection;
            }
        }

        (min, max)
    }
    fn initialize_corners(&mut self);
    fn update_vertices(&mut self);
    fn get_vertices(&self) -> [Vec3; 8] {
        let rot_mat = Mat3::from_quat(self.orientation());
        let he = self.half_extents();
        let center = self.center();
        [
            center + rot_mat * Vec3::new(-he.x, -he.y, -he.z),
            center + rot_mat * Vec3::new(he.x, -he.y, -he.z),
            center + rot_mat * Vec3::new(he.x, he.y, -he.z),
            center + rot_mat * Vec3::new(-he.x, he.y, -he.z),
            center + rot_mat * Vec3::new(-he.x, -he.y, he.z),
            center + rot_mat * Vec3::new(he.x, -he.y, he.z),
            center + rot_mat * Vec3::new(he.x, he.y, he.z),
            center + rot_mat * Vec3::new(-he.x, he.y, he.z),
        ]
    }
    fn get_faces(&self) -> [PolygonPrimitive; 6];
    fn get_edges(&self) -> [(Vec3, Vec3); 12];
    fn initialize_faces() -> [PolygonPrimitive; 6];
    
    fn check_overlap(&self, axis: Vec3, obb2: &DynamicOBB) -> bool {
        let (min1, max1) = self.project_to_axis(axis);
        let (min2, max2) = obb2.project_to_axis(axis);
        min1 <= max2 && max1 >= min2
    }
    fn get_collision_axes(&self, obb2: &DynamicOBB) -> Vec<Vec3> {
        let axes1 = self.get_axes();
        let axes2 = obb2.get_axes();
        let mut collision_axes = Vec::new();
        // Add the axes from both OBBs
        collision_axes.extend(axes1.iter().cloned());
        collision_axes.extend(axes2.iter().cloned());

        // Add the cross products of all combinations of axes from both OBBs
        for axis1 in axes1.iter() {
            for axis2 in axes2.iter() {
                let axis = axis1.cross(*axis2);
                if axis.length_squared() > 1e-6 {
                    // Check for non-zero length
                    collision_axes.push(axis.normalize()); // Ensure the axis is normalized
                }
            }
        }
        collision_axes
    }
    fn get_overlap_pen_depth(&self, obb2: &DynamicOBB, norm: Vec3) -> (bool, f32) {
        let mut obb1_min = f32::INFINITY;
        let mut obb1_max = f32::NEG_INFINITY;
        let mut obb2_min = f32::INFINITY;
        let mut obb2_max = f32::NEG_INFINITY;
        for corner in self.get_vertices().iter() {
            let projection = corner.dot(norm);
            obb1_min = obb1_min.min(projection);
            obb1_max = obb1_max.max(projection);
        }
        for corner in obb2.get_vertices().iter() {
            let projection = corner.dot(norm);
            obb2_min = obb2_min.min(projection);
            obb2_max = obb2_max.max(projection);
        }
        if obb1_max < obb2_min || obb2_max < obb1_min {
            return (false, 0.0);
        }
        let depth1 = obb2_max - obb1_min;
        let depth2 = obb1_max - obb2_min;
        let pen_depth = depth1.min(depth2);
        (true, pen_depth)
    }
    fn get_support_point(&self, norm: Vec3) -> Vec3 { // TODO MAKE THIS MUCH BETTER :)
        let mut support_point = self.get_vertices()[0];
        let mut max_proj = support_point.dot(norm);

        for corner in self.get_vertices().iter() {
            let projection = corner.dot(norm);
            if projection > max_proj {
                max_proj = projection;
                support_point = *corner;
            }
        }
        support_point
    }
    fn get_support_primitive(&self, direction: Vec3) -> WrappedPrimitiveId;
    fn get_support_vertex(&self, direction: Vec3) -> WrappedPrimitiveId;

    fn get_support_edge(&self, direction: Vec3) -> WrappedPrimitiveId;

    fn get_support_face(&self, direction: Vec3) -> WrappedPrimitiveId;
    
}

pub struct DynamicOBB {
    pub center: Vec3,
    pub half_extents: Vec3,
    pub orientation: Quat,
    vertices: [Vec3; 8],
    faces: [PolygonPrimitive; 6],
}

pub struct StaticOBB {
    pub center: Vec3,
    pub half_extents: Vec3,
    pub orientation: Quat,
}

impl StaticOBB {
    pub fn new(center: Vec3, half_extents: Vec3, orientation: Quat) -> Self {
        Self {
            center,
            half_extents,
            orientation,
        }
    }
}
pub struct CollisionPoint {
    pub point: Vec3,
    pub normal: Vec3,
    pub pen_depth: f32,
    primitive_a: Option<WrappedPrimitiveId>,
    primitive_b: Option<WrappedPrimitiveId>,
}

impl CollisionPoint {
    pub fn new() -> Self {
        Self {
            point: Vec3::ZERO,
            normal: Vec3::ZERO,
            pen_depth: 0.0,
            primitive_a: None,
            primitive_b: None,
        }
    }

    pub fn get_primitive_a(&self) -> WrappedPrimitiveId {
        return self.primitive_a.unwrap();
    }
    pub fn get_primitive_b(&self) -> WrappedPrimitiveId {
        return self.primitive_b.unwrap();
    }
}
fn cross_product_axes(axes1: &[Vec3; 3], axes2: &[Vec3; 3]) -> Vec<Vec3> {
    let mut cross_axes = Vec::new();
    for &axis1 in axes1.iter() {
        for &axis2 in axes2.iter() {
            let cross_axis = axis1.cross(axis2);
            if cross_axis.length_squared() > f32::EPSILON {
                cross_axes.push(cross_axis.normalize());
            }
        }
    }
    cross_axes
}
impl DynamicOBB {
    pub fn new(center: Vec3, half_extents: Vec3, orientation: Quat) -> Self {
        let vertices = Self::create_vertices(center, half_extents, orientation);
        let faces : [PolygonPrimitive; 6] = Self::initialize_faces();
        
        Self {
            center,
            half_extents,
            orientation,
            vertices: vertices,
            faces: faces,
        }
    }
    #[inline]
    fn center(&self) -> Vec3 {
        self.center
    }
    fn half_extents(&self) -> Vec3 {
        self.half_extents
    }
    fn orientation(&self) -> Quat {
        self.orientation
    }
    fn get_faces(&self) -> [PolygonPrimitive; 6] {
        self.faces
    }
    fn initialize_corners(&mut self) {
        self.vertices = self.get_vertices();
    }
    fn create_vertices(center: Vec3, he:Vec3, rotation: Quat) -> [Vec3; 8] {
        let rot_mat = Mat3::from_quat(rotation);
        [
            center + rot_mat * Vec3::new(-he.x, -he.y, -he.z), // 0 0 0 
            center + rot_mat * Vec3::new(he.x, -he.y, -he.z), // 1 0 0 
            center + rot_mat * Vec3::new(-he.x, he.y, -he.z), // 0 1 0
            center + rot_mat * Vec3::new(he.x, he.y, -he.z), // 1 1 0
            center + rot_mat * Vec3::new(-he.x, -he.y, he.z), // 0 0 1
            center + rot_mat * Vec3::new(he.x, -he.y, he.z), // 1 0 1
            center + rot_mat * Vec3::new(-he.x, he.y, he.z), // 0 1 1
            center + rot_mat * Vec3::new(he.x, he.y, he.z),] // 1 1 1  
    }
    fn get_vertices(&self) -> [Vec3; 8] {
        let rot_mat = Mat3::from_quat(self.orientation());
        let he = self.half_extents();
        let center = self.center;
        [
            center + rot_mat * Vec3::new(-he.x, -he.y, -he.z),
            center + rot_mat * Vec3::new(he.x, -he.y, -he.z),
            center + rot_mat * Vec3::new(he.x, he.y, -he.z),
            center + rot_mat * Vec3::new(-he.x, he.y, -he.z),
            center + rot_mat * Vec3::new(-he.x, -he.y, he.z),
            center + rot_mat * Vec3::new(he.x, -he.y, he.z),
            center + rot_mat * Vec3::new(he.x, he.y, he.z),
            center + rot_mat * Vec3::new(-he.x, he.y, he.z),
        ]
    }

    pub fn primitive_normal(&self, primitive: PrimitiveId) -> Option<Vec3> {
        match primitive {
            PrimitiveId::Face(face_id) => {
                let mut dir: Vec3 = Vec3::ZERO;

                if face_id < 3 {
                    dir[face_id as usize] = 1.0;
                } else {
                    dir[face_id as usize - 3] = -1.0;
                }
                Some(dir)
            }
            PrimitiveId::Edge(edge_id) => {
                let edge = edge_id & 0b011;
                let face1 = (edge + 1) % 3;
                let face2 = (edge + 2) % 3;
                let signs = edge_id >> 2;

                let mut dir= Vec3::ZERO;
                let _1: f32 = 1.0;

                if signs & (1 << face1) != 0 {
                    dir[face1 as usize] = -_1
                } else {
                    dir[face1 as usize] = _1
                }

                if signs & (1 << face2) != 0 {
                    dir[face2 as usize] = -_1
                } else {
                    dir[face2 as usize] = _1;
                }

                Some(dir.normalize())
            }
            PrimitiveId::Vertex(vertex_id) => {
                let mut dir = Vec3::ZERO;
                for i in 0..3 {
                    let _1: f32 = 1.0;

                    if vertex_id & (1 << i) != 0 {
                        dir[i] = -_1;
                    } else {
                        dir[i] = _1
                    }
                }

                Some(dir.normalize())
            }
            _ => None,
        }
    }
    // fn get_support_primitive(&self, direction: Vec3) -> PrimitiveId {
    //     let vertex_id = self.get_support_vertex(direction);
    //     let edge = self.get_support_edge(direction);
    //     let face = self.get_support_face(direction);

    //     if let Some(e) = edge {
    //         if self.vertices[e.0 as usize].dot(direction) > self.vertices[vertex_id as usize].dot(direction) {
    //             return PrimitiveId::Edge(e.0);
    //         }
    //     }

    //     if let Some(f) = face {
    //         if (self.vertices[f[0] as usize].dot(direction) > self.vertices[vertex_id as usize].dot(direction)) &&
    //            (self.vertices[f[0] as usize].dot(direction) > self.vertices[edge.unwrap().0 as usize].dot(direction)) {
    //             return PrimitiveId::Face(0);
    //         }
    //     }

    //     PrimitiveId::Vertex(vertex_id)
    // }
    // fn get_edges(&self) -> [(Vec3, Vec3); 12] {
    //     let corners = self.vertices;
    //     [
    //         (corners[0], corners[1]),
    //         (corners[1], corners[2]),
    //         (corners[2], corners[3]),
    //         (corners[3], corners[0]),
    //         (corners[4], corners[5]),
    //         (corners[5], corners[6]),
    //         (corners[6], corners[7]),
    //         (corners[7], corners[4]),
    //         (corners[0], corners[4]),
    //         (corners[1], corners[5]),
    //         (corners[2], corners[6]),
    //         (corners[3], corners[7]),
    //     ]
    // }

    fn initialize_faces() ->  [PolygonPrimitive; 6] {
        let mut faces : [PolygonPrimitive; 6] = [PolygonPrimitive::new(); 6];
        let face_vertex_indices = [
            [0, 1, 2, 3], // front
            [4, 5, 6, 7], // back
            [2, 3, 7, 6], // top
            [0, 1, 4, 5], // bottom
            [0, 2, 4, 6], // left
            [1, 3, 5, 7], // right
        ];

        for (i, face_index) in face_vertex_indices.iter().enumerate() {
            let face = &mut faces[i];
            face.face_id = WrappedPrimitiveId::from(PrimitiveId::Face(i as u32));
            face.num_vertices = 4;
            face.vertex_ids = [
                WrappedPrimitiveId::vertex(face_index[0]),
                WrappedPrimitiveId::vertex(face_index[1]),
                WrappedPrimitiveId::vertex(face_index[2]),
                WrappedPrimitiveId::vertex(face_index[3]),
            ];
            face.edge_ids = [

                WrappedPrimitiveId::edge(i as u32),
                WrappedPrimitiveId::edge((i + 1) as u32 % 4),
                WrappedPrimitiveId::edge((i + 2) as u32 % 4),
                WrappedPrimitiveId::edge((i + 3) as u32 % 4),
            ];
        }; 
        println!("Initialized faces");
        faces
        
    }
    fn is_colliding(&self, obb2: &DynamicOBB) -> bool {
        let axes1 = self.get_axes();
        let axes2 = obb2.get_axes();
        for axis in axes1.iter() {
            if !self.check_overlap(*axis, obb2) {
                return false;
            }
        }
        for axis in axes2.iter() {
            if !self.check_overlap(*axis, obb2) {
                return false;
            }
        }
        let cross_axes = cross_product_axes(&axes1, &axes2);
        for axis in cross_axes.iter() {
            if !self.check_overlap(*axis, obb2) {
                return false;
            }
        }

        true
    }
    pub fn sutherland_hodgman_clip(&self, obb2: &DynamicOBB) -> () {

    }
    pub fn get_collision_point_normal(&self, obb2: &DynamicOBB) -> Option<CollisionPoint> {
        if !self.is_colliding(obb2) {
            return None;
        }
        let mut min_pen_depth = f32::INFINITY;

        let mut collision_point = CollisionPoint {
            point: Vec3::ZERO,
            normal: Vec3::ZERO,
            pen_depth: 0.0,
            primitive_a: None,
            primitive_b: None,
        };

        let axes: Vec<Vec3> = self.get_collision_axes(obb2);

        for axis in axes {
            if axis.length() < 0.0001 {
                continue;
            }
            let norm = axis.normalize();
            let (overlap, pen_dept) = self.get_overlap_pen_depth(obb2, norm);
            if !overlap {
                return None;
            }
            if pen_dept < min_pen_depth {
                min_pen_depth = pen_dept;
                collision_point.normal = norm;
            }
            
        }
        collision_point.pen_depth = min_pen_depth;
        collision_point.primitive_a = Some(WrappedPrimitiveId::from(PrimitiveId::Face(0)));
        collision_point.primitive_b = Some(WrappedPrimitiveId::from(PrimitiveId::Face(0)));

        let collision_point1 = self.get_support_point(-collision_point.normal);
        let collision_point2 = obb2.get_support_point(collision_point.normal);

        collision_point.point = (collision_point1 + collision_point2) * 0.5;

        Some(collision_point)
    }
    fn get_support_vertex(&self, direction: Vec3) -> WrappedPrimitiveId {
        WrappedPrimitiveId::vertex(self.get_vertices().iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.dot(direction).partial_cmp(&b.dot(direction)).unwrap())
            .map(|(index, _)| index as u32)
            .unwrap_or(0)) // Fallback to the first vertex if none found, should not happen
    }
    fn get_support_edge(&self, direction: Vec3) -> WrappedPrimitiveId {
        todo!();
        // self.get_edges()
        //     .iter()
        //     .max_by(|a, b| {
        //         let edge_a_dir = self.vertices[a.1 as usize] - self.vertices[a.0 as usize];
        //         let edge_b_dir = self.vertices[b.1 as usize] - self.vertices[b.0 as usize];
        //         edge_a_dir.dot(direction).partial_cmp(&edge_b_dir.dot(direction)).unwrap()
        //     })
        //     .copied();

    }
    fn get_support_face(&self, direction: Vec3) -> WrappedPrimitiveId {
        todo!();

        // self.faces.iter()
        //     .max_by(|a, b| {
        //         // Assuming the first three vertices form a valid face normal
        //         let normal_a = (self.vertices[a[1] as usize] - self.vertices[a[0] as usize])
        //                         .cross(self.vertices[a[2] as usize] - self.vertices[a[0] as usize]);
        //         let normal_b = (self.vertices[b[1] as usize] - self.vertices[b[0] as usize])
        //                         .cross(self.vertices[b[2] as usize] - self.vertices[b[0] as usize]);

        //         normal_a.dot(direction).partial_cmp(&normal_b.dot(direction)).unwrap()
        //     })
        //     .cloned();

    }
    
    pub fn update_faces(&mut self) {
        todo!()
    }
    
    fn get_edges(&self) -> [(PrimitiveId, PrimitiveId); 12] {
        todo!()
    }
    
    fn get_support_primitive(&self, direction: Vec3) -> WrappedPrimitiveId {
        todo!()
    }
    
    fn get_axes(&self) -> [Vec3; 3] {
        let mat = Mat3::from_quat(self.orientation());
        [mat.x_axis, mat.y_axis, mat.z_axis]
    }
    
    fn project_to_axis(&self, axis: Vec3) -> (f32, f32) {
        let corners = self.get_vertices();
    
        let mut min = f32::INFINITY;
        let mut max = f32::NEG_INFINITY;
    
        for corner in corners.iter() {
            let projection = corner.dot(axis);
            if projection < min {
                min = projection;
            }
            if projection > max {
                max = projection;
            }
        }
    
        (min, max)
    }
    
    fn check_overlap(&self, axis: Vec3, obb2: &DynamicOBB) -> bool {
        let (min1, max1) = self.project_to_axis(axis);
        let (min2, max2) = obb2.project_to_axis(axis);
        min1 <= max2 && max1 >= min2
    }
    
    fn get_collision_axes(&self, obb2: &DynamicOBB) -> Vec<Vec3> {
        let axes1 = self.get_axes();
        let axes2 = obb2.get_axes();
        let mut collision_axes = Vec::new();
        // Add the axes from both OBBs
        collision_axes.extend(axes1.iter().cloned());
        collision_axes.extend(axes2.iter().cloned());
    
        // Add the cross products of all combinations of axes from both OBBs
        for axis1 in axes1.iter() {
            for axis2 in axes2.iter() {
                let axis = axis1.cross(*axis2);
                if axis.length_squared() > 1e-6 {
                    // Check for non-zero length
                    collision_axes.push(axis.normalize()); // Ensure the axis is normalized
                }
            }
        }
        collision_axes
    }
    
    fn get_overlap_pen_depth(&self, obb2: &DynamicOBB, norm: Vec3) -> (bool, f32) {
        let mut obb1_min = f32::INFINITY;
        let mut obb1_max = f32::NEG_INFINITY;
        let mut obb2_min = f32::INFINITY;
        let mut obb2_max = f32::NEG_INFINITY;
        for corner in self.get_vertices().iter() {
            let projection = corner.dot(norm);
            obb1_min = obb1_min.min(projection);
            obb1_max = obb1_max.max(projection);
        }
        for corner in obb2.get_vertices().iter() {
            let projection = corner.dot(norm);
            obb2_min = obb2_min.min(projection);
            obb2_max = obb2_max.max(projection);
        }
        if obb1_max < obb2_min || obb2_max < obb1_min {
            return (false, 0.0);
        }
        let depth1 = obb2_max - obb1_min;
        let depth2 = obb1_max - obb2_min;
        let pen_depth = depth1.min(depth2);
        (true, pen_depth)
    }
    
    fn get_support_point(&self, norm: Vec3) -> Vec3 { // TODO MAKE THIS MUCH BETTER :)
        let mut support_point = self.get_vertices()[0];
        let mut max_proj = support_point.dot(norm);
    
        for corner in self.get_vertices().iter() {
            let projection = corner.dot(norm);
            if projection > max_proj {
                max_proj = projection;
                support_point = *corner;
            }
        }
        support_point
    }
    
}

#[cfg(test)]
mod test {
    use super::{DynamicOBB, OBB};
    use glam::{Quat, Vec3};
    #[test]
    fn is_colliding() {
        let box1 = DynamicOBB::new(
            Vec3::new(0., 0., 0.),
            Vec3::new(0.5, 0.5, 0.5),
            Quat::IDENTITY,
        );
        let mut box3 = DynamicOBB::new(
            Vec3::new(0., -0.5, 0.0),
            Vec3::new(0.5, 0.5, 0.5),
            Quat::from_euler(glam::EulerRot::XYZ, 0., 0.0, 0.),
        );

        
        match box3.get_collision_point_normal(&box1) {
            Some(collision_point) => {
                println!("Collision Point: {:?}", collision_point.point);
                println!("Collision Normal: {:?}", collision_point.normal);
            }
            None => {}
        }
    }
}


