use crate::surface::{DeviceResource, QueueResource, SurfaceConfigResource, SurfaceResource};
use crate::uniform_3d::UniformBindGroup;
use crate::utils::to_u32_array;
use crate::view::View3dBindGroup;
use bevy::prelude::*;
use bytemuck::cast_slice;
use std::borrow::Cow;
use wgpu::util::DeviceExt;
use wgpu::*;

#[rustfmt::skip]
const VERTICIES: &[f32] = &[
    -1.0, -1.0, 0.0,
    -1.0, 1.0,  0.0,
    1.0,  -1.0, 0.0,
    1.0,  1.0,  0.0,
    1.0, -1.0,  0.0,
    -1.0, 1.0,  0.0,
];

#[derive(Resource)]
pub struct Render3dPipeline(RenderPipeline);

#[derive(Resource)]
pub struct VertexBuffer(Buffer);

pub fn init_render_pipeline(
    mut commands: Commands,
    device: Res<DeviceResource>,
    uniform_bind_group: Res<UniformBindGroup>,
    view_bind_group: Res<View3dBindGroup>,
    surface_config: Res<SurfaceConfigResource>,
) {
    let vert = unsafe {
        device.create_shader_module_spirv(&ShaderModuleDescriptorSpirV {
            label: Some("vertex-3d"),
            source: Cow::Borrowed(&to_u32_array(include_bytes!("3d.vert.spv"))),
        })
    };
    let frag = unsafe {
        device.create_shader_module_spirv(&ShaderModuleDescriptorSpirV {
            label: Some("fragment-3d"),
            source: Cow::Borrowed(&to_u32_array(include_bytes!("3d.frag.spv"))),
        })
    };

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("vertex-buffer"),
        contents: cast_slice(VERTICIES),
        usage: BufferUsages::VERTEX,
    });

    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("render-3d-pipeline-layout"),
        bind_group_layouts: &[&uniform_bind_group.1, &view_bind_group.1],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("render-3d-pipeline"),
        layout: Some(&pipeline_layout),
        vertex: VertexState {
            module: &vert,
            entry_point: "main",
            buffers: &[VertexBufferLayout {
                array_stride: std::mem::size_of::<[f32; 3]>() as BufferAddress,
                step_mode: VertexStepMode::Vertex,
                attributes: &vertex_attr_array![0 => Float32x3],
            }],
        },
        fragment: Some(FragmentState {
            module: &frag,
            entry_point: "main",
            targets: &[Some(ColorTargetState {
                format: surface_config.format,
                blend: Some(BlendState::REPLACE),
                write_mask: ColorWrites::ALL,
            })],
        }),
        depth_stencil: None,
        primitive: Default::default(),
        multisample: Default::default(),
        multiview: None,
    });

    commands.insert_resource(Render3dPipeline(pipeline));
    commands.insert_resource(VertexBuffer(vertex_buffer));
}

pub fn render(
    device: Res<DeviceResource>,
    queue: Res<QueueResource>,
    surface: Res<SurfaceResource>,
    surface_config: Res<SurfaceConfigResource>,
    render_pipeline: Res<Render3dPipeline>,
    uniform_bind_group: Res<UniformBindGroup>,
    view_3d_bind_group: Res<View3dBindGroup>,
    vertex_buffer: Res<VertexBuffer>,
) {
    let vertex_buffer = &vertex_buffer.0;
    let frame = surface.get_current_texture().unwrap();
    let view = frame.texture.create_view(&TextureViewDescriptor {
        label: Some("surface-texture-view"),
        format: Some(surface_config.format),
        dimension: Some(TextureViewDimension::D2),
        aspect: TextureAspect::All,
        base_mip_level: 0,
        mip_level_count: None,
        base_array_layer: 0,
        array_layer_count: None,
    });
    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("render-3d-encoder"),
    });

    {
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("render-3d-pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });
        render_pass.set_pipeline(&render_pipeline.0);
        render_pass.set_bind_group(0, &uniform_bind_group.0, &[]);
        render_pass.set_bind_group(1, &view_3d_bind_group.0, &[]);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.draw(0..6, 0..1);
    }

    queue.submit(std::iter::once(encoder.finish()));
}
