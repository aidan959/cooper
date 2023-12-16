use std::sync::mpsc::Sender;

use application::application::{CooperApplication, GameEvent};
use glam::{Mat4, Quat, Vec3, Vec4, const_vec3};

const CENTRE :Vec3 =  const_vec3!([0.,0.,0.]);

fn main() {
    env_logger::init();
    let mut amount : Mat4 = Mat4::default();

    let mut counter : f32 = 0.;
    let mut fixed_counter : f32 = 0.;
    let mut fixed_amount : Mat4 = Mat4::default();
    
    CooperApplication::create().run(
        // creates 3 cubes
        |event_stream: &Sender<GameEvent>| {
            (0..3).into_iter().for_each(|_|{
                event_stream.send(GameEvent::Spawn("models/cube.gltf".to_string())).unwrap();
            })
        },
        |renderer_event_stream, delta|
        {
            let (s,mut r,mut t) = amount.to_scale_rotation_translation();
            t.x = 5. * f32::cos(f32::to_radians(counter));
            t.z = 5. * f32::sin(f32::to_radians(counter));

            r = look_at(t, CENTRE);
            amount = Mat4::from_scale_rotation_translation(s,r, t);
            renderer_event_stream.send(GameEvent::MoveEvent(0,amount)).unwrap();
            counter += 20. * delta;
        },
        |renderer_event_stream, _delta|
        {
            let (s,mut r,mut t) = fixed_amount.to_scale_rotation_translation();
            t.x = 5.;
            t.z = 5.;
            t.y = 1. * f32::sin(f32::to_radians(fixed_counter));
            r = look_at(t, CENTRE);
            fixed_amount = Mat4::from_scale_rotation_translation(s,r, t);
            renderer_event_stream.send(GameEvent::MoveEvent(3,fixed_amount)).unwrap();
            fixed_counter += 50. * _delta;
        },
        |event_stream| {
            event_stream.send(GameEvent::NextFrame).unwrap();
        }
    );
}

fn look_at(position: Vec3, target: Vec3) -> Quat {
    let forward = (target - position).normalize();
    let rotation_axis = Vec3::new(0.0, 1.0, 0.0).cross(forward).normalize();
    let rotation_angle = f32::acos(Vec3::new(0.0, 1.0, 0.0).dot(forward));

    Quat::from_axis_angle(rotation_axis, rotation_angle)
}