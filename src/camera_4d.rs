use bevy::prelude::*;
use nalgebra::Rotation;
use std::time::Duration;
use std::time::Instant;

type Rotation4<T> = Rotation<T, 4>;

#[derive(Copy, Clone, Debug)]

pub struct Camera {
    pub rotate_time: Duration,
    pub rotating: Option<Rotating>,
    pub rotation: Rotation4<f32>,
}

#[derive(Copy, Clone, Debug)]
pub struct Rotating {
    last_rotation: Rotation4<f32>,
    interpolate: fn(f32) -> Rotation4<f32>,
    start_time: Instant,
}

impl Camera {
    fn rotate(&mut self, f: fn(f32) -> Rotation4<f32>) -> bool {
        if self.rotating.is_some() {
            return false;
        }
        self.rotating = Some(Rotating {
            last_rotation: self.rotation,
            interpolate: f,
            start_time: Instant::now(),
        });
        return true;
    }
}

fn r1(t: f32) -> Rotation4<f32> {
    todo!()
}
fn r2(t: f32) -> Rotation4<f32> {
    todo!()
}
fn r1_inv(t: f32) -> Rotation4<f32> {
    r1(-t)
}
fn r2_inv(t: f32) -> Rotation4<f32> {
    r2(-t)
}

pub struct CameraPlugin;
impl CameraPlugin {
    fn rotate_system(key: Res<Input<KeyCode>>, mut camera: ResMut<Camera>) {
        if camera.rotating.is_some() {
            return;
        }
        if key.just_pressed(KeyCode::Q) {
            if key.pressed(KeyCode::LShift) {
                camera.rotate(r1_inv);
            } else {
                camera.rotate(r1);
            }
        } else if key.just_pressed(KeyCode::E) {
            if key.pressed(KeyCode::LShift) {
                camera.rotate(r2_inv);
            } else {
                camera.rotate(r2);
            }
        }
    }
    fn rotating_system(mut camera: ResMut<Camera>) {
        if let Some(rotating) = camera.rotating {
            let now = Instant::now();
            let t = (now - rotating.start_time)
                .div_duration_f32(camera.rotate_time)
                .min(1.0);
            camera.rotation = (rotating.interpolate)(t);
            if t == 1.0 {
                camera.rotating = None;
            }
        }
    }
}
