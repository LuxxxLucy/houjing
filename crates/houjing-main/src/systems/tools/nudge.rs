use super::common::selected::{SelectedControlPoint, move_selected_points};
use super::tool::{Tool, ToolState};
use crate::InputSet;
use crate::component::curve::Point;
use bevy::prelude::*;

#[derive(Resource)]
pub struct NudgeConfig {
    pub move_distance: f32,
}

impl Default for NudgeConfig {
    fn default() -> Self {
        Self { move_distance: 1.0 }
    }
}

#[derive(Resource, Default)]
pub struct NudgeToolState;

impl NudgeToolState {
    pub fn reset(&mut self, _commands: &mut Commands) {
        // Nudge tool doesn't maintain persistent state that needs cleanup
    }
}

pub struct NudgePlugin;

impl Plugin for NudgePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NudgeConfig>()
            .init_resource::<NudgeToolState>()
            .add_systems(Update, handle_nudge_input.in_set(InputSet));
    }
}

fn handle_nudge_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    config: Res<NudgeConfig>,
    tool_state: Res<ToolState>,
    mut nudge_state: ResMut<NudgeToolState>,
    selected_query: Query<&SelectedControlPoint>,
    mut point_query: Query<&mut Point>,
) {
    // Reset state if not using select tool (nudge only works with selection)
    if !tool_state.is_currently_using_tool(Tool::Select) {
        nudge_state.reset(&mut commands);
        return;
    }

    // Check if there are any selected points
    if selected_query.is_empty() {
        return;
    }

    // Determine movement direction
    let movement = if keyboard.pressed(KeyCode::ArrowUp) {
        Some(Vec2::new(0.0, config.move_distance))
    } else if keyboard.pressed(KeyCode::ArrowDown) {
        Some(Vec2::new(0.0, -config.move_distance))
    } else if keyboard.pressed(KeyCode::ArrowLeft) {
        Some(Vec2::new(-config.move_distance, 0.0))
    } else if keyboard.pressed(KeyCode::ArrowRight) {
        Some(Vec2::new(config.move_distance, 0.0))
    } else {
        None
    };

    if let Some(offset) = movement {
        // Move all selected points by the offset
        move_selected_points(&selected_query, &mut point_query, offset);
    }
}
