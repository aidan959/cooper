use std::{f32::INFINITY, sync::mpsc::Sender};

use application::application::{CooperApplication, GameEvent};
use frost::{obb, physics::math::physics_system, RigidBody, System,SearchIter,Search, ChainedIterator, Transform, World};
use glam::{const_vec3, Mat3, Mat4, Quat, Vec3};

const CENTRE: Vec3 = const_vec3!([0., 0., 0.]);
struct GfxLocation(usize);
fn main() {  
    env_logger::init();
    let mut amount: Mat4 = Mat4::default();

    let mut counter: f32 = 0.;
    let mut fixed_counter: f32 = 0.;
    let mut fixed_amount: Mat4 = Mat4::default();

    CooperApplication::create().run(
        // creates 3 cubes
        |event_stream: &Sender<GameEvent>, world| {

            (0..2).into_iter().for_each(|_| {
                event_stream
                    .send(GameEvent::Spawn("models/cube.gltf".to_string()))
                    .unwrap();
            });
            event_stream.send(GameEvent::MoveEvent(0, Mat4::from_translation(Vec3::ZERO))).unwrap();
            world.new_entity((
                GfxLocation(0),
                RigidBody::new_static(
                    Transform {
                        position: Vec3::new(0.0, -20.0, 0.0),
                        rotation: Quat::IDENTITY,
                        scale: Vec3::new(5.0, 1.0, 5.0)
                    },),
                obb::DynamicOBB::new(
                    Vec3::new(0.0, -20.0, 0.0),
                    Vec3::new(2.5, 0.5, 2.5),
                    Quat::IDENTITY,
                )
            )).unwrap();
            let mut rb= RigidBody::new(
                10.0,
                Transform {
                    position: Vec3::new(0.0, 10.0, 0.0),
                    rotation: Quat::from_euler(glam::EulerRot::XYZ, 0.0, 0.0, 0.0),
                    scale: Vec3::new(2.5, 2.5, 2.5)
                },);
            rb.velocity = Vec3::new(0.0, -1.0, 0.0);
            

            world.new_entity((
                GfxLocation(1),
                rb ,
                obb::DynamicOBB::new(
                    Vec3::new(0.0, 10.0, 0.0),
                    Vec3::new(1.25, 1.25, 1.25),
                    Quat::IDENTITY,
                )
            )).unwrap();
            let mut rb= RigidBody::new(
                5.0,
                Transform {
                    position: Vec3::new(0.0, 25.0, 0.0),
                    rotation: Quat::IDENTITY,
                    scale: Vec3::new(1.25, 1.25, 1.25)
                },);
            rb.velocity = Vec3::new(0.0, -1.0, 0.0);

            world.new_entity((
                GfxLocation(2),
                rb ,
                obb::DynamicOBB::new(
                    Vec3::new(0.0, 25.0, 0.0),
                    Vec3::new(0.75, 0.75, 0.75),
                    Quat::IDENTITY,
                )
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
