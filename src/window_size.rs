use crate::uniform_3d::Uniforms;
use bevy::prelude::*;
use bevy::winit::WinitWindows;
use nalgebra::Vector2;
#[derive(Resource, Copy, Clone, Eq, PartialEq, Debug)]
pub struct WindowSize(pub Vector2<u32>);

fn get_window_size(winit_windows: &WinitWindows, windows: &Windows) -> WindowSize {
    let window = winit_windows
        .get_window(windows.get_primary().unwrap().id())
        .unwrap();
    let size = window.inner_size();
    WindowSize(Vector2::new(size.width, size.height))
}

pub fn init_window_size(
    mut commands: Commands,
    winit_windows: Res<WinitWindows>,
    windows: Res<Windows>,
) {
    commands.insert_resource(get_window_size(&winit_windows, &windows));
}

pub fn update_window_size(
    winit_windows: Res<WinitWindows>,
    windows: Res<Windows>,
    mut window_size: ResMut<WindowSize>,
    mut uniforms: ResMut<Uniforms>,
) {
    let size = get_window_size(&winit_windows, &windows);
    if size != *window_size {
        *window_size = size;
        uniforms.window_size = size.0.cast();
    }
}
