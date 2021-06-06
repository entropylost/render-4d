#![allow(incomplete_features)]
#![feature(const_generics)]

use bevy::diagnostic::{DiagnosticsPlugin, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use nalgebra::Vector3;
use palette::Srgba;
use wgpu::Buffer;

mod voxel;
use voxel::*;
mod setup;
use setup::*;
mod render;
use render::*;

struct VertexBuffer(Buffer);

fn main() {
    let mut world = World3d::new(5);
    world[Vector3::new(1, 1, 1)] =
        world.insert_type(VoxelType::new(Srgba::new(0.212, 0.247, 0.278, 1.0)));

    let mut app = App::build();
    app.insert_resource(WindowDescriptor {
        title: "render-4d".to_string(),
        width: 500.0,
        height: 500.0,
        vsync: true,
        ..Default::default()
    })
    .insert_resource(WorldSize(world.size()))
    .insert_resource(world);
    app.add_plugins(DefaultPlugins)
        .add_plugin(DiagnosticsPlugin)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin);
    app.add_startup_system(setup.system())
        .add_system(render.system())
        .add_system(update_world_texture.system());
    app.run();
}
