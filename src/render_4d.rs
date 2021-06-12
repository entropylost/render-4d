use nalgebra::Vector3;
use crate::world::WorldSize;
use std::borrow::Cow;
use crate::utils::to_u32_array;
use crate::uniform_4d::UniformBindGroup;
use crate::world::WorldBindGroup;
use crate::view::View4dBindGroup;
use wgpu::*;
use bevy::prelude::*;

const LOCAL_WORKGROUP_SIZE: Vector3<u32> = Vector3::new(8, 8, 1);

pub fn init_render_pipeline(
    mut commands: Commands,
    device: Res<Device>,
    uniform_bind_group: Res<UniformBindGroup>,
    world_bind_group: Res<WorldBindGroup>,
    view_bind_group: Res<View4dBindGroup>,
) {
    let comp = device.create_shader_module(&ShaderModuleDescriptor {
        label: Some("compute-4d"),
        source: ShaderSource::SpirV(Cow::Borrowed(&to_u32_array(include_bytes!("4d.comp.spv")))),
        flags: ShaderFlags::empty(),
    });

    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("render-4d-pipeline-layout"),
        bind_group_layouts: &[&uniform_bind_group.1, &world_bind_group.1, &view_bind_group.1],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
        label: Some("render-4d-pipeline"),
        layout: Some(&pipeline_layout),
        module: &comp,
        entry_point: "main",
    });

    commands.insert_resource(pipeline);
}

pub fn render(
    world_size: Res<WorldSize>,
    device: Res<Device>,
    queue: Res<Queue>,
    render_pipeline: Res<ComputePipeline>,
    uniform_bind_group: Res<UniformBindGroup>,
    world_bind_group: Res<WorldBindGroup>,
    view_bind_group: Res<View4dBindGroup>,
) {
    device.poll(Maintain::Wait);

    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("render-4d-encoder"),
    });

    {
        let mut render_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("render-4d-pass"),
        });
        render_pass.set_pipeline(&render_pipeline);
        render_pass.set_bind_group(0, &uniform_bind_group.0, &[]);
        render_pass.set_bind_group(1, &world_bind_group.0, &[]);
        render_pass.set_bind_group(2, &view_bind_group.0, &[]);
        let workgroup_counts = Vector3::repeat(world_size.0).component_div(&LOCAL_WORKGROUP_SIZE);
        render_pass.dispatch(workgroup_counts.x, workgroup_counts.y, workgroup_counts.z);
    }

    queue.submit(std::iter::once(encoder.finish()));
}
