#![feature(div_duration)]

use crate::swap_chain::init_swap_chain;
use crate::swap_chain::update_swap_chain;
use crate::view::init_view;
use crate::view::ViewSize;
use crate::voxel::VoxelType;
use crate::window_size::init_window_size;
use crate::window_size::update_window_size;
use crate::world::init_world;
use crate::world::update_world;
use crate::world::World;
use crate::world::WorldSize;
use bevy::diagnostic::{DiagnosticsPlugin, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use nalgebra::Vector3;
use nalgebra::Vector4;
use palette::Srgb;

mod camera_3d;
mod camera_4d;
mod render_3d;
mod render_4d;
mod swap_chain;
mod uniform_3d;
mod uniform_4d;
mod utils;
mod view;
mod voxel;
mod window_size;
mod world;

fn main() {
    let mut app = App::build();
    app.insert_resource(WindowDescriptor {
        title: "render-4d".to_string(),
        width: 500.0,
        height: 500.0,
        vsync: false,
        ..Default::default()
    })
    .insert_resource(WorldSize(88))
    .insert_resource(ViewSize(128))
    .insert_resource(camera_3d::Camera::new(Vector3::new(4.0, 4.0, 4.0), 0.0))
    .insert_resource(camera_4d::Camera::new());
    app.add_plugins(DefaultPlugins)
        .add_plugin(DiagnosticsPlugin)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin)
        .add_plugin(camera_3d::CameraPlugin)
        .add_plugin(camera_4d::CameraPlugin);
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
        .add_startup_system_to_stage("startup-bind-groups", uniform_4d::init_uniforms.system())
        .add_startup_system_to_stage("startup-bind-groups", uniform_3d::init_uniforms.system())
        .add_startup_system_to_stage("startup-bind-groups", init_world.system())
        .add_startup_system_to_stage("startup-bind-groups", init_view.system())
        .add_startup_system_to_stage("startup-pipeline", render_4d::init_render_pipeline.system())
        .add_startup_system_to_stage("startup-pipeline", render_3d::init_render_pipeline.system())
        .add_startup_system_to_stage("startup-finish", init_world_data.system())
        .add_system(update_window_size.system().before("update-swap-chain"))
        .add_system(update_swap_chain.system().label("update-swap-chain"))
        .add_system(update_world.system().label("update-world"))
        .add_system(
            uniform_4d::update_uniform_buffer
                .system()
                .label("update-uniforms-4d")
                .after("camera-4d")
                .after("update-swap-chain"),
        )
        .add_system(
            uniform_3d::update_uniform_buffer
                .system()
                .label("update-uniforms-3d")
                .after("camera-3d")
                .after("update-swap-chain"),
        )
        .add_system(
            render_4d::render
                .system()
                .label("render-4d")
                .after("update-uniforms-4d")
                .after("update-world"),
        )
        .add_system(
            render_3d::render
                .system()
                .label("render-3d")
                .after("update-uniforms-3d")
                .after("render-4d"),
        );
    app.run();
}

fn init_world_data(mut world: ResMut<World>) {
    let normal_type = world.insert_type(VoxelType::new(Srgb::new(0.212, 0.247, 0.278)));

    for i in 8..30 {
        for j in 8..24 {
            for k in 8..13 {
                for l in 5..26 {
                    world[Vector4::new(i, j, k, l)] = normal_type;
                }
            }
        }
    }
}
