use crate::uniform_3d::Uniforms;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bytemuck::{Pod, Zeroable};
use nalgebra::Matrix3;
use nalgebra::Matrix4x3;
use nalgebra::UnitQuaternion;
use nalgebra::Vector3;
use std::f32::consts::PI;
use std::ops::RangeInclusive;

#[derive(Clone, Debug)]
pub struct Camera {
    pub x: f32,
    pub y: f32,
    pub pitch_range: RangeInclusive<f32>,
    pub position: Vector3<f32>,
    pub fov: f32,
    pub sensitivity: f32,
    pub speed: f32,
    pub active: bool,
}

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Debug, Default)]
pub struct CameraInternal {
    position: Vector3<f32>,
    _padding: f32,
    inv_rotation: Matrix4x3<f32>,
    tan_half_fov: f32,
    _padding_2: [f32; 3],
}

impl Default for Camera {
    fn default() -> Self {
        Self::new(Vector3::zeros(), 0.0)
    }
}

impl Camera {
    pub fn new(position: Vector3<f32>, x: f32) -> Camera {
        let pitch_range = 0.01..=(PI - 0.01);
        Camera {
            x,
            y: (pitch_range.start() + pitch_range.end()) / 2.0,
            pitch_range,
            position,
            fov: 1.0,
            sensitivity: 1.0,
            speed: 5.0,
            active: false,
        }
    }

    fn rotation_matrix(&self) -> Matrix3<f32> {
        let rot = UnitQuaternion::from_axis_angle(&Vector3::z_axis(), self.x)
            * UnitQuaternion::from_axis_angle(&Vector3::y_axis(), self.y)
            * UnitQuaternion::from_axis_angle(&Vector3::z_axis(), -PI / 2.0);
        *rot.to_rotation_matrix().matrix()
    }

    pub fn to_internal(&self) -> CameraInternal {
        let r = self.rotation_matrix();
        #[rustfmt::skip]
        let inv_rotation = Matrix4x3::new(
            r[(0, 0)], r[(0, 1)], r[(0, 2)],
            r[(1, 0)], r[(1, 1)], r[(1, 2)],
            r[(2, 0)], r[(2, 1)], r[(2, 2)],
            0.0, 0.0, 0.0
        );
        CameraInternal {
            position: self.position,
            _padding: 0.0,
            inv_rotation,
            tan_half_fov: (self.fov / 2.0).tan(),
            _padding_2: [0.0; 3],
        }
    }
}

pub struct CameraPlugin;
impl CameraPlugin {
    fn cursor_grab_system(
        mut windows: ResMut<Windows>,
        btn: Res<Input<MouseButton>>,
        key: Res<Input<KeyCode>>,
        mut camera: ResMut<Camera>,
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
        mut camera: ResMut<Camera>,
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
    fn move_system(time: Res<Time>, key: Res<Input<KeyCode>>, mut camera: ResMut<Camera>) {
        if !camera.active {
            return;
        }
        let mut delta = Vector3::<f32>::zeros();

        if key.pressed(KeyCode::W) {
            delta += Vector3::x();
        }
        if key.pressed(KeyCode::S) {
            delta -= Vector3::x();
        }
        if key.pressed(KeyCode::D) {
            delta -= Vector3::y();
        }
        if key.pressed(KeyCode::A) {
            delta += Vector3::y();
        }
        if key.pressed(KeyCode::Space) {
            delta += Vector3::z();
        }
        if key.pressed(KeyCode::LShift) {
            delta -= Vector3::z();
        }
        if delta != Vector3::zeros() {
            delta.normalize_mut();
            delta *= time.delta_seconds() * camera.speed;
            delta = UnitQuaternion::from_axis_angle(&Vector3::z_axis(), camera.x) * delta;
            camera.position += delta;
        }
    }
    fn update_uniform_system(camera: Res<Camera>, mut uniforms: ResMut<Uniforms>) {
        if camera.is_changed() {
            uniforms.camera = camera.to_internal();
        }
    }
}

#[derive(SystemLabel, Clone, Hash, Debug, Eq, PartialEq)]
enum Labels {
    CursorGrab,
    Rotate,
    Move,
    UpdateUniform,
}

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_set(
            SystemSet::new()
                .label("camera-3d")
                .with_system(Self::cursor_grab_system.system().label(Labels::CursorGrab))
                .with_system(
                    Self::rotate_system
                        .system()
                        .label(Labels::Rotate)
                        .after(Labels::CursorGrab),
                )
                .with_system(
                    Self::move_system
                        .system()
                        .label(Labels::Move)
                        .after(Labels::CursorGrab),
                )
                .with_system(
                    Self::update_uniform_system
                        .system()
                        .label(Labels::UpdateUniform)
                        .after(Labels::Rotate)
                        .after(Labels::Move),
                ),
        );
    }
}
