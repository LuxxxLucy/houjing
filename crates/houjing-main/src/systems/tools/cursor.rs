use crate::InputSet;
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct CursorWorldPos(pub Vec2);

#[derive(Resource, Default)]
pub struct CursorState {
    pub cursor_just_pressed: bool,
    pub dragging: bool,
    pub drag_start_pos: Vec2,
}

// Default cursor configuration constants
const DEFAULT_DRAG_THRESHOLD: f32 = 5.0;

#[derive(Resource)]
pub struct CursorConfig {
    pub drag_threshold: f32,
}

impl Default for CursorConfig {
    fn default() -> Self {
        Self {
            drag_threshold: DEFAULT_DRAG_THRESHOLD,
        }
    }
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
            .init_resource::<CursorConfig>()
            .init_resource::<CursorVisualizationConfig>()
            .add_systems(
                Update,
                (
                    update_cursor_world_position,
                    handle_cursor_input,
                    manage_cursor_visibility,
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
    mut input_state: ResMut<CursorState>,
    cursor_input: Res<ButtonInput<MouseButton>>,
    cursor_pos: Res<CursorWorldPos>,
    config: Res<CursorConfig>,
) {
    let just_pressed = cursor_input.just_pressed(MouseButton::Left);
    let pressed = cursor_input.pressed(MouseButton::Left);
    let just_released = cursor_input.just_released(MouseButton::Left);

    if just_pressed {
        input_state.drag_start_pos = cursor_pos.0;
        input_state.dragging = false;
    }

    if pressed
        && !input_state.dragging
        && cursor_pos.0.distance(input_state.drag_start_pos) > config.drag_threshold
    {
        input_state.dragging = true;
    }

    if just_released {
        input_state.dragging = false;
    }
    input_state.cursor_just_pressed = just_pressed;
}

fn debug_cursor_position(cursor_pos: Res<CursorWorldPos>, input_state: Res<CursorState>) {
    if input_state.cursor_just_pressed {
        debug!("Cursor clicked at: {:?}", cursor_pos.0);
    }
}

// disable system cursor when dragging (and instead show our dragging cursor)
fn manage_cursor_visibility(input_state: Res<CursorState>, mut windows: Query<&mut Window>) {
    if let Ok(mut window) = windows.get_single_mut() {
        // Hide system cursor when dragging, show it otherwise
        window.cursor.visible = !input_state.dragging;
    }
}
