#![allow(incomplete_features)]
#![feature(const_generics)]

use crate::camera3d::Camera3d;
use crate::camera3d::Camera3dPlugin;
use crate::render::init_render_pipeline;
use crate::render::render;
use crate::swap_chain::init_swap_chain;
use crate::swap_chain::update_swap_chain;
use crate::uniform::init_uniforms;
use crate::uniform::update_uniform_buffer;
use crate::voxel::VoxelType;
use crate::window_size::init_window_size;
use crate::window_size::update_window_size;
use crate::world::init_world_3d;
use crate::world::update_world_3d;
use crate::world::World;
use crate::world::WorldSize;
use bevy::diagnostic::{DiagnosticsPlugin, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use nalgebra::Vector3;
use palette::Srgb;

mod camera3d;
mod render;
mod swap_chain;
mod uniform;
mod voxel;
mod window_size;
mod world;

fn main() {
    let mut app = App::build();
    app.insert_resource(WindowDescriptor {
        title: "render-4d".to_string(),
        width: 500.0,
        height: 500.0,
        vsync: true,
        ..Default::default()
    })
    .insert_resource(WorldSize(5))
    .insert_resource(Camera3d::new(Vector3::new(4.0, 4.0, 4.0), 0.0));
    app.add_plugins(DefaultPlugins)
        .add_plugin(DiagnosticsPlugin)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin)
        .add_plugin(Camera3dPlugin);
    app.add_startup_stage_after(
        StartupStage::Startup,
        "startup-swap-chain",
        SystemStage::single_threaded(),
    )
    .add_startup_stage_after(
        "startup-swap-chain",
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
    app.add_startup_system(init_window_size.system())
        .add_startup_system_to_stage("startup-swap-chain", init_swap_chain.system())
        .add_startup_system_to_stage("startup-bind-groups", init_uniforms.system())
        .add_startup_system_to_stage("startup-bind-groups", init_world_3d.system())
        .add_startup_system_to_stage("startup-pipeline", init_render_pipeline.system())
        .add_startup_system_to_stage("startup-finish", init_world_data.system())
        .add_system(update_window_size.system().before("update-swap-chain"))
        .add_system(
            update_swap_chain
                .system()
                .label("update-swap-chain")
                .before("render")
                .before("update-uniforms"),
        )
        .add_system(
            update_world_3d
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

fn init_world_data(mut world: ResMut<World>) {
    let normal_type = world.insert_type(VoxelType::new(Srgb::new(0.212, 0.247, 0.278)));

    for i in 0..2 {
        for j in 0..2 {
            for k in 0..2 {
                world[Vector3::new(i, j, k)] = normal_type;
            }
        }
    }
}
