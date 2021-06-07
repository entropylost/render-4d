use crate::VertexBuffer;
use bevy::prelude::*;
use bevy::winit::WinitWindows;
use byte_slice_cast::AsByteSlice;
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
) {
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

    let sc_desc = SwapChainDescriptor {
        usage: TextureUsage::RENDER_ATTACHMENT,
        format: adapter.get_swap_chain_preferred_format(&surface).unwrap(),
        width: size.width,
        height: size.height,
        present_mode: PresentMode::Fifo,
    };
    let swap_chain = device.create_swap_chain(&surface, &sc_desc);

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: VERTICIES.as_byte_slice(),
        usage: BufferUsage::VERTEX,
    });

    commands.insert_resource(device);
    commands.insert_resource(queue);
    commands.insert_resource(sc_desc);
    commands.insert_resource(swap_chain);
    commands.insert_resource(VertexBuffer(vertex_buffer));
}
