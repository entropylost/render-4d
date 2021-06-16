use crate::camera_3d::Camera;
use crate::view::ViewTexture;
use crate::voxel::VoxelId;
use bevy::prelude::*;
use nalgebra::Vector3;
use ndarray::s;
use ndarray::Array3;
use ndarray::ArrayView3;
use std::num::NonZeroU32;
use std::ops::Deref;
use wgpu::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Player {
    position: Vector3<f32>,
    velocity: Vector3<f32>,
    stats: PlayerStats,
    dead: bool,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PlayerStats {
    pub air_friction: f32,
    pub movement_acceleration: f32,
    pub jump_velocity: f32,
    pub size: Vector3<f32>,
    pub gravity: f32,
}

impl Player {
    pub fn new(position: Vector3<f32>, stats: PlayerStats) -> Self {
        Self {
            position,
            velocity: Vector3::zeros(),
            stats,
            dead: false,
        }
    }
}

pub struct PhysicsViewStagingBuffer(Buffer);
pub struct PhysicsView {
    start: Vector3<u32>,
    next_start: Vector3<u32>,
    voxels: Array3<VoxelId>,
    size: u32,
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct CollisionResult {
    shift: Vector3<f32>,
    collided: bool,
    conflicted: Vector3<bool>,
}

impl PhysicsView {
    pub fn new(size: u32) -> Self {
        Self {
            start: Vector3::zeros(),
            next_start: Vector3::zeros(),
            voxels: Array3::from_elem((0, 0, 0), VoxelId(0)),
            size,
        }
    }
    fn aabb_collide(&self, start: Vector3<f32>, size: Vector3<f32>) -> CollisionResult {
        let (min_shift, max_shift) = self.voxels.indexed_iter().fold(
            (Vector3::<f32>::zeros(), Vector3::<f32>::zeros()),
            |prev, current| {
                if *current.1 == VoxelId::solid_air() {
                    return prev;
                }
                let pos = Vector3::new(
                    current.0 .0 as f32,
                    current.0 .1 as f32,
                    current.0 .2 as f32,
                ) + self.start.cast();
                let diff_s = start - (pos + Vector3::repeat(1.0));
                let diff_e = start + size - pos;
                if !(diff_s < Vector3::zeros()) || !(diff_e > Vector3::zeros()) {
                    return prev;
                }
                let min_abs = diff_s.zip_map(&diff_e, |a, b| if -a > b { b } else { a });
                (
                    prev.0.zip_map(&min_abs, |a, b| a.min(b)),
                    prev.1.zip_map(&min_abs, |a, b| a.max(b)),
                )
            },
        );
        let collided = min_shift != Vector3::zeros() || max_shift != Vector3::zeros();
        let conflicted = min_shift.zip_map(&max_shift, |a, b| (a != 0.0) && (b != 0.0));
        let shift = min_shift.zip_zip_map(&max_shift, &conflicted, |a, b, c| {
            if c {
                0.0
            } else {
                if a == 0.0 {
                    b
                } else {
                    a
                }
            }
        });
        CollisionResult {
            shift,
            collided,
            conflicted,
        }
    }
}

pub struct PhysicsPlugin;

impl PhysicsPlugin {
    fn init_staging_buffer(
        mut commands: Commands,
        device: Res<Device>,
        physics_view: Res<PhysicsView>,
    ) {
        let size_rounded = ((physics_view.size + 255) / 256) * 256;

        commands.insert_resource(PhysicsViewStagingBuffer(device.create_buffer(
            &BufferDescriptor {
                label: Some("physics-view-staging-buffer"),
                size: (size_rounded * size_rounded * physics_view.size) as u64,
                usage: BufferUsage::MAP_READ | BufferUsage::COPY_DST,
                mapped_at_creation: true,
            },
        )));
    }

    fn update_physics(time: Res<Time>, mut player: ResMut<Player>, view: Res<PhysicsView>) {
        println!("Position: {:?}", player.position);
        println!("Dead: {:?}", player.dead);
        if player.dead {
            return;
        }
        let timestep = time.delta_seconds();
        let stats = player.stats;
        let acceleration = -stats.gravity * Vector3::z() - stats.air_friction * player.velocity;
        player.velocity += acceleration * timestep;
        let velocity = player.velocity;
        player.position += velocity * timestep;
        let collision = view.aabb_collide(player.position - stats.size, stats.size * 2.0);
        if !collision.collided {
            return;
        }
        if collision.conflicted.fold(true, |x, a| a & x) {
            player.dead = true;
            return;
        }
        // TODO: WALKING UP STAIRS
        player.position += collision.shift;
        player
            .velocity
            .component_mul_assign(&collision.shift.map(|x| if x == 0.0 { 1.0 } else { 0.0 }));
        if view
            .aabb_collide(player.position - stats.size, stats.size * 2.0)
            .collided
        {
            player.dead = true;
        }
    }

    fn update_camera(player: Res<Player>, mut camera: ResMut<Camera>) {
        camera.position = player.position;
    }

    fn update_view(
        device: Res<Device>,
        queue: Res<Queue>,
        staging_buffer: Res<PhysicsViewStagingBuffer>,
        mut physics_view: ResMut<PhysicsView>,
        player: Res<Player>,
        view_texture: Res<ViewTexture>,
    ) {
        let size = physics_view.size;
        let size_rounded = ((size + 255) / 256) * 256;

        let buffer = &staging_buffer.0;
        {
            let slice = buffer.slice(..).get_mapped_range();
            let data = bytemuck::cast_slice::<_, VoxelId>(slice.deref());
            let data = ArrayView3::from_shape(
                (size_rounded as usize, size_rounded as usize, size as usize),
                data,
            )
            .unwrap();
            physics_view.voxels = data
                .slice(s![0..size as usize, 0..size as usize, ..])
                .to_owned();
            physics_view.start = physics_view.next_start;
        }
        buffer.unmap();

        physics_view.next_start =
            (player.position - Vector3::repeat(size).cast() / 2.0).map(|x| x.round() as u32);

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("physics-view-update-encoder"),
        });
        encoder.copy_texture_to_buffer(
            ImageCopyTexture {
                texture: &view_texture.0,
                mip_level: 0,
                origin: Origin3d {
                    x: physics_view.next_start.x,
                    y: physics_view.next_start.y,
                    z: physics_view.next_start.z,
                },
            },
            ImageCopyBuffer {
                buffer: &buffer,
                layout: ImageDataLayout {
                    offset: 0,
                    bytes_per_row: NonZeroU32::new(size_rounded),
                    rows_per_image: NonZeroU32::new(size_rounded),
                },
            },
            Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: size,
            },
        );
        queue.submit(std::iter::once(encoder.finish()));
        let _ = buffer.slice(..).map_async(MapMode::Read);
    }
}

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system_to_stage("startup-finish", Self::init_staging_buffer.system())
            .add_stage_after(
                CoreStage::Update,
                "physics",
                SystemStage::single_threaded()
                    .with_system(Self::update_view.system())
                    .with_system(Self::update_physics.system())
                    .with_system(Self::update_camera.system()),
            );
    }
}
