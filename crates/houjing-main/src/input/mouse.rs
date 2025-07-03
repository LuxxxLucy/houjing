use crate::InputSet;
use bevy::prelude::*;
use log::debug;

// Default mouse configuration constants
const DEFAULT_DRAG_THRESHOLD: f32 = 5.0;

#[derive(Resource)]
pub struct MouseConfig {
    pub drag_threshold: f32,
}

impl Default for MouseConfig {
    fn default() -> Self {
        Self {
            drag_threshold: DEFAULT_DRAG_THRESHOLD,
        }
    }
}

#[derive(Resource, Default)]
pub struct MouseWorldPos(pub Vec2);

#[derive(Resource, Default)]
pub struct MouseState {
    pub mouse_pressed: bool,
    pub mouse_just_pressed: bool,
    pub dragging: bool,
    pub drag_start_pos: Vec2,
}

pub struct MousePlugin;

impl Plugin for MousePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MouseWorldPos>()
            .init_resource::<MouseState>()
            .init_resource::<MouseConfig>()
            .add_systems(
                Update,
                (
                    update_mouse_world_position,
                    handle_mouse_input,
                    debug_mouse_position,
                )
                    .in_set(InputSet),
            );
    }
}

fn debug_mouse_position(mouse_pos: Res<MouseWorldPos>, input_state: Res<MouseState>) {
    if input_state.mouse_just_pressed {
        debug!("Mouse clicked at: {:?}", mouse_pos.0);
    }
}

fn update_mouse_world_position(
    mut mouse_world_pos: ResMut<MouseWorldPos>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
) {
    let window = windows.single();
    let (camera, camera_transform) = camera_q.single();

    if let Some(cursor_pos) = window.cursor_position() {
        if let Some(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
            mouse_world_pos.0 = world_pos;
        }
    }
}

fn handle_mouse_input(
    mut input_state: ResMut<MouseState>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mouse_pos: Res<MouseWorldPos>,
    config: Res<MouseConfig>,
) {
    let just_pressed = mouse_input.just_pressed(MouseButton::Left);
    let pressed = mouse_input.pressed(MouseButton::Left);
    let just_released = mouse_input.just_released(MouseButton::Left);

    if just_pressed {
        input_state.drag_start_pos = mouse_pos.0;
        input_state.dragging = false;
    }

    if pressed
        && !input_state.dragging
        && mouse_pos.0.distance(input_state.drag_start_pos) > config.drag_threshold
    {
        input_state.dragging = true;
    }

    if just_released {
        input_state.dragging = false;
    }

    input_state.mouse_just_pressed = just_pressed;
    input_state.mouse_pressed = pressed;
}
