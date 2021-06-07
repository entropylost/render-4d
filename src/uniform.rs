use crate::voxel::VoxelTypeInternal;
use bevy::prelude::*;
use bytemuck::*;
use nalgebra::Matrix4;
use wgpu::*;

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy, Debug)]
pub struct Uniforms {
    pub projection_matrix: Matrix4<f32>,
    pub voxel_types: [VoxelTypeInternal; 256],
}

pub struct UniformBuffer(pub Buffer);
pub struct UniformBindGroup(pub BindGroup, pub BindGroupLayout);

pub fn init_uniforms(mut commands: Commands, device: Res<Device>) {
    let buffer = device.create_buffer(&BufferDescriptor {
        label: Some("uniform-buffer"),
        size: std::mem::size_of::<Uniforms>() as u64,
        usage: BufferUsage::UNIFORM | BufferUsage::COPY_DST,
        mapped_at_creation: false,
    });
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("uniform-bind-group-layout"),
        entries: &[BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStage::FRAGMENT,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("uniform-bind-group"),
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
    });
    commands.insert_resource(Uniforms {
        // TODO: CHANGE
        projection_matrix: Default::default(),
        voxel_types: [Default::default(); 256],
    });
    commands.insert_resource(UniformBuffer(buffer));
    commands.insert_resource(UniformBindGroup(bind_group, bind_group_layout));
}

pub fn update_uniform_buffer(
    uniforms: Res<Uniforms>,
    queue: Res<Queue>,
    buffer: ResMut<UniformBuffer>,
) {
    if uniforms.is_changed() {
        queue.write_buffer(&buffer.0, 0, bytemuck::cast_slice(&[*uniforms]));
    }
}
