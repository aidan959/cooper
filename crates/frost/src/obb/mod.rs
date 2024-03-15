use glam::{Mat3, Quat, Vec3};

use crate::Transform;

pub trait OBB {
    fn is_colliding(&self, other: &dyn OBB) -> bool;
    fn get_collision_point_normal(&self, other: &dyn OBB) -> Option<CollisionPoint>;
    fn center(&self) -> Vec3;
    fn half_extents(&self) -> Vec3;
    fn orientation(&self) -> Quat;

    fn get_axes(&self) -> [Vec3; 3] {
        let mat = Mat3::from_quat(self.orientation());
        [mat.x_axis, mat.y_axis, mat.z_axis]
    }
    fn project_to_axis(&self, axis: Vec3) -> (f32, f32) {
        let corners = self.get_corners();

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
    fn get_corners(&self) -> [Vec3; 8] {
        let rot_mat = Mat3::from_quat(self.orientation());
        let mut vertices = [Vec3::ZERO; 8];
        let he = self.half_extents();

        for i in 0..8 {
            let corner = Vec3::new(
                if i & 1 == 0 { -he.x } else { he.x },
                if i & 2 == 0 { -he.y } else { he.y },
                if i & 4 == 0 { -he.z } else { he.z },
            );
            vertices[i] = self.center() + rot_mat * corner;
            
        }

        vertices
    }
    fn check_overlap(&self, axis: Vec3, obb2: &dyn OBB) -> bool {
        let (min1, max1) = self.project_to_axis(axis);
        let (min2, max2) = obb2.project_to_axis(axis);
        min1 <= max2 && max1 >= min2
    }
    fn get_collision_axes(&self, obb2: &dyn OBB) -> Vec<Vec3> {
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
    fn get_overlap_pen_depth(&self, obb2: &dyn OBB, norm: Vec3) -> (bool, f32) {
        let mut obb1_min = f32::INFINITY;
        let mut obb1_max = f32::NEG_INFINITY;
        let mut obb2_min = f32::INFINITY;
        let mut obb2_max = f32::NEG_INFINITY;
        for corner in self.get_corners().iter() {
            let projection = corner.dot(norm);
            obb1_min = obb1_min.min(projection);
            obb1_max = obb1_max.max(projection);
        }
        for corner in obb2.get_corners().iter() {
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
    fn get_support_point(&self, norm: Vec3) -> Vec3 {
        let mut support_point = self.get_corners()[0];
        let mut max_proj = support_point.dot(norm);

        for corner in self.get_corners().iter() {
            let projection = corner.dot(norm);
            if projection > max_proj {
                max_proj = projection;
                support_point = *corner;
            }
        }
        support_point
    }
}

pub struct DynamicOBB {
    pub center: Vec3,
    pub half_extents: Vec3,
    pub orientation: Quat,
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
        Self {
            center,
            half_extents,
            orientation,
        }
    }
}
impl OBB for DynamicOBB {
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
    fn is_colliding(&self, obb2: &dyn OBB) -> bool {
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
    fn get_collision_point_normal(&self, obb2: &dyn OBB) -> Option<CollisionPoint> {
        if !self.is_colliding(obb2) {
            return None;
        }
        let mut min_pen_depth = f32::INFINITY;

        let mut collision_point = CollisionPoint {
            point: Vec3::ZERO,
            normal: Vec3::ZERO,
            pen_depth: 0.0,
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

        let collision_point1 = self.get_support_point(-collision_point.normal);
        let collision_point2 = obb2.get_support_point(collision_point.normal);

        collision_point.point = (collision_point1 + collision_point2) * 0.5;

        Some(collision_point)
    }
}
impl Default for DynamicOBB {
    fn default() -> Self {
        Self {
            center: Vec3::ZERO,
            half_extents: Vec3::ZERO,
            orientation: Quat::IDENTITY,
        }
    }
}
#[cfg(test)]
mod test {
    use super::OBB;
    use glam::{Quat, Vec3};
    #[test]
    fn is_colliding() {
        let box1 = OBB::new(
            Vec3::new(0., 0., 0.),
            Vec3::new(0.5, 0.5, 0.5),
            Quat::IDENTITY,
        );
        let mut box3 = OBB::new(
            Vec3::new(0., 0.49, 0.0),
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
