use std::sync::{mpsc::Sender, Mutex};

use application::{
    application::{CooperApplication, GameEvent, GfxLocation},
    EngineSettings,
};
use frost::{obb, physics::math::physics_system, RigidBody, System, Transform};
use glam::{const_vec3, Mat4, Quat, Vec3};
use lynch::{Camera, WindowSize};

const WINDOW_SIZE: WindowSize = (1280., 1080.);
fn main() {
    env_logger::init();
    CooperApplication::builder()
        .engine_settings(
            EngineSettings::builder()
                .set_window_name("Cooper")
                .fps_max(512).unwrap()
                .window_size(WINDOW_SIZE)
                .build(),
        )
        .camera(
            Camera::builder()
                .fov_degrees(90.)
                .position(const_vec3!([0.0, 0.0, 0.0]))
                .aspect_ratio_from_window(WINDOW_SIZE)
                .build(),
        )
        //.schedule_system(application::application::Schedule::OnFixedUpdate, Box::new(physics_system))
        .build()
        .run(
            // creates 3 cubes
            |event_stream: &Sender<GameEvent>, world| {
                (0..13).into_iter().for_each(|_| {
                    event_stream
                        .send(GameEvent::Spawn("models/cube.gltf".to_string()))
                        .unwrap();
                });
                event_stream
                    .send(GameEvent::MoveEvent(0, Mat4::from_translation(Vec3::ZERO)))
                    .unwrap();
                // platform
                let platform_transform = Transform {
                    position: Vec3::new(0.0, -20.0, 0.0),
                    rotation: Quat::IDENTITY,
                    scale: Vec3::new(5.0, 1.0, 5.0),
                };
                world
                    .new_entity((
                        GfxLocation(13),
                        RigidBody::new_static(platform_transform.clone()),
                        obb::DynamicOBB::from_transform(platform_transform),
                    ))
                    .unwrap();
                // medium cube
                let medium_cube_transform = Transform {
                    position: Vec3::new(0.0, 10.0, 0.0),
                    rotation: Quat::from_euler(glam::EulerRot::XYZ, 0.0, 0.0, 0.0),
                    scale: Vec3::new(2.5, 2.5, 2.5),
                };
                let mut rb = RigidBody::new(100.0, medium_cube_transform);
                rb.gravity = true;
                rb.restitution = 0.1;
                world
                    .new_entity((
                        GfxLocation(1),
                        rb,
                        obb::DynamicOBB::from_transform(medium_cube_transform),
                    ))
                    .unwrap();
                // platform
                world
                    .new_entity((
                        GfxLocation(5),
                        RigidBody::new_static(Transform {
                            position: Vec3::new(10.0, -20.0, 0.0),
                            rotation: Quat::IDENTITY,
                            scale: Vec3::new(5.0, 1.0, 5.0),
                        }),
                        obb::DynamicOBB::new(
                            Vec3::new(0.0, -20.0, 0.0),
                            Vec3::new(2.5, 0.5, 2.5),
                            Quat::IDENTITY,
                        ),
                    ))
                    .unwrap();
                // medium cube
                let mut rb = RigidBody::new(
                    100.0,
                    Transform {
                        position: Vec3::new(10.0, 10.0, 0.0),
                        rotation: Quat::from_euler(glam::EulerRot::XYZ, 0.0, 0.0, 60.0),
                        scale: Vec3::new(2.5, 2.5, 2.5),
                    },
                );
                rb.gravity = true;
                rb.restitution = 0.1;
                world
                    .new_entity((
                        GfxLocation(7),
                        rb,
                        obb::DynamicOBB::new(
                            Vec3::new(10.0, 10.0, 0.0),
                            Vec3::new(1.25, 1.25, 1.25),
                            Quat::from_euler(glam::EulerRot::XYZ, 0.0, 0.0, 60.0),
                        ),
                    ))
                    .unwrap();
                // platform
                world
                    .new_entity((
                        GfxLocation(6),
                        RigidBody::new_static(Transform {
                            position: Vec3::new(-10.0, -20.0, 0.0),
                            rotation: Quat::IDENTITY,
                            scale: Vec3::new(5.0, 1.0, 5.0),
                        }),
                        obb::DynamicOBB::new(
                            Vec3::new(-10.0, -20.0, 0.0),
                            Vec3::new(2.5, 0.5, 2.5),
                            Quat::IDENTITY,
                        ),
                    ))
                    .unwrap();
                // medium cube
                let mut rb = RigidBody::new(
                    100.0,
                    Transform {
                        position: Vec3::new(-10.0, 10.0, 0.0),
                        rotation: Quat::from_euler(glam::EulerRot::XYZ, 0.0, 0.0, 60.0),
                        scale: Vec3::new(2.5, 2.5, 2.5),
                    },
                );
                rb.gravity = true;
                rb.restitution = 0.1;
                world
                    .new_entity((
                        GfxLocation(8),
                        rb,
                        obb::DynamicOBB::new(
                            Vec3::new(-10.0, 10.0, 0.0),
                            Vec3::new(1.25, 1.25, 1.25),
                            Quat::from_euler(glam::EulerRot::XYZ, 0.0, 0.0, 45.0),
                        ),
                    ))
                    .unwrap();

                // small cube
                let mut rb = RigidBody::new(
                    20.0,
                    Transform {
                        position: Vec3::new(0.0, 25.0, 0.0),
                        rotation: Quat::IDENTITY,
                        scale: Vec3::new(1.25, 1.25, 1.25),
                    },
                );
                rb.velocity = Vec3::new(0.0, 0.0, 0.0);
                rb.gravity = false;
                world
                    .new_entity((
                        GfxLocation(2),
                        rb,
                        obb::DynamicOBB::new(
                            Vec3::new(0.0, 25.0, 0.0),
                            Vec3::new(0.75, 0.75, 0.75),
                            Quat::IDENTITY,
                        ),
                    ))
                    .unwrap();
                {
                    // moving collider
                    let moving_transform = Transform {
                        position: Vec3::new(-50.0, 0.0, 0.0),
                        rotation: Quat::IDENTITY,
                        scale: Vec3::new(1., 1., 1.),
                    };
                    let mut rb = RigidBody::new(5.0, moving_transform);
                    rb.gravity = false;
                    rb.velocity = Vec3::new(2.0, 0.0, 0.0);
                    rb.restitution = 1.0;
                    world
                        .new_entity((
                            GfxLocation(3),
                            rb,
                            obb::DynamicOBB::from_transform(moving_transform),
                        ))
                        .unwrap();

                    // starts stationary
                    let stationary_cube_transform = Transform {
                        position: Vec3::new(-20.0, 0.0, 0.0),
                        rotation: Quat::IDENTITY,
                        scale: Vec3::new(1.0, 1.0, 1.0),
                    };
                    let mut rb = RigidBody::new(25.0, stationary_cube_transform);
                    rb.gravity = false;
                    rb.velocity = Vec3::new(0.0, 0.0, 0.0);
                    rb.restitution = 1.0;
                    world
                        .new_entity((
                            GfxLocation(4),
                            rb,
                            obb::DynamicOBB::from_transform(stationary_cube_transform),
                        ))
                        .unwrap();
                }

                // moving collider
                let moving_transform = Transform {
                    position: Vec3::new(-50.0, -5.9, 0.0),
                    rotation: Quat::IDENTITY,
                    scale: Vec3::new(1., 1., 1.),
                };
                let mut rb = RigidBody::new(5.0, moving_transform);
                rb.gravity = false;
                rb.velocity = Vec3::new(2.0, 0.0, 0.0);
                rb.restitution = 1.0;
                world
                    .new_entity((
                        GfxLocation(9),
                        rb,
                        obb::DynamicOBB::from_transform(moving_transform),
                    ))
                    .unwrap();

                // starts stationary
                let stationary_cube_transform = Transform {
                    position: Vec3::new(-20.0, -5., 0.0),
                    rotation: Quat::IDENTITY,
                    scale: Vec3::new(1.0, 1.0, 1.0),
                };
                let mut rb = RigidBody::new(100.0, stationary_cube_transform);
                rb.gravity = false;
                rb.velocity = Vec3::new(0.0, 0.0, 0.0);
                rb.restitution = 1.0;
                world
                    .new_entity((
                        GfxLocation(10),
                        rb,
                        obb::DynamicOBB::from_transform(stationary_cube_transform),
                    ))
                    .unwrap();
                // moving collider
                let moving_transform = Transform {
                    position: Vec3::new(-50.0, -10.5, 0.4),
                    rotation: Quat::IDENTITY,
                    scale: Vec3::new(1., 1., 1.),
                };
                let mut rb = RigidBody::new(5.0, moving_transform);
                rb.gravity = false;
                rb.velocity = Vec3::new(2.0, 0.0, 0.0);
                rb.restitution = 0.9;

                world
                    .new_entity((
                        GfxLocation(11),
                        rb,
                        obb::DynamicOBB::from_transform(moving_transform),
                    ))
                    .unwrap();

                // starts stationary
                let stationary_cube_transform = Transform {
                    position: Vec3::new(-20.0, -10.0, 0.0),
                    rotation: Quat::IDENTITY,
                    scale: Vec3::new(1.0, 1.0, 1.0),
                };
                let mut rb = RigidBody::new(25.0, stationary_cube_transform);
                rb.gravity = false;
                rb.velocity = Vec3::new(0.0, 0.0, 0.0);
                rb.restitution = 0.9;
                world
                    .new_entity((
                        GfxLocation(12),
                        rb,
                        obb::DynamicOBB::from_transform(stationary_cube_transform),
                    ))
                    .unwrap();
            },
            |renderer_event_stream, delta| {},
            |renderer_event_stream, delta, world| {},
            |event_stream| {
                event_stream.send(GameEvent::NextFrame).unwrap();
            },
            move |_, _ui| {},
        );
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::Sender;

    use application::application::{CooperApplication, GameEvent};
    use frost::{
        obb, physics::math::physics_system, RigidBody, Search, SearchIter, System, Transform,
    };
    use glam::{Mat4, Quat, Vec3};

    struct GfxLocation(usize);

    #[test]
    fn graphic_scene_engine() {
        env_logger::init();
        fn rotate_system(mut rbs: Search<(&mut RigidBody,)>, _fixed_delta: f32) {
            for rb in rbs.iter() {
                rb.apply_torque(Vec3::new(100.0, 0.0, 0.0));
            }
        }
        CooperApplication::create().run(
            |event_stream: &Sender<GameEvent>, world| {
                event_stream
                    .send(GameEvent::Spawn("models/2CylinderEngine.gltf".to_string()))
                    .unwrap();
                world
                    .new_entity((
                        GfxLocation(1),
                        RigidBody::new(
                            100.0,
                            Transform {
                                position: Vec3::new(0.0, 0.0, 0.0),
                                rotation: Quat::IDENTITY,
                                scale: Vec3::new(1.0, 1.0, 1.0),
                            },
                        ),
                    ))
                    .unwrap();
            },
            |_renderer_event_stream, _delta| {},
            |_renderer_event_stream, delta, world| {
                rotate_system.run(world, delta).unwrap();
            },
            |event_stream| {
                event_stream.send(GameEvent::NextFrame).unwrap();
            },
            move |_, _ui| {},
        );
    }
    #[test]
    fn graphic_scene_sponza() {
        env_logger::init();
        fn rotate_system(mut rbs: Search<(&mut RigidBody,)>, _fixed_delta: f32) {
            for rb in rbs.iter() {
                rb.apply_torque(Vec3::new(100.0, 0.0, 0.0));
            }
        }
        CooperApplication::create().run(
            |event_stream: &Sender<GameEvent>, world| {
                event_stream
                    .send(GameEvent::Spawn(
                        "models/Sponza/glTF/Sponza.gltf".to_string(),
                    ))
                    .unwrap();
                world
                    .new_entity((
                        GfxLocation(1),
                        RigidBody::new(
                            100.0,
                            Transform {
                                position: Vec3::new(0.0, 0.0, 0.0),
                                rotation: Quat::IDENTITY,
                                scale: Vec3::new(1.0, 1.0, 1.0),
                            },
                        ),
                    ))
                    .unwrap();
            },
            |_renderer_event_stream, _delta| {},
            |_renderer_event_stream, delta, world| {
                rotate_system.run(world, delta).unwrap();
            },
            |event_stream| {
                event_stream.send(GameEvent::NextFrame).unwrap();
            },
            move |_, _ui| {},
        );
    }
}
