use crate::uniform_4d::Uniforms;
use crate::world::WorldSize;
use bevy::prelude::*;
use bytemuck::{Pod, Zeroable};
use nalgebra::Matrix4;
use nalgebra::Rotation;
use nalgebra::Vector4;
use std::f32::consts::PI;
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

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Debug, Default)]
pub struct CameraInternal {
    position: Vector4<f32>,
    rotation: Matrix4<f32>,
    voxel_size: f32,
    _padding: [f32; 3],
}

fn print_mat4(m: Matrix4<f32>) {
    #[rustfmt::skip]
    println!("{} {} {} {}\n{} {} {} {}\n{} {} {} {}\n{} {} {} {}",
        m[(0, 0)], m[(0, 1)], m[(0, 2)], m[(0, 3)],
        m[(1, 0)], m[(1, 1)], m[(1, 2)], m[(1, 3)],
        m[(2, 0)], m[(2, 1)], m[(2, 2)], m[(2, 3)],
        m[(3, 0)], m[(3, 1)], m[(3, 2)], m[(3, 3)],
    );
}

impl Camera {
    pub fn new() -> Self {
        #[rustfmt::skip]
        let rotation = Matrix4::new(
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
            1.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
        );
        println!("{:?}", rotation);
        print_mat4(rotation);
        Camera {
            rotate_time: Duration::from_secs(1),
            rotating: None,
            rotation: Rotation4::from_matrix_unchecked(rotation),
        }
    }

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

    fn to_internal(&self, world_size: WorldSize) -> CameraInternal {
        let rotation = *self.rotation.matrix();
        CameraInternal {
            position: Vector4::repeat(world_size.0 as f32 / 2.0 + 1.0)
                - rotation * Vector4::new(0.0, 0.0, 0.0, 1.0) * world_size.0 as f32 * 2.0,
            rotation,
            voxel_size: 1.0, // TODO
            _padding: [0.0; 3],
        }
    }
}

fn r1(t: f32) -> Rotation4<f32> {
    if t == 1.0 {
        #[rustfmt::skip]
        let rot = Matrix4::new(
            0.0, -1., 0.0, 0.0,
            1.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        );
        Rotation4::from_matrix_unchecked(rot)
    } else {
        let t = t * PI / 2.0;
        #[rustfmt::skip]
        let rot = Matrix4::new(
            t.cos(), -t.sin(), 0.0, 0.0,
            t.sin(), t.cos(), 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        );
        Rotation4::from_matrix_unchecked(rot)
    }
}
fn r2(t: f32) -> Rotation4<f32> {
    if t == 1.0 {
        #[rustfmt::skip]
        let rot = Matrix4::new(
            1.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, -1., 0.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        );
        Rotation4::from_matrix_unchecked(rot)
    } else {
        let t = t * PI / 2.0;
        #[rustfmt::skip]
        let rot = Matrix4::new(
            1.0, 0.0, 0.0, 0.0,
            0.0, t.cos(), t.sin(), 0.0,
            0.0, -t.sin(), t.cos(), 0.0,
            0.0, 0.0, 0.0, 1.0,
        );
        Rotation4::from_matrix_unchecked(rot)
    }
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
            println!("Q");
            if key.pressed(KeyCode::LShift) {
                camera.rotate(r1_inv);
            } else {
                camera.rotate(r1);
            }
        } else if key.just_pressed(KeyCode::E) {
            println!("E");
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
            camera.rotation = (rotating.interpolate)(t) * rotating.last_rotation;
            if t == 1.0 {
                print_mat4(*camera.rotation.matrix());
                println!("X -> {:?}", camera.rotation * Vector4::x());
                println!("Y -> {:?}", camera.rotation * Vector4::y());
                println!("Z -> {:?}", camera.rotation * Vector4::z());
                println!("W -> {:?}", camera.rotation * Vector4::w());
                camera.rotating = None;
            }
        }
    }
    fn update_uniform_system(
        world_size: Res<WorldSize>,
        camera: Res<Camera>,
        mut uniforms: ResMut<Uniforms>,
    ) {
        if camera.is_changed() {
            uniforms.camera = camera.to_internal(*world_size);
        }
    }
}

#[derive(SystemLabel, Clone, Hash, Debug, Eq, PartialEq)]
enum Labels {
    Rotate,
    Rotating,
    UpdateUniform,
}

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_set(
            SystemSet::new()
                .label("camera-4d")
                .with_system(Self::rotate_system.system().label(Labels::Rotate))
                .with_system(
                    Self::rotating_system
                        .system()
                        .label(Labels::Rotating)
                        .after(Labels::Rotate),
                )
                .with_system(
                    Self::update_uniform_system
                        .system()
                        .label(Labels::UpdateUniform)
                        .after(Labels::Rotating),
                ),
        );
    }
}
