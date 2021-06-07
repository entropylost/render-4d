#![allow(incomplete_features)]
#![feature(const_generics)]

use crate::render::init_render_pipeline;
use crate::render::render;
use crate::setup::setup;
use crate::uniform::init_uniforms;
use crate::uniform::update_uniform_buffer;
use crate::voxel::VoxelType;
use crate::world::init_world_3d;
use crate::world::update_world_texture;
use crate::world::World3d;
use crate::world::WorldSize;
use bevy::diagnostic::{DiagnosticsPlugin, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use nalgebra::Vector3;
use palette::Srgba;
use wgpu::Buffer;

mod player;
mod render;
mod setup;
mod uniform;
mod voxel;
mod world;

pub struct VertexBuffer(Buffer);

fn main() {
    let mut app = App::build();
    app.insert_resource(WindowDescriptor {
        title: "render-4d".to_string(),
        width: 500.0,
        height: 500.0,
        vsync: true,
        ..Default::default()
    })
    .insert_resource(WorldSize(5));
    app.add_plugins(DefaultPlugins)
        .add_plugin(DiagnosticsPlugin)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin);
    app.add_startup_stage_after(
        StartupStage::Startup,
        "startup-bind-groups",
        SystemStage::single_threaded(),
    )
    .add_startup_stage_after(
        "startup-bind-groups",
        "startup-pipeline",
        SystemStage::single_threaded(),
    )
    .add_startup_stage_after(
        "startup-pipeline",
        "startup-finish",
        SystemStage::single_threaded(),
    );
    app.add_startup_system(setup.system())
        .add_startup_system_to_stage("startup-bind-groups", init_uniforms.system())
        .add_startup_system_to_stage("startup-bind-groups", init_world_3d.system())
        .add_startup_system_to_stage("startup-pipeline", init_render_pipeline.system())
        .add_startup_system_to_stage("startup-finish", init_world_data.system())
        .add_system(
            update_world_texture
                .system()
                .before("render")
                .before("update-uniforms"),
        )
        .add_system(
            update_uniform_buffer
                .system()
                .label("update-uniforms")
                .before("render"),
        )
        .add_system(render.system().label("render"));
    app.run();
}

fn init_world_data(mut world: ResMut<World3d>) {
    let normal_type = world.insert_type(VoxelType::new(Srgba::new(0.212, 0.247, 0.278, 1.0)));

    for i in 0..2 {
        for j in 0..2 {
            for k in 3..5 {
                world[Vector3::new(i, j, k)] = normal_type;
            }
        }
    }
}
