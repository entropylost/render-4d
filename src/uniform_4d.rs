use crate::camera_4d::CameraInternal;
use bytemuck::{Pod, Zeroable};
use wgpu::*;

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy, Debug)]
pub struct Uniforms {
    pub camera: CameraInternal,
    world_size: u32,
}

pub struct UniformBuffer(pub Buffer);
pub struct UniformBindGroup(pub BindGroup, pub BindGroupLayout);


