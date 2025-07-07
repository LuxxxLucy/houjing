use super::cursor::*;
use super::tool::{Tool, ToolState};
use crate::InputSet;
use bevy::prelude::*;
use log::debug;

// Minimum movement threshold to prevent micro-jitter
const MIN_MOVEMENT_THRESHOLD: f32 = 0.1;

#[derive(Resource, Default)]
pub struct HandToolState {
    pub is_panning: bool,
    pub last_screen_pos: Option<Vec2>,
}

impl HandToolState {
    pub fn reset(&mut self, _commands: &mut Commands) {
        self.is_panning = false;
        self.last_screen_pos = None;
    }
}

// Default hand configuration constants
const DEFAULT_HAND_SENSITIVITY: f32 = 1.0;

#[derive(Resource)]
pub struct HandConfig {
    pub hand_sensitivity: f32,
}

impl Default for HandConfig {
    fn default() -> Self {
        Self {
            hand_sensitivity: DEFAULT_HAND_SENSITIVITY,
        }
    }
}

pub struct HandPlugin;

impl Plugin for HandPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HandConfig>()
            .init_resource::<HandToolState>()
            .add_systems(
                Update,
                (handle_hand_input, update_hand_cursor).in_set(InputSet),
            );
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_hand_input(
    mut hand_state: ResMut<HandToolState>,
    mut camera_query: Query<(&mut Transform, &Camera, &GlobalTransform), With<Camera2d>>,
    cursor_state: Res<CursorState>,
    tool_state: Res<ToolState>,
    config: Res<HandConfig>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    mut commands: Commands,
) {
    // Check if tool is active, reset state if not
    if !tool_state.is_currently_using_tool(Tool::Hand) {
        hand_state.reset(&mut commands);
        return;
    }

    let window = windows.single();
    let (mut camera_transform, _camera, camera_global_transform) = camera_query.single_mut();

    // Get current screen position
    let current_screen_pos = window.cursor_position();

    if cursor_state.cursor_just_pressed {
        if let Some(screen_pos) = current_screen_pos {
            hand_state.is_panning = true;
            hand_state.last_screen_pos = Some(screen_pos);
            debug!("Started panning at screen pos: {screen_pos:?}");
        }
    }

    if mouse_input.pressed(MouseButton::Left) && hand_state.is_panning {
        if let (Some(current_screen), Some(last_screen)) =
            (current_screen_pos, hand_state.last_screen_pos)
        {
            // Calculate screen space delta
            let screen_delta = last_screen - current_screen;

            // Only update if movement is significant enough (prevents micro-jitter)
            if screen_delta.length() > MIN_MOVEMENT_THRESHOLD {
                // Convert screen delta to world space delta using camera scale
                let camera_scale = camera_global_transform.compute_transform().scale.x;
                let world_delta = screen_delta / camera_scale * config.hand_sensitivity;

                // Apply camera movement (much more stable than using world coordinates)
                camera_transform.translation += Vec3::new(world_delta.x, -world_delta.y, 0.0);

                // Update last position
                hand_state.last_screen_pos = Some(current_screen);
            }
        }
    }

    if mouse_input.just_released(MouseButton::Left) {
        hand_state.is_panning = false;
        hand_state.last_screen_pos = None;
        debug!("Stopped panning");
    }
}

fn update_hand_cursor(
    tool_state: Res<ToolState>,
    hand_state: Res<HandToolState>,
    mut windows: Query<&mut Window>,
) {
    if let Ok(mut window) = windows.get_single_mut() {
        if tool_state.is_currently_using_tool(Tool::Hand) {
            if hand_state.is_panning {
                // When actively panning, use grabbing cursor
                window.cursor.icon = CursorIcon::Grabbing;
            } else {
                // When in hand mode but not actively panning, use grab cursor
                window.cursor.icon = CursorIcon::Grab;
            }
        } else {
            // Reset to default cursor for other tools
            window.cursor.icon = CursorIcon::Default;
        }
    }
}
