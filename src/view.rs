use crate::surface::DeviceResource;
use bevy::prelude::*;
use wgpu::*;

#[derive(Resource, Copy, Clone, Debug, PartialEq, Eq)]
pub struct ViewSize(pub u32);

#[derive(Resource)]
pub struct ViewTexture(pub Texture, pub Extent3d);
#[derive(Resource)]
pub struct ViewDepthTexture(pub Texture, pub Extent3d);
#[derive(Resource)]
pub struct View3dBindGroup(pub BindGroup, pub BindGroupLayout);
#[derive(Resource)]
pub struct View4dBindGroup(pub BindGroup, pub BindGroupLayout);

pub fn init_view(mut commands: Commands, size: Res<ViewSize>, device: Res<DeviceResource>) {
    let size = size.0;

    let extent = Extent3d {
        width: size,
        height: size,
        depth_or_array_layers: size,
    };

    let texture = device.create_texture(&TextureDescriptor {
        label: Some("view-texture"),
        size: extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D3,
        format: TextureFormat::R8Uint,
        usage: TextureUsages::STORAGE_BINDING | TextureUsages::COPY_DST,
    });
    let view = texture.create_view(&TextureViewDescriptor::default());
    let sampler = device.create_sampler(&SamplerDescriptor {
        mag_filter: FilterMode::Nearest,
        min_filter: FilterMode::Nearest,
        mipmap_filter: FilterMode::Nearest,
        ..Default::default()
    });

    let depth_texture = device.create_texture(&TextureDescriptor {
        label: Some("view-depth-texture"),
        size: extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D3,
        format: TextureFormat::R8Uint,
        usage: TextureUsages::STORAGE_BINDING,
    });
    let depth_view = depth_texture.create_view(&TextureViewDescriptor::default());

    let bind_group_layout_3d = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("view-3d-bind-group-layout"),
        entries: &[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    multisampled: false,
                    view_dimension: TextureViewDimension::D3,
                    sample_type: TextureSampleType::Uint,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                count: None,
            },
        ],
    });

    let bind_group_3d = device.create_bind_group(&BindGroupDescriptor {
        label: Some("view-3d-bind-group"),
        layout: &bind_group_layout_3d,
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

    let bind_group_layout_4d = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("view-4d-bind-group-layout"),
        entries: &[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::StorageTexture {
                    access: StorageTextureAccess::WriteOnly,
                    format: TextureFormat::R8Uint,
                    view_dimension: TextureViewDimension::D3,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::StorageTexture {
                    access: StorageTextureAccess::WriteOnly,
                    format: TextureFormat::R8Uint,
                    view_dimension: TextureViewDimension::D3,
                },
                count: None,
            },
        ],
    });

    let bind_group_4d = device.create_bind_group(&BindGroupDescriptor {
        label: Some("view-4d-bind-group"),
        layout: &bind_group_layout_4d,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&view),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::TextureView(&depth_view),
            },
        ],
    });

    commands.insert_resource(ViewTexture(texture, extent));
    commands.insert_resource(ViewDepthTexture(depth_texture, extent));
    commands.insert_resource(View3dBindGroup(bind_group_3d, bind_group_layout_3d));
    commands.insert_resource(View4dBindGroup(bind_group_4d, bind_group_layout_4d));
}
