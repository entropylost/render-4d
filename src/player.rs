use bytemuck::{Pod, Zeroable};
use derive_new::new;
use nalgebra::Matrix3;
use nalgebra::Matrix4x3;
use nalgebra::Rotation;
use nalgebra::Vector2;
use nalgebra::Vector3;

#[derive(new, Copy, Clone, Debug)]
pub struct Player {
    pub position: Vector3<f32>,
    pub facing: Vector3<f32>,
    pub fov: f32,
}

impl Player {
    fn rotation_matrix(&self) -> Matrix3<f32> {
        *Rotation::look_at_rh(&self.facing, &Vector3::y())
            .inverse()
            .matrix()
    }

    pub fn to_internal(&self, screen_size: Vector2<f32>) -> CameraInternal {
        let r = self.rotation_matrix();
        #[rustfmt::skip]
        let rotation = Matrix4x3::new(
            r[(0, 0)], r[(0, 1)], r[(0, 2)],
            r[(1, 0)], r[(1, 1)], r[(1, 2)],
            r[(2, 0)], r[(2, 1)], r[(2, 2)],
            0.0, 0.0, 0.0
        );
        CameraInternal {
            position: self.position,
            _padding: 0.0,
            rotation,
            screen_size,
            aspect_ratio: screen_size.x / screen_size.y,
            tan_half_fov: (self.fov / 2.0).tan(),
        }
    }
}

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct CameraInternal {
    position: Vector3<f32>,
    _padding: f32,
    rotation: Matrix4x3<f32>,
    screen_size: Vector2<f32>,
    aspect_ratio: f32,
    tan_half_fov: f32,
}
