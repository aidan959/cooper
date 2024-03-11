use glam::{Vec3, Quat};

use crate::Transform;

struct OBB {
    center: Vec3,
    half_extents: Vec3,
    orientation: Quat,
}
struct CollisionPoint{
    point: Vec3,
    normal: Vec3,
}
impl OBB {
    pub fn new(center: Vec3, half_extents: Vec3, orientation: Quat) -> Self {
        Self {
            center,
            half_extents,
            orientation,
        }
    }
    fn transform(&self, transform: &Transform) -> OBB {
        let center = transform.position + transform.rotation * self.center;
        let orientation = transform.rotation * self.orientation;
        OBB {
            center,
            half_extents: self.half_extents,
            orientation,
        }
    }
    fn get_axes(&self) -> [Vec3; 3] {
        let x = self.orientation * Vec3::X;
        let y = self.orientation * Vec3::Y;
        let z = self.orientation * Vec3::Z;
        [x, y, z]
    }
    fn get_corners(&self) -> [Vec3; 8] {
        let axes = self.get_axes();
        let x = axes[0];
        let y = axes[1];
        let z = axes[2];
        let x = x * self.half_extents.x;
        let y = y * self.half_extents.y;
        let z = z * self.half_extents.z;
        let center = self.center;
        [
            center + x + y + z,
            center + x + y - z,
            center + x - y + z,
            center + x - y - z,
            center - x + y + z,
            center - x + y - z,
            center - x - y + z,
            center - x - y - z,
        ]
    }
    fn check_overlap(&self, axis: Vec3, obb2: &OBB) -> bool {
        let mut obb1_min = f32::INFINITY;
        let mut obb1_max = f32::NEG_INFINITY;
        let mut obb2_min = f32::INFINITY;
        let mut obb2_max = f32::NEG_INFINITY;
        for corner in self.get_corners().iter() {
            let projection = corner.dot(axis);
            obb1_min = obb1_min.min(projection);
            obb1_max = obb1_max.max(projection);
        }
        for corner in obb2.get_corners().iter() {
            let projection = corner.dot(axis);
            obb2_min = obb2_min.min(projection);
            obb2_max = obb2_max.max(projection);
        }
        let obb1_overlap = obb1_min <= obb2_max && obb1_max >= obb2_min;
        let obb2_overlap = obb2_min <= obb1_max && obb2_max >= obb1_min;
        obb1_overlap && obb2_overlap
    }
    fn is_colliding(&self, obb2: &OBB) -> bool {
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
        for axis1 in axes1.iter() {
            for axis2 in axes2.iter() {
                let axis = axis1.cross(*axis2);
                if !self.check_overlap(axis, obb2) {
                    return false;
                }
            }
        }
        true
    }
    fn get_collision_point_normal(&self, obb2: &OBB) -> Option<CollisionPoint>{
        if !self.is_colliding(obb2) {
            return None;
        }
        let mut min_pen_depth = f32::INFINITY;
        
        let mut collision_point = CollisionPoint{
            point: Vec3::ZERO,
            normal: Vec3::ZERO,
        };
        
        let axes :Vec<Vec3> = self.get_collision_axes(obb2);

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

        let collision_point1 = self.get_support_point(collision_point.normal * -1.);
        let collision_point2 = obb2.get_support_point(collision_point.normal);

        collision_point.point = (collision_point1 + collision_point2) * 0.5;
        Some(collision_point)
    }
    
    fn get_collision_axes (&self, obb2: &OBB) -> Vec<Vec3> {
        let axes1 = self.get_axes();
        let axes2 = obb2.get_axes();
        let mut collision_axes = Vec::new();
        for axis in axes1.iter() {
            collision_axes.push(*axis);
        }
        for axis in axes2.iter() {
            collision_axes.push(*axis);
        }
        for axis1 in axes1.iter() {
            for axis2 in axes2.iter() {
                let axis = axis1.cross(*axis2);
                collision_axes.push(axis);
            }
        }
        collision_axes
    }
    fn get_overlap_pen_depth(&self, obb2: &OBB, norm: Vec3) -> (bool, f32) {
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
        let obb1_overlap = obb1_min <= obb2_max && obb1_max >= obb2_min;
        let obb2_overlap = obb2_min <= obb1_max && obb2_max >= obb1_min;
        if !(obb1_overlap && obb2_overlap) {
            return (false, 0.0)   
        }
        let depth1 = obb2_max - obb1_min;
        let depth2 = obb1_max - obb2_min;
        let pen_depth = depth1.min(depth2);
        (true, pen_depth)
    }
    fn get_support_point(&self, norm: Vec3) -> Vec3 {
        let mut max_projection = f32::NEG_INFINITY;
        let mut support_point = Vec3::ZERO;
        for corner in self.get_corners().iter() {
            let projection = corner.dot(norm);
            if projection > max_projection {
                max_projection = projection;
                support_point = *corner;
            }
        }
        support_point
    }
}

#[cfg(test)]
mod test {
    use glam::{Vec3, Quat};
    use super::OBB;
    #[test]
    fn is_colliding() {
        let box1 = OBB::new(Vec3::new(0.,0.,0.), Vec3::new(0.5,0.5,0.5), Quat::from_rotation_x(45.));
        let box2 = OBB::new(Vec3::new(2.,0.,0.), Vec3::new(0.5,0.5,0.5), Quat::from_rotation_x(20.));
        let box3 = OBB::new(Vec3::new(0.5,0.,0.), Vec3::new(0.5,0.5,0.5), Quat::from_rotation_x(45.));
        assert_eq!(box1.is_colliding(&box2), false);
        assert_eq!(box1.is_colliding(&box3), true);
        match box1.get_collision_point_normal(&box2) {
            Some(collision_point) => {
                println!("Collision Point: {:?}", collision_point.point);
                println!("Collision Normal: {:?}", collision_point.normal);
            },
            None => println!("No collision"),
        }
        match box1.get_collision_point_normal(&box3) {
            Some(collision_point) => {
                println!("Collision Point: {:?}", collision_point.point);
                println!("Collision Normal: {:?}", collision_point.normal);
            },
            None => println!("No collision"),
        }
        


    }
}