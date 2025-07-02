// src/main.rs
use bevy::prelude::*;
use log::debug;

use houjing_main::components::*;
use houjing_main::input::*;
use houjing_main::selection::*;
use houjing_main::systems::*;
use houjing_main::tools::create::{handle_curve_creation, render_creation_points};
use houjing_main::tools::*;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: houjing_main::params::WINDOW_TITLE.into(),
                    resolution: (
                        houjing_main::params::WINDOW_WIDTH,
                        houjing_main::params::WINDOW_HEIGHT,
                    )
                        .into(),
                    ..default()
                }),
                ..default()
            }),
        )
        .init_resource::<MouseWorldPos>()
        .init_resource::<InputState>()
        .init_resource::<ToolState>()
        .add_systems(Startup, (setup_camera, setup_test_curve))
        .add_systems(
            Update,
            (
                update_mouse_world_position,
                handle_mouse_input,
                handle_tool_switching,
                render_curves,
                render_control_points,
                render_creation_points,
                handle_point_selection,
                handle_point_dragging,
                handle_curve_creation,
                update_curve_meshes,
                debug_mouse_position,
            ),
        )
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn setup_test_curve(mut commands: Commands) {
    let curve = BezierCurve::new(vec![
        Vec2::new(-200.0, 0.0),
        Vec2::new(0.0, 200.0),
        Vec2::new(200.0, 0.0),
    ]);

    commands.spawn(curve);
}

fn debug_mouse_position(mouse_pos: Res<MouseWorldPos>, input_state: Res<InputState>) {
    if input_state.mouse_just_pressed {
        debug!("Mouse clicked at: {:?}", mouse_pos.0);
    }
}
