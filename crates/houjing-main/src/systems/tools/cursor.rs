use crate::InputSet;
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct CursorWorldPos(pub Vec2);

#[derive(Resource, Default)]
pub struct CursorState {
    pub mouse_just_pressed: bool,
    pub cursor_position: Vec2,
    pub mouse_pressed: bool,
    pub mouse_just_released: bool,
}

// Default cursor visualization configuration constants
const DEFAULT_DRAG_COLOR: Color = Color::ORANGE;
const DEFAULT_CURSOR_SIZE: f32 = 8.0;

#[derive(Resource)]
pub struct CursorVisualizationConfig {
    pub drag_color: Color,
    pub cursor_size: f32,
}

impl Default for CursorVisualizationConfig {
    fn default() -> Self {
        Self {
            drag_color: DEFAULT_DRAG_COLOR,
            cursor_size: DEFAULT_CURSOR_SIZE,
        }
    }
}
pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorWorldPos>()
            .init_resource::<CursorState>()
            .init_resource::<CursorVisualizationConfig>()
            .add_systems(
                Update,
                (
                    update_cursor_world_position,
                    handle_cursor_input,
                    debug_cursor_position,
                )
                    .in_set(InputSet),
            );
    }
}

fn update_cursor_world_position(
    mut cursor_world_pos: ResMut<CursorWorldPos>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
) {
    let window = windows.single();
    let (camera, camera_transform) = camera_q.single();

    if let Some(cursor_pos) = window.cursor_position() {
        if let Some(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
            cursor_world_pos.0 = world_pos;
        }
    }
}

fn handle_cursor_input(
    mut cursor_state: ResMut<CursorState>,
    cursor_input: Res<ButtonInput<MouseButton>>,
    cursor_pos: Res<CursorWorldPos>,
) {
    let just_pressed = cursor_input.just_pressed(MouseButton::Left);
    let pressed = cursor_input.pressed(MouseButton::Left);
    let just_released = cursor_input.just_released(MouseButton::Left);

    cursor_state.cursor_position = cursor_pos.0;
    cursor_state.mouse_pressed = pressed;
    cursor_state.mouse_just_released = just_released;
    cursor_state.mouse_just_pressed = just_pressed;
}

fn debug_cursor_position(cursor_pos: Res<CursorWorldPos>, cursor_state: Res<CursorState>) {
    if cursor_state.mouse_just_pressed {
        debug!(
            "Cursor at: {:?}, pressed: {}, just_pressed: {}",
            cursor_pos.0, cursor_state.mouse_pressed, cursor_state.mouse_just_pressed
        );
    }
}
