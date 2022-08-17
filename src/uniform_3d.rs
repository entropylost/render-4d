use crate::camera_3d::CameraInternal;
use crate::surface::{DeviceResource, QueueResource};
use crate::voxel::VoxelTypeInternal;
use crate::window_size::WindowSize;
use bevy::prelude::*;
use bytemuck::*;
use nalgebra::Vector2;
use wgpu::*;

#[repr(C)]
#[derive(Resource, Pod, Zeroable, Clone, Copy, Debug)]
pub struct Uniforms {
    pub camera: CameraInternal,
    pub window_size: Vector2<f32>,
    _padding: [f32; 2],
    pub voxel_types: [VoxelTypeInternal; 256],
}

#[derive(Resource)]
pub struct UniformBuffer(pub Buffer);
#[derive(Resource)]
pub struct UniformBindGroup(pub BindGroup, pub BindGroupLayout);

pub fn init_uniforms(
    mut commands: Commands,
    device: Res<DeviceResource>,
    window_size: Res<WindowSize>,
) {
    let buffer = device.create_buffer(&BufferDescriptor {
        label: Some("uniform-3d-buffer"),
        size: std::mem::size_of::<Uniforms>() as u64,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("uniform-3d-bind-group-layout"),
        entries: &[BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("uniform-3d-bind-group"),
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
    });
    commands.insert_resource(Uniforms {
        camera: Default::default(),
        window_size: window_size.0.cast(),
        _padding: Default::default(),
        voxel_types: [Default::default(); 256],
    });
    commands.insert_resource(UniformBuffer(buffer));
    commands.insert_resource(UniformBindGroup(bind_group, bind_group_layout));
}

pub fn update_uniform_buffer(
    uniforms: Res<Uniforms>,
    queue: Res<QueueResource>,
    buffer: ResMut<UniformBuffer>,
) {
    if uniforms.is_changed() {
        queue.write_buffer(&buffer.0, 0, bytemuck::cast_slice(&[*uniforms]));
    }
}
