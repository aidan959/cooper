use std::collections::HashMap;

use glam::{const_vec3, Mat3, Quat, Vec3};

use crate::{
    shapes::{PolygonPrimitive, PrimitiveId, WrappedPrimitiveId},
    Transform,
};

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
    fn retrieve_support_point(&self, norm: Vec3) -> Vec3 {
        // TODO MAKE THIS MUCH BETTER :)
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
    fn retrieve_support_primitive(&self, direction: Vec3) -> WrappedPrimitiveId;
    fn retrieve_support_vertex(&self, direction: Vec3) -> WrappedPrimitiveId;

    fn retrieve_support_edge(&self, direction: Vec3) -> WrappedPrimitiveId;

    fn retrieve_support_face(&self, direction: Vec3) -> WrappedPrimitiveId;
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
    primitive_a: WrappedPrimitiveId,
    primitive_b: WrappedPrimitiveId,
}

impl CollisionPoint {
    pub fn new() -> Self {
        Self {
            point: Vec3::ZERO,
            normal: Vec3::ZERO,
            pen_depth: 0.0,
            primitive_a: WrappedPrimitiveId::UNKNOWN,
            primitive_b: WrappedPrimitiveId::UNKNOWN,
        }
    }

    pub fn retrieve_primitive_a(&self) -> WrappedPrimitiveId {
        return self.primitive_a;
    }
    pub fn retrieve_primitive_b(&self) -> WrappedPrimitiveId {
        return self.primitive_b;
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
const EDGE_VERTEX_INDICES: [[u32; 2]; 12] = [
    [0, 1], // 0
    [1, 2], // 1
    [2, 3],
    [3, 0],
    [0, 4],
    [1, 5],
    [2, 6],
    [3, 7],
    [4, 5],
    [5, 6],
    [6, 7],
    [7, 4],
];
const FACE_VERTEX_INDICES: [[u32; 4]; 6] = [
    [0, 1, 2, 3], // front
    [4, 5, 6, 7], // back
    [3, 2, 6, 7], // top
    [0, 1, 5, 4], // bottom
    [4, 0, 3, 7], // left
    [1, 5, 6, 2], // right
];

const FACE_NORMALS: [Vec3; 6] = [
    const_vec3!([0., 0., -1.0]),   //front
    const_vec3!([0., 0., 1.0]),    //back
    const_vec3!([0., 1.0, 0.0]),   //top
    const_vec3!([0., -1.0, 0.0]),  //bottom
    const_vec3!([-1.0, 0.0, 0.0]), //left
    const_vec3!([1.0, 0.0, 0.0]),  //right
];
impl DynamicOBB {
    pub fn new(center: Vec3, half_extents: Vec3, orientation: Quat) -> Self {
        let vertices = Self::create_vertices(center, half_extents, orientation);
        let faces: [PolygonPrimitive; 6] = Self::initialize_faces();

        Self {
            center,
            half_extents,
            orientation,
            vertices: vertices,
            faces: faces,
        }
    }
    pub fn from_transform(transform: Transform) -> Self {
        Self::new(transform.position, transform.scale/2.0, transform.rotation)
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
    fn retrieve_faces(&self) -> [PolygonPrimitive; 6] {
        self.faces
    }
    fn initialize_corners(&mut self) {
        self.vertices = self.get_vertices();
    }
    fn create_vertices(center: Vec3, he: Vec3, rotation: Quat) -> [Vec3; 8] {
        let rot_mat = Mat3::from_quat(rotation);
        [
            center + rot_mat * Vec3::new(-he.x, -he.y, -he.z), // 0 0 0
            center + rot_mat * Vec3::new(he.x, -he.y, -he.z),  // 1 0 0
            center + rot_mat * Vec3::new(-he.x, he.y, -he.z),  // 0 1 0
            center + rot_mat * Vec3::new(he.x, he.y, -he.z),   // 1 1 0
            center + rot_mat * Vec3::new(-he.x, -he.y, he.z),  // 0 0 1
            center + rot_mat * Vec3::new(he.x, -he.y, he.z),   // 1 0 1
            center + rot_mat * Vec3::new(-he.x, he.y, he.z),   // 0 1 1
            center + rot_mat * Vec3::new(he.x, he.y, he.z),
        ] // 1 1 1
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
    pub fn update_vertices(&mut self) {
        self.vertices[0] = self.center + self.orientation * Vec3::new(-self.half_extents.x, -self.half_extents.y, -self.half_extents.z);
        self.vertices[1] = self.center + self.orientation * Vec3::new(self.half_extents.x, -self.half_extents.y, -self.half_extents.z);
        self.vertices[2] = self.center + self.orientation * Vec3::new(-self.half_extents.x, self.half_extents.y, -self.half_extents.z);
        self.vertices[3] = self.center + self.orientation * Vec3::new(self.half_extents.x, self.half_extents.y, -self.half_extents.z);
        self.vertices[4] = self.center + self.orientation * Vec3::new(-self.half_extents.x, -self.half_extents.y, self.half_extents.z);
        self.vertices[5] = self.center + self.orientation * Vec3::new(self.half_extents.x, -self.half_extents.y, self.half_extents.z);
        self.vertices[6] = self.center + self.orientation * Vec3::new(-self.half_extents.x, self.half_extents.y, self.half_extents.z);
        self.vertices[7] = self.center + self.orientation * Vec3::new(self.half_extents.x, self.half_extents.y, self.half_extents.z);
    }
    // Adjust the method to get the face normal based on the OBB's orientation
    fn get_face_normal(&self, face_index: usize) -> Vec3 {
        // Retrieve the normal and rotate it according to the OBB's orientation
        let normal = FACE_NORMALS[face_index];
        self.orientation * normal
    }
    pub fn find_face_with_normal(&self, given_normal: &Vec3) -> Option<PolygonPrimitive> {
        let normalized_given_normal = given_normal.normalize();
        let mut max_dot = f32::MIN;
        let mut closest_face = None;

        for (i, face) in self.faces.iter().enumerate() {
            // Adjust the normal based on the OBB's orientation
            let face_normal = self.get_face_normal(i);

            let dot = face_normal.dot(normalized_given_normal);

            if dot > max_dot {
                max_dot = dot;
                closest_face = Some(*face);
            }
        }

        closest_face
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

                let mut dir = Vec3::ZERO;
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
    // fn retrieve_support_primitive(&self, direction: Vec3) -> PrimitiveId {
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
    // fn retrieve_edges(&self) -> [(Vec3, Vec3); 12] {
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
    fn retrieve_face(&self, face_id: WrappedPrimitiveId) -> PolygonPrimitive {
        self.faces[face_id.unpack().face().unwrap() as usize]
    }
    fn retrieve_vertex(&self, vertex_id: WrappedPrimitiveId) -> &Vec3 {
        &self.vertices[vertex_id.unpack().vertex().unwrap() as usize]
    }
    fn initialize_faces() -> [PolygonPrimitive; 6] {
        let mut faces: [PolygonPrimitive; 6] = [PolygonPrimitive::new(); 6];

        // silly solution to consistently id edges
        let mut edge_vertex_map: HashMap<[u32; 2], u32> = HashMap::new();

        for (id, edge) in EDGE_VERTEX_INDICES.iter().enumerate() {
            edge_vertex_map.insert(edge.clone(), id as u32);
            let mut edge = edge.clone();
            edge.reverse();
            edge_vertex_map.insert(edge, id as u32);
        }

        for (i, face_index) in FACE_VERTEX_INDICES.iter().enumerate() {
            let face = &mut faces[i];
            face.face_id = WrappedPrimitiveId::from(PrimitiveId::Face(i as u32));
            face.num_vertices = 4;
            face.vertex_ids = [
                WrappedPrimitiveId::vertex(face_index[0]),
                WrappedPrimitiveId::vertex(face_index[1]),
                WrappedPrimitiveId::vertex(face_index[2]),
                WrappedPrimitiveId::vertex(face_index[3]),
            ];

            let mut pairs: [[u32; 2]; 4] = [[0; 2]; 4];
            pairs.iter_mut().enumerate().for_each(|(j, pair)| {
                *pair = [face_index[j], face_index[(j + 1) % face_index.len()]];
            });

            face.edge_ids = [
                WrappedPrimitiveId::edge(*edge_vertex_map.get(&pairs[0]).unwrap()),
                WrappedPrimitiveId::edge(*edge_vertex_map.get(&pairs[1]).unwrap()),
                WrappedPrimitiveId::edge(*edge_vertex_map.get(&pairs[2]).unwrap()),
                WrappedPrimitiveId::edge(*edge_vertex_map.get(&pairs[3]).unwrap()),
            ];
        }

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

    pub fn sutherland_hodgman_clip(&self, obb2: &DynamicOBB) -> () {}
    pub fn get_collision_point_normal(&self, obb2: &DynamicOBB) -> Option<CollisionPoint> {
        if !self.is_colliding(obb2) {
            return None;
        }
        let mut min_pen_depth = f32::INFINITY;

        let mut collision_point = CollisionPoint {
            point: Vec3::ZERO,
            normal: Vec3::ZERO,
            pen_depth: 0.0,
            primitive_a: WrappedPrimitiveId::UNKNOWN,
            primitive_b: WrappedPrimitiveId::UNKNOWN,
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
        fn get_face_name (face_id: PrimitiveId) -> &'static str {
            match face_id {
                PrimitiveId::Face(0) => "Front",
                PrimitiveId::Face(1) => "Back",
                PrimitiveId::Face(2) => "Top",
                PrimitiveId::Face(3) => "Bottom",
                PrimitiveId::Face(4) => "Left",
                PrimitiveId::Face(5) => "Right",
                _ => "Unknown"
            }
        }
        let colliding_face_obb1 = self.find_face_with_normal(&collision_point.normal).unwrap();
        let obb2_collision_normal = -collision_point.normal;
        let colliding_face_obb2 = obb2.find_face_with_normal(&-collision_point.normal).unwrap();
        collision_point.pen_depth = min_pen_depth;
        collision_point.primitive_a = colliding_face_obb1.face_id;
        collision_point.primitive_b = colliding_face_obb2.face_id;
        // println!(
        //     "Colliding Face OBB1: {}",
        //     get_face_name(colliding_face_obb1.face_id.unpack())
        // );
        // println!(
        //     "Colliding Face OBB2: {}",
        //     get_face_name(colliding_face_obb2.face_id.unpack())
        // );
        //let collision_point1 = sel;
        //let collision_point2 = obb2.get_support_point(collision_point.normal);

        collision_point.point = obb2.center() - collision_point.normal * obb2.half_extents().dot(collision_point.normal) - self.center() + collision_point.normal * self.half_extents().dot(collision_point.normal);

        Some(collision_point)
    }
    fn get_support_vertex(&self, direction: Vec3) -> WrappedPrimitiveId {
        WrappedPrimitiveId::vertex(
            self.get_vertices()
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.dot(direction).partial_cmp(&b.dot(direction)).unwrap())
                .map(|(index, _)| index as u32)
                .unwrap_or(0),
        ) // Fallback to the first vertex if none found, should not happen
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

    fn get_edge_vertices(&self, edge_id: WrappedPrimitiveId) -> (&Vec3, &Vec3) {
        let edge_id = edge_id.unpack().edge().unwrap();
        let edge = EDGE_VERTEX_INDICES[edge_id as usize];
        (
            &self.vertices[edge[0] as usize],
            &self.vertices[edge[1] as usize],
        )
    }
    fn get_transformed_vertex(&self, index: usize) -> Vec3 {
        self.orientation * (self.vertices[index] - self.center) + self.center
    }
    fn get_vertex_pos(&self, vertex_id: WrappedPrimitiveId) -> &Vec3 {
        &self.vertices[vertex_id.unpack().vertex().unwrap() as usize]
    }

    fn find_face_on_normal(&self, norm: Vec3) -> Option<&PolygonPrimitive> {
        self.faces.iter().max_by(|face_a, face_b| {
            let dot_a =
                norm.dot(self.get_transformed_vertex(
                    face_a.vertex_ids[0].unpack().vertex().unwrap() as usize,
                ));
            let dot_b =
                norm.dot(self.get_transformed_vertex(
                    face_b.vertex_ids[0].unpack().vertex().unwrap() as usize,
                ));
            dot_a.partial_cmp(&dot_b).unwrap()
        })
    }
    fn get_support_primitive(&self, _direction: Vec3) -> WrappedPrimitiveId {
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

    fn get_collision_axes<'a>(&self, obb2: &'a DynamicOBB) -> Vec<Vec3> {
        let axes1 = self.get_axes();
        let axes2 = obb2.get_axes();
        let mut collision_axes: Vec<Vec3> = Vec::new();
        // Add the axes from both OBBs
        collision_axes.extend(axes1.iter().map(|x|*x).collect::<Vec<Vec3>>());
        collision_axes.extend(axes2.iter().map(|x|*x).collect::<Vec<Vec3>>());
         
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

    fn retrieve_support_point(&self, norm: Vec3) -> Vec3 {
        // TODO MAKE THIS MUCH BETTER :)
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
    use std::collections::HashMap;

    use crate::shapes::{PrimitiveId, WrappedPrimitiveId};

    use super::{DynamicOBB, OBB};
    use glam::{Quat, Vec3};
    #[test]
    fn is_colliding() {
        let box1 = DynamicOBB::new(
            Vec3::new(0., 0., 0.),
            Vec3::new(0.5, 0.5, 0.5),
            Quat::from_euler(glam::EulerRot::XYZ, 0., 0.0, 0.),
        );
        let box2 = DynamicOBB::new(
            Vec3::new(0., 0.9, 0.0),
            Vec3::new(0.5, 0.5, 0.5),
            Quat::from_euler(glam::EulerRot::XYZ, 0., 0.0, 0.),
        );

        match box1.get_collision_point_normal(&box2) {
            Some(collision_point) => {
                println!("Collision Point: {:?}", collision_point.point);
                println!("Collision Normal: {:?}", collision_point.normal);
            }
            None => {}
        }
    }
    fn retrieve_face_name (face_id: PrimitiveId) -> &'static str {
        match face_id {
            PrimitiveId::Face(0) => "Front",
            PrimitiveId::Face(1) => "Back",
            PrimitiveId::Face(2) => "Top",
            PrimitiveId::Face(3) => "Bottom",
            PrimitiveId::Face(4) => "Left",
            PrimitiveId::Face(5) => "Right",
            _ => "Unknown"
        }
    }
    #[test]
    fn find_face_on_normal() {
        let test_rotations = [
            Quat::from_euler(glam::EulerRot::XYZ, 0.0, 0.0, 0.0),
            Quat::from_euler(glam::EulerRot::XYZ, std::f32::consts::FRAC_PI_2/2., 0.0, 0.0),
            Quat::from_euler(glam::EulerRot::XYZ, 0.0, std::f32::consts::FRAC_PI_2/2., 0.0),
            Quat::from_euler(glam::EulerRot::XYZ, 0.0, 0.0, std::f32::consts::FRAC_PI_2/2.),
        ];
        
        let face_normals = [
            Vec3::new(0., 0., -1.0),   //front
            Vec3::new(0., 0., 1.0),    //back
            Vec3::new(0., 1.0, 0.0),   //top
            Vec3::new(0., -1.0, 0.0),  //bottom
            Vec3::new(-1.0, 0.0, 0.0), //left
            Vec3::new(1.0, 0.0, 0.0),  //right
        ];

        for rotation in test_rotations.iter() {
            let box1 = DynamicOBB::new(
                Vec3::new(0., 0., 0.),
                Vec3::new(0.5, 0.5, 0.5),
                *rotation,
            );
            
            let face_map = HashMap::from([
                (WrappedPrimitiveId::face(0), *rotation * face_normals[0]), //front
                (WrappedPrimitiveId::face(1), *rotation * face_normals[1]), //back
                (WrappedPrimitiveId::face(2), *rotation * face_normals[2]), //top
                (WrappedPrimitiveId::face(3), *rotation * face_normals[3]), //bottom
                (WrappedPrimitiveId::face(4), *rotation * face_normals[4]), //left
                (WrappedPrimitiveId::face(5), *rotation * face_normals[5]), //right
            ]);
            let euler = rotation.to_euler(glam::EulerRot::XYZ);
            println!("Rotation: ({}, {}, {})", euler.0 * 180.0 / std::f32::consts::PI, euler.1 * 180.0 / std::f32::consts::PI, euler.2 * 180.0 / std::f32::consts::PI);
            for (face_id, &rotated_normal) in face_map.iter() {
                let face = box1.find_face_with_normal(&rotated_normal).unwrap();
                println!("Face: {:?} -> normal ({}) ", get_face_name(face_id.unpack()), rotated_normal);
                assert_eq!(face_id, &face.face_id);
            }
        }
    }
    #[test]
    fn find_face_on_unaligned_normal() {
        let test_rotations = [
            Quat::from_euler(glam::EulerRot::XYZ, 0.0, 0.0, 0.0),
            Quat::from_euler(glam::EulerRot::XYZ, std::f32::consts::FRAC_PI_2/2., 0.0, 0.0),
            Quat::from_euler(glam::EulerRot::XYZ, 0.0, std::f32::consts::FRAC_PI_2/2., 0.0),
            Quat::from_euler(glam::EulerRot::XYZ, 0.0, 0.0, std::f32::consts::FRAC_PI_2/2.),
        ];
        
        let face_normals = [
            Vec3::new(0., 0.1, -0.9).normalize(),   //front
            Vec3::new(0., -0.05, 0.9).normalize(),    //back
            Vec3::new(0.1, 0.8, 0.1).normalize(),   //top
            Vec3::new(0.2, -0.5, 0.2).normalize(),  //bottom
            Vec3::new(-1.0, 0.0, 0.0).normalize(), //left
            Vec3::new(1.0, 0.0, 0.0).normalize(),  //right
        ];

        for rotation in test_rotations.iter() {
            let box1 = DynamicOBB::new(
                Vec3::new(0., 0., 0.),
                Vec3::new(0.5, 0.5, 0.5),
                *rotation,
            );
            
            let face_map = HashMap::from([
                (WrappedPrimitiveId::face(0), *rotation * face_normals[0]), //front
                (WrappedPrimitiveId::face(1), *rotation * face_normals[1]), //back
                (WrappedPrimitiveId::face(2), *rotation * face_normals[2]), //top
                (WrappedPrimitiveId::face(3), *rotation * face_normals[3]), //bottom
                (WrappedPrimitiveId::face(4), *rotation * face_normals[4]), //left
                (WrappedPrimitiveId::face(5), *rotation * face_normals[5]), //right
            ]);
            let euler = rotation.to_euler(glam::EulerRot::XYZ);
            println!("Rotation: ({}, {}, {})", euler.0 * 180.0 / std::f32::consts::PI, euler.1 * 180.0 / std::f32::consts::PI, euler.2 * 180.0 / std::f32::consts::PI);
            for (face_id, &rotated_normal) in face_map.iter() {
                let face = box1.find_face_with_normal(&rotated_normal).unwrap();
                println!("Face: {:?} -> normal ({}) ", get_face_name(face_id.unpack()), rotated_normal);
                assert_eq!(face_id, &face.face_id);
            }
        }
    }
}
