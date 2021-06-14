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
impl PhysicsView {
    pub fn new(size: u32) -> Self {
        Self {
            start: Vector3::zeros(),
            next_start: Vector3::zeros(),
            voxels: Array3::from_elem((0, 0, 0), VoxelId(0)),
            size,
        }
    }
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
                mapped_at_creation: true,
            },
        )));
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
            println!("{:?}", physics_view.voxels);
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
                SystemStage::single_threaded().with_system(Self::update_view.system()),
            );
    }
}
