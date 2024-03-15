use std::{cell::RefCell, default};

use glam::{vec3, Mat3, Quat, Vec3};

use self::obb::{DynamicOBB, OBB};

use super::{*,obb::CollisionPoint};



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
    
    rigid_body.apply_force(impulse, collision_point.point);
    rigid_body2.apply_force(-impulse, collision_point.point);
}


pub fn handle_collision_static(rigid_body: &mut RigidBody, static_rigid_body: &mut RigidBody, collision_point: &CollisionPoint) {
    let pen_depth = collision_point.pen_depth;
    let correction_ratio = 1.0;
    let total_inverse_mass = rigid_body.inverse_mass;

    if total_inverse_mass > 0.0 {
        let correction = collision_point.normal * (pen_depth * correction_ratio / total_inverse_mass);
        rigid_body.transform.position += correction  * rigid_body.inverse_mass / total_inverse_mass ;
        
    }

    let relative_p1 = collision_point.point - rigid_body.transform.position;
    let relative_velocity = calculate_relative_velocity(&rigid_body, &static_rigid_body, relative_p1, Vec3::ZERO);
    let e = (rigid_body.restitution) / 2.0;
    let r1_cross_n = relative_p1.cross(collision_point.normal);
    let inv_mass_sum = rigid_body.inverse_mass + r1_cross_n.dot(rigid_body.inverse_inertia_tensor * r1_cross_n);
    let j = (-(1.0 + e) * relative_velocity.dot(collision_point.normal)) / inv_mass_sum;
    let impulse = collision_point.normal * j;
    rigid_body.apply_force(-impulse, collision_point.point);
}

pub struct RigidBody{
    pub inverse_mass: f32,
    pub transform: Transform,
    pub acceleration: Vec3,
    pub velocity: Vec3,
    pub angular_velocity:Vec3,
    pub inverse_inertia_tensor: Mat3,
    pub force_accumulator: Vec3,
    pub torque_accumulator: Vec3,
    pub gravity: bool,
    pub angular_drag: f32,
    pub restitution: f32,
    pub is_static: bool
}


impl RigidBody {
    fn apply_force(&mut self, force: Vec3, point: Vec3) {
        debug_assert_ne!(self.is_static, true, "Static rigid bodies cannot have forces");

        self.force_accumulator += force;
        if point != self.transform.position {
            let lever_arm = point - self.transform.position;
            self.apply_torque(lever_arm.cross(force));
        }
        
    }
    fn apply_angular_drag(&mut self, fixed_update: f32) {
        debug_assert_ne!(self.is_static, true, "Static rigid bodies cannot have angular drag");
        self.angular_velocity *= 1.0 - self.angular_drag * fixed_update;
    }

    fn apply_torque(&mut self, torque: Vec3) {
        debug_assert_ne!(self.is_static, true, "Static rigid bodies cannot have torques");
        self.torque_accumulator += torque;
    }

    fn clear_accumulators(&mut self) {
        debug_assert_ne!(self.is_static, true, "Static rigid bodies cannot have accumulators");
        self.torque_accumulator = Vec3::ZERO;
        self.force_accumulator = Vec3::ZERO;
    }

    pub fn integrate(&mut self, fixed_time: f32) {
        debug_assert_ne!(fixed_time, 0.0, "Fixed time step cannot be zero");
        debug_assert_ne!(self.is_static, true, "Static rigid bodies cannot be integrated");
        let accel = self.force_accumulator * self.inverse_mass;

        self.velocity += accel * fixed_time;
        self.transform.position += self.velocity * fixed_time;

        let angular_velocity = self.inverse_inertia_tensor * self.torque_accumulator;
        self.angular_velocity += angular_velocity * fixed_time;
        
        let rotation = Quat::from_euler( glam::EulerRot::XYZ ,self.angular_velocity.x, self.angular_velocity.y, self.angular_velocity.z);
        self.transform.rotation = (rotation * self.transform.rotation).normalize();
        self.clear_accumulators();
    
    }

    pub fn apply_gravity(&mut self) {
        if self.gravity {
            self.apply_force((GRAVITY * 0.1) * (1.0/self.inverse_mass), self.transform.position);
        }
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
            force_accumulator : Default::default(),
            torque_accumulator : Default::default(),
            angular_drag: 0.01,
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
            rb.apply_gravity();
            rb.apply_angular_drag(fixed_update);
            rb.integrate(fixed_update);
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
            force_accumulator : Default::default(),
            torque_accumulator : Default::default(),
            angular_drag: 0.01,
            gravity: true,
            restitution: 0.5,
            is_static: false,
        }
    }
    pub fn new_static( transform: Transform, ) -> Self{
        let scale = transform.scale;
        Self {
            inverse_mass: 0.0,
            transform: transform,
            acceleration: Default::default(),
            velocity: Default::default(),
            angular_velocity: Default::default(),
            inverse_inertia_tensor: Mat3::IDENTITY,
            force_accumulator : Default::default(),
            torque_accumulator : Default::default(),
            angular_drag: 0.01,
            gravity: false,
            restitution: 0.0,
            is_static: true,
        }
    }   
}