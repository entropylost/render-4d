use crate::uniform_3d;
use crate::voxel::VoxelId;
use crate::voxel::VoxelTypeInternal;
use crate::VoxelType;
use arrayvec::ArrayVec;
use bevy::prelude::*;
use nalgebra::Vector4;
use ndarray::Array4;
use std::num::NonZeroU32;
use std::ops::Index;
use std::ops::IndexMut;
use wgpu::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct WorldSize(pub u32);

#[derive(Debug, Clone)]
pub struct World {
    voxels: Array4<VoxelId>,
    types: ArrayVec<VoxelType, 256>,
    types_internal: ArrayVec<VoxelTypeInternal, 256>,
}

impl World {
    pub fn new(size: u32) -> World {
        let size = size as usize;

        let mut types = ArrayVec::new();
        let mut types_internal = ArrayVec::new();
        types.push(VoxelType::default());
        types_internal.push(types[0].to_internal());
        World {
            voxels: Array4::from_elem((size, size, size, size), VoxelId(0)),
            types,
            types_internal,
        }
    }

    pub fn insert_type(&mut self, ty: VoxelType) -> VoxelId {
        let id = self.types.len();
        self.types.push(ty);
        self.types_internal.push(ty.to_internal());
        VoxelId(id as u8)
    }

    pub fn air(&mut self) -> VoxelId {
        VoxelId(0)
    }

    pub fn size(&self) -> u32 {
        self.voxels.shape()[0] as u32
    }

    pub fn texture_layout(&self) -> ImageDataLayout {
        let size = self.size();
        ImageDataLayout {
            offset: 0,
            bytes_per_row: NonZeroU32::new(size),
            rows_per_image: NonZeroU32::new(size),
        }
    }

    pub fn voxel_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(self.voxels.as_slice().unwrap())
    }

    pub fn types_internal(&self) -> [VoxelTypeInternal; 256] {
        let mut out = [Default::default(); 256];
        out[..self.types.len()].clone_from_slice(&self.types_internal);
        out
    }
}

impl Index<Vector4<u32>> for World {
    type Output = VoxelId;
    fn index(&self, index: Vector4<u32>) -> &Self::Output {
        let index = index.cast::<usize>();
        &self.voxels[[index.x, index.y, index.z, index.w]]
    }
}
impl IndexMut<Vector4<u32>> for World {
    fn index_mut(&mut self, index: Vector4<u32>) -> &mut Self::Output {
        let index = index.cast::<usize>();
        &mut self.voxels[[index.x, index.y, index.z, index.w]]
    }
}

pub struct WorldTexture(pub Texture, pub Extent3d);
pub struct WorldBindGroup(pub BindGroup, pub BindGroupLayout);

pub fn init_world(mut commands: Commands, size: Res<WorldSize>, device: Res<Device>) {
    let size = size.0;

    let world = World::new(size);

    let extent = Extent3d {
        width: size,
        height: size,
        depth_or_array_layers: size * size,
    };

    let texture = device.create_texture(&TextureDescriptor {
        label: Some("world-texture"),
        size: extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D3,
        format: TextureFormat::R8Uint,
        usage: TextureUsage::SAMPLED | TextureUsage::COPY_DST,
    });
    let view = texture.create_view(&TextureViewDescriptor::default());
    let sampler = device.create_sampler(&SamplerDescriptor {
        mag_filter: FilterMode::Nearest,
        min_filter: FilterMode::Nearest,
        mipmap_filter: FilterMode::Nearest,
        ..Default::default()
    });

    let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("world-bind-group-layout"),
        entries: &[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStage::COMPUTE,
                ty: BindingType::Texture {
                    multisampled: false,
                    view_dimension: TextureViewDimension::D3,
                    sample_type: TextureSampleType::Uint,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStage::COMPUTE,
                ty: BindingType::Sampler {
                    comparison: false,
                    filtering: false,
                },
                count: None,
            },
        ],
    });

    let bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: Some("world-bind-group"),
        layout: &bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&view),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::Sampler(&sampler),
            },
        ],
    });

    commands.insert_resource(world);
    commands.insert_resource(WorldTexture(texture, extent));
    commands.insert_resource(WorldBindGroup(bind_group, bind_group_layout));
}

pub fn update_world(
    world: Res<World>,
    queue: Res<Queue>,
    texture: Res<WorldTexture>,
    mut uniforms: ResMut<uniform_3d::Uniforms>,
) {
    if world.is_changed() {
        queue.write_texture(
            ImageCopyTexture {
                texture: &texture.0,
                mip_level: 0,
                origin: Origin3d::ZERO,
            },
            world.voxel_bytes(),
            world.texture_layout(),
            texture.1,
        );
        uniforms.voxel_types = world.types_internal();
    }
}
