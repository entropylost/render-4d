use crate::view::ViewTexture;
use crate::voxel::VoxelId;
use bevy::prelude::*;
use nalgebra::Vector3;
use ndarray::Array3;
use std::num::NonZeroU32;
use wgpu::*;

pub struct Player {
    position: Vector3<f32>,
    velocity: Vector3<f32>,
    air_friction: f32,
    accelleration: f32,
    size: u32,
}

pub struct PhysicsViewStagingBuffer(Buffer);
pub struct PhysicsView {
    start: Vector3<u32>,
    next_start: Vector3<u32>,
    voxels: Array3<VoxelId>,
    size: u32,
}

pub struct PhysicsPlugin;

impl PhysicsPlugin {
    fn collide_player(player: ResMut<Player>, view: Res<PhysicsView>) {}

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
                mapped_at_creation: false,
            },
        )));
    }

    fn update_view(
        device: Res<Device>,
        queue: Res<Queue>,
        staging_buffer: Res<PhysicsViewStagingBuffer>,
        physics_view: Res<PhysicsView>,
        view_texture: Res<ViewTexture>,
    ) {
        let buffer = &staging_buffer.0;
        let data = Box::from(buffer.slice(..).get_mapped_range());
        println!("{:?}", data);

        let size_rounded = ((physics_view.size + 255) / 256) * 256;

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
                width: physics_view.size,
                height: physics_view.size,
                depth_or_array_layers: physics_view.size,
            },
        );
        queue.submit(std::iter::once(encoder.finish()));
        let _ = buffer.slice(..).map_async(MapMode::Read);
    }
}
