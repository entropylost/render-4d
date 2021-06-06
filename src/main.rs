#![allow(incomplete_features)]
#![feature(const_generics)]

use bevy::prelude::*;
use wgpu::Buffer;

mod voxel;
use voxel::*;
mod setup;
use setup::*;
mod render;
use render::*;

struct VertexBuffer(Buffer);

fn main() {
    let world = World3d::new(100);

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
    app.add_plugins(DefaultPlugins);
    app
        .add_startup_system(setup.system())
        .add_system(render.system());
    app.run();
}
