use crate::voxel::WorldSize;
use crate::VertexBuffer;
use bevy::prelude::*;
use bevy::winit::WinitWindows;
use std::borrow::Cow;
use byte_slice_cast::{AsByteSlice, AsSliceOf};
use futures::executor::block_on;
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

pub(crate) fn setup(
    mut commands: Commands,
    winit_windows: Res<WinitWindows>,
    windows: Res<Windows>,
    world_size: Res<WorldSize>,
) {
    let world_size = world_size.0;
    let window = winit_windows
        .get_window(windows.get_primary().unwrap().id())
        .unwrap();
    let size = window.inner_size();
    let instance = Instance::new(BackendBit::all());
    let surface = unsafe { instance.create_surface(window) };
    let adapter = block_on(instance.request_adapter(&RequestAdapterOptions {
        power_preference: PowerPreference::HighPerformance,
        // Request an adapter which can render to our surface
        compatible_surface: Some(&surface),
    }))
    .expect("Failed to find an appropriate adapter");

    let (device, queue) = block_on(adapter.request_device(
        &DeviceDescriptor {
            label: None,
            features: Features::empty(),
            limits: Limits::default(),
        },
        None,
    ))
    .expect("Failed to create device");

    let world_3d = device.create_texture(&TextureDescriptor {
        label: None,
        size: Extent3d {
            width: world_size,
            height: world_size,
            depth_or_array_layers: world_size,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D3,
        format: TextureFormat::R8Uint,
        usage: TextureUsage::COPY_DST | TextureUsage::SAMPLED | TextureUsage::RENDER_ATTACHMENT,
    });

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: VERTICIES.as_byte_slice(),
        usage: BufferUsage::VERTEX,
    });

    let sc_desc = SwapChainDescriptor {
        usage: TextureUsage::RENDER_ATTACHMENT,
        format: adapter.get_swap_chain_preferred_format(&surface).unwrap(),
        width: size.width,
        height: size.height,
        present_mode: PresentMode::Fifo,
    };
    let swap_chain = device.create_swap_chain(&surface, &sc_desc);

    let shader_vert = device.create_shader_module(&ShaderModuleDescriptor {
        label: None,
        source: ShaderSource::SpirV(Cow::Borrowed(
            include_bytes!("shader.vert.spv")
                .as_slice_of::<u32>()
                .unwrap(),
        )),
        flags: ShaderFlags::all(),
    });
    let shader_frag = device.create_shader_module(&ShaderModuleDescriptor {
        label: None,
        source: ShaderSource::SpirV(Cow::Borrowed(
            include_bytes!("shader.frag.spv")
                .as_slice_of::<u32>()
                .unwrap(),
        )),
        flags: ShaderFlags::all(),
    });

    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: VertexState {
            module: &shader_vert,
            entry_point: "main",
            buffers: &[VertexBufferLayout {
                array_stride: std::mem::size_of::<[f32; 3]>() as BufferAddress,
                step_mode: InputStepMode::Vertex,
                attributes: &vertex_attr_array![0 => Float32x3],
            }],
        },
        fragment: Some(FragmentState {
            module: &shader_frag,
            entry_point: "main",
            targets: &[sc_desc.format.into()],
        }),
        depth_stencil: None,
        primitive: Default::default(),
        multisample: Default::default(),
    });

    commands.insert_resource(device);
    commands.insert_resource(queue);
    commands.insert_resource(swap_chain);
    commands.insert_resource(render_pipeline);
    commands.insert_resource(VertexBuffer(vertex_buffer));
}
