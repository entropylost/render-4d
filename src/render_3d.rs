use crate::utils::to_u32_array;
use crate::uniform_3d::UniformBindGroup;
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

pub struct VertexBuffer(Buffer);

pub fn init_render_pipeline(
    mut commands: Commands,
    device: Res<Device>,
    sc_desc: Res<SwapChainDescriptor>,
    uniform_bind_group: Res<UniformBindGroup>,
    view_bind_group: Res<View3dBindGroup>,
) {
    let vert = device.create_shader_module(&ShaderModuleDescriptor {
        label: Some("vertex-3d"),
        source: ShaderSource::SpirV(Cow::Borrowed(&to_u32_array(include_bytes!("3d.vert.spv")))),
        flags: ShaderFlags::all(),
    });
    let frag = device.create_shader_module(&ShaderModuleDescriptor {
        label: Some("fragment-3d"),
        source: ShaderSource::SpirV(Cow::Borrowed(&to_u32_array(include_bytes!("3d.frag.spv")))),
        flags: ShaderFlags::all(),
    });

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("vertex-buffer"),
        contents: cast_slice(VERTICIES),
        usage: BufferUsage::VERTEX,
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
                step_mode: InputStepMode::Vertex,
                attributes: &vertex_attr_array![0 => Float32x3],
            }],
        },
        fragment: Some(FragmentState {
            module: &frag,
            entry_point: "main",
            targets: &[sc_desc.format.into()],
        }),
        depth_stencil: None,
        primitive: Default::default(),
        multisample: Default::default(),
    });

    commands.insert_resource(pipeline);
    commands.insert_resource(VertexBuffer(vertex_buffer));
}

pub fn render(
    device: Res<Device>,
    queue: Res<Queue>,
    swap_chain: Res<SwapChain>,
    render_pipeline: Res<RenderPipeline>,
    uniform_bind_group: Res<UniformBindGroup>,
    view_3d_bind_group: Res<View3dBindGroup>,
    vertex_buffer: Res<VertexBuffer>,
) {
    let vertex_buffer = &vertex_buffer.0;
    let frame = swap_chain.get_current_frame().unwrap().output;
    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("render-3d-encoder"),
    });

    {
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("render-3d-pass"),
            color_attachments: &[RenderPassColorAttachment {
                view: &frame.view,
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
            }],
            depth_stencil_attachment: None,
        });
        render_pass.set_pipeline(&render_pipeline);
        render_pass.set_bind_group(0, &uniform_bind_group.0, &[]);
        render_pass.set_bind_group(1, &view_3d_bind_group.0, &[]);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.draw(0..6, 0..1);
    }

    queue.submit(std::iter::once(encoder.finish()));
}
