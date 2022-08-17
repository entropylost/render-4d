use crate::world::WorldSize;
use bevy::prelude::*;
use bevy::winit::WinitWindows;
use futures::executor::block_on;
use std::ops::{Deref, DerefMut};
use wgpu::*;

#[derive(Resource)]
pub struct DeviceResource(pub Device);
impl Deref for DeviceResource {
    type Target = Device;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for DeviceResource {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Resource)]
pub struct QueueResource(pub Queue);
impl Deref for QueueResource {
    type Target = Queue;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for QueueResource {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Resource)]
pub struct SurfaceResource(pub Surface);
impl Deref for SurfaceResource {
    type Target = Surface;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for SurfaceResource {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Resource)]
pub struct SurfaceFormat(pub TextureFormat);

pub fn init_surface(
    mut commands: Commands,
    winit_windows: NonSend<WinitWindows>,
    windows: Res<Windows>,
    world_size: Res<WorldSize>,
) {
    let window = winit_windows
        .get_window(windows.get_primary().unwrap().id())
        .unwrap();
    let instance = Instance::new(Backends::all());
    let surface = unsafe { instance.create_surface(window) };
    let adapter = block_on(instance.request_adapter(&RequestAdapterOptions {
        power_preference: PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
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

    commands.insert_resource(SurfaceFormat(surface.get_supported_formats(&adapter)[0]));
    commands.insert_resource(SurfaceResource(surface));
    commands.insert_resource(DeviceResource(device));
    commands.insert_resource(QueueResource(queue));
}
