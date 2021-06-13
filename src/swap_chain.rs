use crate::window_size::WindowSize;
use crate::world::WorldSize;
use bevy::prelude::*;
use bevy::winit::WinitWindows;
use futures::executor::block_on;
use wgpu::*;

pub fn init_swap_chain(
    mut commands: Commands,
    winit_windows: Res<WinitWindows>,
    windows: Res<Windows>,
    window_size: Res<WindowSize>,
    world_size: Res<WorldSize>,
) {
    let window = winit_windows
        .get_window(windows.get_primary().unwrap().id())
        .unwrap();
    let instance = Instance::new(BackendBit::all());
    let surface = unsafe { instance.create_surface(window) };
    let adapter = block_on(instance.request_adapter(&RequestAdapterOptions {
        power_preference: PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
    }))
    .expect("Failed to find an appropriate adapter");

    let (device, queue) = block_on(adapter.request_device(
        &DeviceDescriptor {
            label: Some("device"),
            features: Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
            limits: Limits {
                max_texture_dimension_3d: (world_size.0 + 2) * (world_size.0 + 2),
                ..Default::default()
            },
        },
        None,
    ))
    .expect("Failed to create device");

    let sc_desc = SwapChainDescriptor {
        usage: TextureUsage::RENDER_ATTACHMENT,
        format: adapter.get_swap_chain_preferred_format(&surface).unwrap(),
        width: window_size.0.x,
        height: window_size.0.y,
        present_mode: PresentMode::Fifo,
    };
    let swap_chain = device.create_swap_chain(&surface, &sc_desc);

    commands.insert_resource(surface);
    commands.insert_resource(device);
    commands.insert_resource(queue);
    commands.insert_resource(sc_desc);
    commands.insert_resource(swap_chain);
}

pub fn update_swap_chain(
    window_size: Res<WindowSize>,
    surface: Res<Surface>,
    device: Res<Device>,
    mut sc_desc: ResMut<SwapChainDescriptor>,
    mut swap_chain: ResMut<SwapChain>,
) {
    if !window_size.is_changed() {
        return;
    }
    sc_desc.width = window_size.0.x;
    sc_desc.height = window_size.0.y;
    *swap_chain = device.create_swap_chain(&surface, &sc_desc);
}
