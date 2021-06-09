use crate::uniform::Uniforms;
use crate::window_size::WindowSize;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bytemuck::{Pod, Zeroable};
use nalgebra::Matrix3;
use nalgebra::Matrix4x3;
use nalgebra::Rotation;
use nalgebra::UnitQuaternion;
use nalgebra::Vector2;
use nalgebra::Vector3;
use std::f32::consts::PI;
use std::ops::RangeInclusive;

#[derive(Clone, Debug)]
pub struct Camera3d {
    pub x: f32,
    pub y: f32,
    pub pitch_range: RangeInclusive<f32>,
    pub position: Vector3<f32>,
    pub fov: f32,
    pub sensitivity: f32,
    pub speed: f32,
    pub active: bool,
}

impl Default for Camera3d {
    fn default() -> Self {
        Self::new(Vector3::zeros(), 0.0)
    }
}

impl Camera3d {
    pub fn new(position: Vector3<f32>, x: f32) -> Camera3d {
        let pitch_range = 0.01..=(PI - 0.01);
        Camera3d {
            x,
            y: (pitch_range.start() + pitch_range.end()) / 2.0,
            pitch_range,
            position,
            fov: 1.8,
            sensitivity: 1.0,
            speed: 1.0,
            active: false,
        }
    }

    fn rotation_matrix(&self) -> Matrix3<f32> {
        let rot = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), self.x)
            * UnitQuaternion::from_axis_angle(&-Vector3::x_axis(), self.y);
        *Rotation::look_at_rh(&(rot * Vector3::y()), &Vector3::y())
            .inverse()
            .matrix()
    }

    pub fn to_internal(&self, window_size: Vector2<f32>) -> Camera3dInternal {
        let r = self.rotation_matrix();
        #[rustfmt::skip]
        let rotation = Matrix4x3::new(
            r[(0, 0)], r[(0, 1)], r[(0, 2)],
            r[(1, 0)], r[(1, 1)], r[(1, 2)],
            r[(2, 0)], r[(2, 1)], r[(2, 2)],
            0.0, 0.0, 0.0
        );
        Camera3dInternal {
            position: self.position,
            _padding: 0.0,
            rotation,
            window_size,
            aspect_ratio: window_size.x / window_size.y,
            tan_half_fov: (self.fov / 2.0).tan(),
        }
    }
}

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct Camera3dInternal {
    position: Vector3<f32>,
    _padding: f32,
    rotation: Matrix4x3<f32>,
    window_size: Vector2<f32>,
    aspect_ratio: f32,
    tan_half_fov: f32,
}

pub struct Camera3dPlugin;
impl Camera3dPlugin {
    fn cursor_grab_system(
        mut windows: ResMut<Windows>,
        btn: Res<Input<MouseButton>>,
        key: Res<Input<KeyCode>>,
        mut camera: ResMut<Camera3d>,
    ) {
        let window = windows.get_primary_mut().unwrap();

        if btn.just_pressed(MouseButton::Left) {
            window.set_cursor_lock_mode(true);
            window.set_cursor_visibility(false);
            camera.active = true;
        }

        if key.just_pressed(KeyCode::Escape) {
            window.set_cursor_lock_mode(false);
            window.set_cursor_visibility(true);
            camera.active = false;
        }
    }
    fn rotate_system(
        time: Res<Time>,
        mut reader: EventReader<MouseMotion>,
        mut camera: ResMut<Camera3d>,
    ) {
        for event in reader.iter() {
            if !camera.active {
                continue;
            }
            let delta = event.delta;
            camera.x -= delta.x * camera.sensitivity * time.delta_seconds();
            camera.y += delta.y * camera.sensitivity * time.delta_seconds();
            camera.y = camera
                .y
                .max(*camera.pitch_range.start())
                .min(*camera.pitch_range.end());
        }
    }
    fn move_system(time: Res<Time>, key: Res<Input<KeyCode>>, mut camera: ResMut<Camera3d>) {
        let mut delta = Vector3::<f32>::zeros();
        if key.just_pressed(KeyCode::W) {
            delta += Vector3::x();
        }
        if key.just_pressed(KeyCode::S) {
            delta -= Vector3::x();
        }
        if key.just_pressed(KeyCode::D) {
            delta += Vector3::z();
        }
        if key.just_pressed(KeyCode::A) {
            delta -= Vector3::z();
        }
        if key.just_pressed(KeyCode::Space) {
            delta += Vector3::y();
        }
        if key.just_pressed(KeyCode::LShift) {
            delta -= Vector3::y();
        }
        if delta != Vector3::zeros() {
            delta.normalize_mut();
            delta *= time.delta_seconds() * camera.speed;
            delta = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), camera.x) * delta;
            camera.position += delta;
        }
    }
    fn update_uniform_system(
        window_size: Res<WindowSize>,
        camera: Res<Camera3d>,
        mut uniforms: ResMut<Uniforms>,
    ) {
        if camera.is_changed() {
            uniforms.camera = camera.to_internal(window_size.0.cast());
        }
    }
}
impl Plugin for Camera3dPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(Self::cursor_grab_system.system())
            .add_system(Self::rotate_system.system())
            .add_system(Self::move_system.system())
            .add_system(Self::update_uniform_system.system());
    }
}
