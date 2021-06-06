use crate::VertexBuffer;
use bevy::prelude::*;
use wgpu::*;

pub(crate) fn render(
    device: Res<Device>,
    queue: Res<Queue>,
    swap_chain: Res<SwapChain>,
    render_pipeline: Res<RenderPipeline>,
    bind_group: Res<BindGroup>,
    vertex_buffer: Res<VertexBuffer>,
) {
    let vertex_buffer = &vertex_buffer.0;
    let frame = swap_chain.get_current_frame().unwrap().output;
    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("Render Encoder"),
    });

    {
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Render Pass"),
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
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.draw(0..6, 0..1);
    }

    queue.submit(std::iter::once(encoder.finish()));
}
