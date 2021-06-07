use crate::uniform::Uniforms;
use crate::voxel::VoxelId;
use crate::voxel::VoxelTypeInternal;
use crate::VoxelType;
use arrayvec::ArrayVec;
use bevy::prelude::*;
use nalgebra::Vector3;
use ndarray::Array3;
use std::num::NonZeroU32;
use std::ops::Index;
use std::ops::IndexMut;
use wgpu::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct WorldSize(pub u32);

#[derive(Debug, Clone)]
pub struct World3d {
    voxels: Array3<VoxelId>,
    types: ArrayVec<VoxelType, 256>,
    types_internal: ArrayVec<VoxelTypeInternal, 256>,
}

impl World3d {
    pub fn new(size: u32) -> World3d {
        let size = size as usize;

        let mut types = ArrayVec::new();
        let mut types_internal = ArrayVec::new();
        types.push(VoxelType::default());
        types_internal.push(types[0].to_internal());
        World3d {
            voxels: Array3::from_elem((size, size, size), VoxelId(0)),
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

impl Index<Vector3<u32>> for World3d {
    type Output = VoxelId;
    fn index(&self, index: Vector3<u32>) -> &Self::Output {
        let index = index.cast::<usize>();
        &self.voxels[[index.x, index.y, index.z]]
    }
}
impl IndexMut<Vector3<u32>> for World3d {
    fn index_mut(&mut self, index: Vector3<u32>) -> &mut Self::Output {
        let index = index.cast::<usize>();
        &mut self.voxels[[index.x, index.y, index.z]]
    }
}

#[derive(Debug)]
pub struct World3dTexture(pub Texture, pub Extent3d);
#[derive(Debug)]
pub struct World3dBindGroup(pub BindGroup, pub BindGroupLayout);

pub fn init_world_3d(mut commands: Commands, size: Res<WorldSize>, device: Res<Device>) {
    let size = size.0;

    let world_3d = World3d::new(size);

    let extent = Extent3d {
        width: size,
        height: size,
        depth_or_array_layers: size,
    };
    let texture = device.create_texture(&TextureDescriptor {
        label: None,
        size: extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D3,
        format: TextureFormat::R8Uint,
        usage: TextureUsage::COPY_DST | TextureUsage::SAMPLED | TextureUsage::RENDER_ATTACHMENT,
    });
    let view = texture.create_view(&TextureViewDescriptor::default());
    let sampler = device.create_sampler(&SamplerDescriptor {
        mag_filter: FilterMode::Nearest,
        min_filter: FilterMode::Nearest,
        mipmap_filter: FilterMode::Nearest,
        ..Default::default()
    });

    let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStage::FRAGMENT,
                ty: BindingType::Texture {
                    multisampled: false,
                    view_dimension: TextureViewDimension::D3,
                    sample_type: TextureSampleType::Uint,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStage::FRAGMENT,
                ty: BindingType::Sampler {
                    comparison: false,
                    filtering: false,
                },
                count: None,
            },
        ],
    });

    let bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });

    commands.insert_resource(world_3d);
    commands.insert_resource(World3dTexture(texture, extent));
    commands.insert_resource(World3dBindGroup(bind_group, bind_group_layout));
}

pub fn update_world_texture(
    world: Res<World3d>,
    queue: Res<Queue>,
    texture: Res<World3dTexture>,
    mut uniforms: ResMut<Uniforms>,
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
