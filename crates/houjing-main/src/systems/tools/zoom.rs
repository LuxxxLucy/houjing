use super::tool::{Tool, ToolState};
use crate::InputSet;
use bevy::prelude::*;
use log::debug;

// Default zoom configuration constants
const DEFAULT_KEYBOARD_ZOOM_FACTOR: f32 = 1.1;
const MIN_ZOOM: f32 = 0.1;
const MAX_ZOOM: f32 = 10.0;

#[derive(Resource)]
pub struct ZoomConfig {
    pub keyboard_zoom_factor: f32,
    pub min_zoom: f32,
    pub max_zoom: f32,
}

impl Default for ZoomConfig {
    fn default() -> Self {
        Self {
            keyboard_zoom_factor: DEFAULT_KEYBOARD_ZOOM_FACTOR,
            min_zoom: MIN_ZOOM,
            max_zoom: MAX_ZOOM,
        }
    }
}

#[derive(Resource, Default)]
pub struct ZoomToolState;

impl ZoomToolState {
    pub fn reset(&mut self, _commands: &mut Commands) {
        // Zoom tool doesn't maintain persistent state that needs cleanup
    }
}

pub struct ZoomPlugin;

impl Plugin for ZoomPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ZoomConfig>()
            .init_resource::<ZoomToolState>()
            .add_systems(
                Update,
                (handle_zoom_input, update_zoom_cursor).in_set(InputSet),
            );
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_zoom_input(
    mut zoom_state: ResMut<ZoomToolState>,
    mut camera_query: Query<(&mut Transform, &Camera, &GlobalTransform), With<Camera2d>>,
    tool_state: Res<ToolState>,
    config: Res<ZoomConfig>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    mut commands: Commands,
) {
    // Zoom functionality works globally, regardless of active tool
    // Only reset state if not using zoom tool
    if !tool_state.is_currently_using_tool(Tool::Zoom) {
        zoom_state.reset(&mut commands);
    }

    let window = windows.single();
    let (mut camera_transform, camera, camera_global_transform) = camera_query.single_mut();

    // Handle keyboard zoom (Ctrl + and Ctrl -)
    let ctrl_pressed =
        keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);

    if ctrl_pressed {
        let zoom_factor =
            if keyboard.just_pressed(KeyCode::Equal) || keyboard.just_pressed(KeyCode::NumpadAdd) {
                // Zoom in
                debug!("Keyboard zoom in");
                Some(config.keyboard_zoom_factor)
            } else if keyboard.just_pressed(KeyCode::Minus)
                || keyboard.just_pressed(KeyCode::NumpadSubtract)
            {
                // Zoom out
                debug!("Keyboard zoom out");
                Some(1.0 / config.keyboard_zoom_factor)
            } else {
                None
            };

        if let Some(factor) = zoom_factor {
            apply_zoom(&mut camera_transform, factor, None, &config);
        }
    }

    // Handle mouse click zoom when in zoom tool mode
    if tool_state.is_currently_using_tool(Tool::Zoom) {
        let zoom_factor = if mouse_input.just_pressed(MouseButton::Left) {
            // Left click = zoom in
            debug!("Mouse left click zoom in");
            Some(config.keyboard_zoom_factor)
        } else if mouse_input.just_pressed(MouseButton::Right) {
            // Right click = zoom out
            debug!("Mouse right click zoom out");
            Some(1.0 / config.keyboard_zoom_factor)
        } else {
            None
        };

        if let Some(factor) = zoom_factor {
            // Get cursor position for zoom-to-cursor behavior
            let zoom_center = window.cursor_position().and_then(|cursor_pos| {
                camera.viewport_to_world_2d(camera_global_transform, cursor_pos)
            });
            apply_zoom(&mut camera_transform, factor, zoom_center, &config);
        }
    }
}

fn apply_zoom(
    camera_transform: &mut Transform,
    zoom_factor: f32,
    zoom_center: Option<Vec2>,
    config: &ZoomConfig,
) {
    // Calculate new scale
    let current_scale = camera_transform.scale.x;
    let new_scale = (current_scale / zoom_factor).clamp(config.min_zoom, config.max_zoom);

    // If zoom would exceed limits, don't apply it
    if (new_scale - current_scale).abs() < f32::EPSILON {
        return;
    }

    // If we have a zoom center, adjust camera position to zoom towards that point
    if let Some(world_center) = zoom_center {
        let camera_pos = camera_transform.translation.truncate();
        let offset_to_center = world_center - camera_pos;
        let scale_change = new_scale / current_scale;
        let adjusted_offset = offset_to_center * (1.0 - scale_change);

        camera_transform.translation.x += adjusted_offset.x;
        camera_transform.translation.y += adjusted_offset.y;
    }

    // Apply the new scale
    camera_transform.scale = Vec3::splat(new_scale);

    debug!("Applied zoom: scale {current_scale} -> {new_scale}");
}

fn update_zoom_cursor(tool_state: Res<ToolState>, mut windows: Query<&mut Window>) {
    if let Ok(mut window) = windows.get_single_mut() {
        if tool_state.is_currently_using_tool(Tool::Zoom) {
            // Use zoom cursor when in zoom mode
            window.cursor.icon = CursorIcon::ZoomIn;
        } else {
            // Reset to default cursor for other tools (if not overridden by other tools)
            if window.cursor.icon == CursorIcon::ZoomIn {
                window.cursor.icon = CursorIcon::Default;
            }
        }
    }
}
