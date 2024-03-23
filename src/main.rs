use std::sync::mpsc::Sender;

use application::application::{CooperApplication, GameEvent};
use frost::{obb, physics::math::physics_system, RigidBody, System,SearchIter,Transform};
use glam::{Mat4, Quat, Vec3};

struct GfxLocation(usize);
fn main() {  
    env_logger::init();

    CooperApplication::create().run(
        // creates 3 cubes
        |event_stream: &Sender<GameEvent>, world| {

            (0..13).into_iter().for_each(|_| {
                event_stream
                    .send(GameEvent::Spawn("models/cube.gltf".to_string()))
                    .unwrap();
            });
            event_stream.send(GameEvent::MoveEvent(0, Mat4::from_translation(Vec3::ZERO))).unwrap();
            // platform
            let platform_transform = Transform {
                position: Vec3::new(0.0, -20.0, 0.0),
                rotation: Quat::IDENTITY,
                scale: Vec3::new(5.0, 1.0, 5.0)
            };
            world.new_entity((
                GfxLocation(0),
                RigidBody::new_static(platform_transform.clone()),
                obb::DynamicOBB::from_transform(platform_transform)
            )).unwrap();
            // medium cube
            let medium_cube_transform = Transform {
                position: Vec3::new(0.0, 10.0, 0.0),
                rotation: Quat::from_euler(glam::EulerRot::XYZ,0.0, 0.0, 0.0),
                scale: Vec3::new(2.5, 2.5, 2.5)
            };
            let mut rb= RigidBody::new(
                100.0,
                medium_cube_transform,);
            rb.gravity = true;
            rb.restitution = 0.1;
            world.new_entity((
                GfxLocation(1),
                rb ,
                obb::DynamicOBB::from_transform(medium_cube_transform)
            )).unwrap();
            // platform
            world.new_entity((
                GfxLocation(5),
                RigidBody::new_static(
                    Transform {
                        position: Vec3::new(10.0, -20.0, 0.0),
                        rotation: Quat::IDENTITY,
                        scale: Vec3::new(5.0, 1.0, 5.0)
                    },),
                obb::DynamicOBB::new(
                    Vec3::new(0.0, -20.0, 0.0),
                    Vec3::new(2.5, 0.5, 2.5),
                    Quat::IDENTITY,
                )
            )).unwrap();
            // medium cube
            let mut rb= RigidBody::new(
                100.0,
                Transform {
                    position: Vec3::new(10.0, 10.0, 0.0),
                    rotation: Quat::from_euler(glam::EulerRot::XYZ,0.0, 0.0, 60.0),
                    scale: Vec3::new(2.5, 2.5, 2.5)
                },);
            rb.gravity = true;
            rb.restitution = 0.1;
            world.new_entity((
                GfxLocation(7),
                rb ,
                obb::DynamicOBB::new(
                    Vec3::new(10.0, 10.0, 0.0),
                    Vec3::new(1.25, 1.25, 1.25),
                    Quat::from_euler(glam::EulerRot::XYZ,0.0, 0.0, 60.0),
                )
            )).unwrap();
            // platform
            world.new_entity((
                GfxLocation(6),
                RigidBody::new_static(
                    Transform {
                        position: Vec3::new(-10.0, -20.0, 0.0),
                        rotation: Quat::IDENTITY,
                        scale: Vec3::new(5.0, 1.0, 5.0)
                    },),
                obb::DynamicOBB::new(
                    Vec3::new(-10.0, -20.0, 0.0),
                    Vec3::new(2.5, 0.5, 2.5),
                    Quat::IDENTITY,
                )
            )).unwrap();
            // medium cube
            let mut rb= RigidBody::new(
                100.0,
                Transform {
                    position: Vec3::new(-10.0, 10.0, 0.0),
                    rotation: Quat::from_euler(glam::EulerRot::XYZ,0.0, 0.0, 60.0),
                    scale: Vec3::new(2.5, 2.5, 2.5)
                },);
            rb.gravity = true;
            rb.restitution = 0.1;
            world.new_entity((
                GfxLocation(8),
                rb ,
                obb::DynamicOBB::new(
                    Vec3::new(-10.0, 10.0, 0.0),
                    Vec3::new(1.25, 1.25, 1.25),
                    Quat::from_euler(glam::EulerRot::XYZ,0.0, 0.0, 45.0),
                )
            )).unwrap();

            // small cube
            let mut rb= RigidBody::new(
                20.0,
                Transform {
                    position: Vec3::new(0.0, 25.0, 0.0),
                    rotation: Quat::IDENTITY,
                    scale: Vec3::new(1.25, 1.25, 1.25)
                },);
            rb.velocity = Vec3::new(0.0, 0.0, 0.0);
            rb.gravity = false;
            world.new_entity((
                GfxLocation(2),
                rb ,
                obb::DynamicOBB::new(
                    Vec3::new(0.0, 25.0, 0.0),
                    Vec3::new(0.75, 0.75, 0.75),
                    Quat::IDENTITY,
                )
            )).unwrap();
            {// moving collider
            let moving_transform = Transform {
                position: Vec3::new(-50.0, 0.0, 0.0),
                rotation: Quat::IDENTITY,
                scale: Vec3::new(1., 1., 1.)
            };
            let mut rb= RigidBody::new(
                5.0,
                moving_transform,);
            rb.gravity = false;
            rb.velocity = Vec3::new(2.0, 0.0, 0.0);
            rb.restitution = 1.0;
            world.new_entity((
                GfxLocation(3),
                rb ,
                obb::DynamicOBB::from_transform(moving_transform)
            )).unwrap();

            // starts stationary
            let stationary_cube_transform = Transform {
                position: Vec3::new(-20.0, 0.0, 0.0),
                rotation: Quat::IDENTITY,
                scale: Vec3::new(1.0, 1.0, 1.0)
            };
            let mut rb= RigidBody::new(
                25.0,
                stationary_cube_transform,);
            rb.gravity = false;
            rb.velocity = Vec3::new(0.0, 0.0, 0.0);
            rb.restitution = 1.0;
            world.new_entity((
                GfxLocation(4),
                rb ,
                obb::DynamicOBB::from_transform(stationary_cube_transform)
            )).unwrap();
            }

            // moving collider
            let moving_transform = Transform {
                position: Vec3::new(-50.0, -5.9, 0.0),
                rotation: Quat::IDENTITY,
                scale: Vec3::new(1., 1., 1.)
            };
            let mut rb= RigidBody::new(
                5.0,
                moving_transform,);
            rb.gravity = false;
            rb.velocity = Vec3::new(2.0, 0.0, 0.0);
            rb.restitution = 1.0;
            world.new_entity((
                GfxLocation(9),
                rb ,
                obb::DynamicOBB::from_transform(moving_transform)
            )).unwrap();

            // starts stationary
            let stationary_cube_transform = Transform {
                position: Vec3::new(-20.0, -5., 0.0),
                rotation: Quat::IDENTITY,
                scale: Vec3::new(1.0, 1.0, 1.0)
            };
            let mut rb= RigidBody::new(
                100.0,
                stationary_cube_transform,);
            rb.gravity = false;
            rb.velocity = Vec3::new(0.0, 0.0, 0.0);
            rb.restitution = 1.0;
            world.new_entity((
                GfxLocation(10),
                rb ,
                obb::DynamicOBB::from_transform(stationary_cube_transform)
            )).unwrap();
            // moving collider
            let moving_transform = Transform {
                position: Vec3::new(-50.0, -10.5, 0.4),
                rotation: Quat::IDENTITY,
                scale: Vec3::new(1., 1., 1.)
            };
            let mut rb= RigidBody::new(
                5.0,
                moving_transform,);
            rb.gravity = false;
            rb.velocity = Vec3::new(2.0, 0.0, 0.0);
            rb.restitution = 0.9;

            world.new_entity((
                GfxLocation(11),
                rb ,
                obb::DynamicOBB::from_transform(moving_transform)
            )).unwrap();

            // starts stationary
            let stationary_cube_transform = Transform {
                position: Vec3::new(-20.0, -10.0, 0.0),
                rotation: Quat::IDENTITY,
                scale: Vec3::new(1.0, 1.0, 1.0)
            };
            let mut rb= RigidBody::new(
                25.0,
                stationary_cube_transform,);
            rb.gravity = false;
            rb.velocity = Vec3::new(0.0, 0.0, 0.0);
            rb.restitution = 0.9;
            world.new_entity((
                GfxLocation(12),
                rb ,
                obb::DynamicOBB::from_transform(stationary_cube_transform)
            )).unwrap();
        
        },
        |renderer_event_stream, delta| {

        },
        |renderer_event_stream, delta, world| {
            physics_system.run(world, delta).unwrap();
            let mut rb_search = world.search::<(&GfxLocation, &RigidBody)>().unwrap();
            rb_search.iter().for_each(|(gfx, rb)| {
                
                let new_location = Mat4::from_scale_rotation_translation(rb.transform.scale, rb.transform.rotation, rb.transform.position);
                
                renderer_event_stream
                    .send(GameEvent::MoveEvent(gfx.0, new_location))
                    .unwrap();
            });

            
            
        },
        |event_stream| {
            event_stream.send(GameEvent::NextFrame).unwrap();
        },
    );
}
