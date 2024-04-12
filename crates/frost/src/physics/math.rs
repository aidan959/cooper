use std::{cell::RefCell, default};

use glam::{Quat, Vec3,Mat3};

use self::obb::{DynamicOBB, OBB};

use super::{*,obb::CollisionPoint};

#[inline]
pub fn integrate_velocity(velocity: &mut Vec3, acceleration: Vec3, fixed_time: f32) {
    *velocity += acceleration * fixed_time;
}
#[inline]
pub fn integrate_position(transform: &mut Transform, velocity: Vec3, fixed_time: f32) {
    transform.position += velocity * fixed_time;
}
pub fn integrate_angular_velocity(angular_velocity: &mut Vec3, torque: Vec3, fixed_time: f32) {
    *angular_velocity += torque * fixed_time;
}
pub fn integrate_rotation(transform: &mut Transform, angular_velocity: Vec3, fixed_time: f32) {
    let rotation = Quat::from_euler( glam::EulerRot::XYZ ,angular_velocity.x, angular_velocity.y, angular_velocity.z);
    transform.rotation = rotation * transform.rotation * fixed_time;
}
pub fn integrate_rigid_body(rigid_body: &mut RigidBody, fixed_time: f32) {
    if rigid_body.gravity {
        rigid_body.acceleration += Vec3::new(0., -0.981, 0.) * fixed_time;
    }
    integrate_angular_velocity(&mut rigid_body.angular_velocity, Vec3::new(1., 0., 0.), fixed_time);
    integrate_velocity(&mut rigid_body.velocity, rigid_body.acceleration, fixed_time);
    integrate_position(&mut rigid_body.transform, rigid_body.velocity, fixed_time);
    rigid_body.clear_accumulators();
}

pub fn calculate_velocity_change(velocity: Vec3, acceleration: Vec3, fixed_time: f32) -> Vec3 {
    velocity + acceleration * fixed_time
}

pub fn calculate_position_change(position: Vec3, velocity: Vec3, fixed_time: f32) -> Vec3 {
    position + velocity * fixed_time
}
fn calculate_relative_velocity(rigid_body: &RigidBody, rigid_body2: &RigidBody, relative_p1: Vec3, relative_p2: Vec3) -> Vec3 {
    rigid_body.velocity + rigid_body.angular_velocity.cross(relative_p1) - 
    rigid_body2.velocity - rigid_body2.angular_velocity.cross(relative_p2)
}
pub fn handle_collision(rigid_body: &mut RigidBody, rigid_body2: &mut RigidBody, collision_point: &CollisionPoint) {


    if rigid_body.is_static && rigid_body2.is_static {
        return;
    }
    if rigid_body.is_static {
        handle_collision_static(rigid_body2, rigid_body, collision_point);
        return;
    }
    if rigid_body2.is_static {
        handle_collision_static(rigid_body, rigid_body2, collision_point);
        return;
    }
    let pen_depth = collision_point.pen_depth;
    let correction_ratio = 0.5; // Split the correction between the two bodies if both are dynamic
    let total_inverse_mass = rigid_body.inverse_mass + rigid_body2.inverse_mass;

    if total_inverse_mass > 0.0 {
        let correction = collision_point.normal * (pen_depth * correction_ratio / total_inverse_mass);

        if !rigid_body.is_static {
            rigid_body.transform.position -= correction * rigid_body.inverse_mass / total_inverse_mass;
        }
        if !rigid_body2.is_static {
            rigid_body2.transform.position += correction * rigid_body2.inverse_mass / total_inverse_mass;
        }
    }


    // get relative velocity at collision point
    let relative_p1 = collision_point.point - rigid_body.transform.position;
    let relative_p2 = collision_point.point - rigid_body2.transform.position;
    

    let relative_velocity = calculate_relative_velocity(&rigid_body, &rigid_body2, relative_p1, relative_p2);
    
    // do we use min or max here?
    let e = (rigid_body.restitution + rigid_body2.restitution) / 2.0;
    
    let r1_cross_n = relative_p1.cross(collision_point.normal);
    let r2_cross_n = relative_p2.cross(collision_point.normal);

    let inv_mass_sum = rigid_body.inverse_mass + rigid_body2.inverse_mass +
                    r1_cross_n.dot(rigid_body.inverse_inertia_tensor * r1_cross_n ) +
                    r2_cross_n.dot(rigid_body2.inverse_inertia_tensor * r2_cross_n );
    
    let j = (-(1.0 + e) * relative_velocity.dot(collision_point.normal)) / inv_mass_sum;

    let impulse = collision_point.normal * j; 
    
    if !rigid_body.is_static {
        rigid_body.velocity += impulse * rigid_body.inverse_mass;
        rigid_body.angular_velocity += rigid_body.inverse_inertia_tensor * r1_cross_n.cross(impulse);
    }
    if !rigid_body2.is_static {
        rigid_body2.velocity -= impulse * rigid_body2.inverse_mass;
        rigid_body2.angular_velocity -= rigid_body2.inverse_inertia_tensor * r2_cross_n.cross(impulse);
    }

}


pub fn handle_collision_static(rigid_body: &mut RigidBody, static_rigid_body: &mut RigidBody, collision_point: &CollisionPoint) {
    let pen_depth = collision_point.pen_depth;
    let correction_ratio = 1.0;
    let total_inverse_mass = rigid_body.inverse_mass;

    if total_inverse_mass > 0.0 {
        let correction = collision_point.normal * (pen_depth * correction_ratio / total_inverse_mass);

            rigid_body.transform.position -= correction  *10. * rigid_body.inverse_mass / total_inverse_mass ;
        
    }

    let relative_p1 = collision_point.point - rigid_body.transform.position;
    let relative_velocity = calculate_relative_velocity(&rigid_body, &static_rigid_body, relative_p1, Vec3::ZERO);
    let e = (rigid_body.restitution + static_rigid_body.restitution) / 2.0;
    let r1_cross_n = relative_p1.cross(collision_point.normal);
    let inv_mass_sum = rigid_body.inverse_mass + r1_cross_n.dot(rigid_body.inverse_inertia_tensor * r1_cross_n);
    let j = (-(1.0 + e) * relative_velocity.dot(collision_point.normal)) / inv_mass_sum;
    let impulse = collision_point.normal * j;

    rigid_body.velocity += impulse * rigid_body.inverse_mass;
    rigid_body.angular_velocity += rigid_body.inverse_inertia_tensor * r1_cross_n.cross(impulse);
    
}



pub struct RigidBody{
    pub inverse_mass: f32,
    pub transform: Transform,
    pub acceleration: Vec3,
    pub velocity: Vec3,
    pub angular_velocity:Vec3,
    pub inverse_inertia_tensor: Mat3,
    pub gravity: bool,
    pub restitution: f32,
    pub is_static: bool
}


impl RigidBody {
    fn apply_force(&mut self, force: Vec3, point: Vec3) {
        // Implementation
    }

    fn apply_torque(&mut self, torque: Vec3) {
        // Implementation
    }

    fn clear_accumulators(&mut self) {
        // Implementation
    }
}

pub struct Transform{
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3
}

impl Default for RigidBody {
    fn default() -> Self {
        Self {
            inverse_mass : 0.0,
            transform : Default::default(),
            acceleration : Default::default(),
            velocity : Default::default(),
            angular_velocity : Default::default(),
            inverse_inertia_tensor : Default::default(),
            gravity : Default::default(),
            restitution  : 0.5,
            is_static : false
        }
    }
}
impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Default::default(),
            rotation: Default::default(),
            scale: Vec3::new(1.0, 1.0, 1.0)
        }
    }
}


pub fn physics_system(mut search: Search<(&mut RigidBody, &mut obb::DynamicOBB)>, fixed_update: f32) {
    let bodies_and_boxes = search.iter().collect::<Vec<_>>();

    let mut collision_details = Vec::new();
    for i in 0..bodies_and_boxes.len() {
        for j in (i + 1)..bodies_and_boxes.len() {
            let (rb, obb1) = &bodies_and_boxes[i];
            let (rb2, obb2) = &bodies_and_boxes[j];
            if rb.is_static && rb2.is_static {
                continue;
            }
            
            if let Some(collision_point) = obb1.get_collision_point_normal(*obb2) {
                collision_details.push((i, j, collision_point));
            }
        }
    }
    let mut bodies_and_boxes: Vec<RefCell<(&mut RigidBody, &mut DynamicOBB)>> = search
        .iter()
        .map(|(rb, obb)| RefCell::new((rb, obb)))
        .collect::<Vec<RefCell<(&mut RigidBody, &mut DynamicOBB)>>>();
    for collision in collision_details {
        let rb1 = &mut bodies_and_boxes[collision.0].borrow_mut().0;
        let rb2 = &mut bodies_and_boxes[collision.1].borrow_mut().0;
        handle_collision(rb1, rb2, &collision.2);
    }

    bodies_and_boxes.iter_mut().for_each(|b_b| {
        let (rb, obb) = b_b.borrow_mut().get_mut();

        if !rb.is_static {
            integrate_rigid_body(rb, fixed_update);
        }
        obb.center = rb.transform.position;
        obb.orientation = rb.transform.rotation;
        obb.half_extents = rb.transform.scale * 0.5;
    });
}


pub trait InertiaTensor {
    fn get_inverse_cube_inertia_tensor(half_extents: Vec3, mass: f32) -> Mat3;
}

impl InertiaTensor for Mat3 {
    fn get_inverse_cube_inertia_tensor(half_extents: Vec3, mass: f32) -> Mat3 {
        let x = half_extents.x;
        let y = half_extents.y;
        let z = half_extents.z;
        let x2 = x * x;
        let y2 = y * y;
        let z2 = z * z;
        
        // Compute the diagonal components of the inertia tensor
        let coeff = mass / 12.0;
        let ix = coeff * (y2 + z2);
        let iy = coeff * (x2 + z2);
        let iz = coeff * (x2 + y2);
        
        // Calculate the inverse of each diagonal element
        let inv_ix = if ix != 0.0 { 1.0 / ix } else { 0.0 };
        let inv_iy = if iy != 0.0 { 1.0 / iy } else { 0.0 };
        let inv_iz = if iz != 0.0 { 1.0 / iz } else { 0.0 };
        
        // Create the inverse inertia tensor as a diagonal matrix
        Mat3::from_cols(
            Vec3::new(inv_ix, 0.0, 0.0),
            Vec3::new(0.0, inv_iy, 0.0),
            Vec3::new(0.0, 0.0, inv_iz),
        )
    }
}


impl RigidBody {
    pub fn new(mass: f32, transform: Transform, ) -> Self{
        let scale = transform.scale;
        Self {
            inverse_mass: 1.0/mass,
            transform: transform,
            acceleration: Default::default(),
            velocity: Default::default(),
            angular_velocity: Default::default(),
            inverse_inertia_tensor: Mat3::get_inverse_cube_inertia_tensor(scale * 0.5, mass),
            gravity: true,
            restitution: 0.5,
            is_static: false,
        }
    }
    pub fn new_static( transform: Transform, ) -> Self{
        Self {
            inverse_mass: 0.0,
            transform: transform,
            acceleration: Default::default(),
            velocity: Default::default(),
            angular_velocity: Default::default(),
            inverse_inertia_tensor: Mat3::IDENTITY,
            gravity: false,
            restitution: 0.0,
            is_static: true,
        }
    }   
}