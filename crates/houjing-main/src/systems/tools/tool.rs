use crate::InputSet;
use bevy::prelude::*;

#[derive(Resource, Default)]
pub(crate) struct ToolState {
    current_tool: Tool,
    pub previous_tool: Tool,
    pub is_space_held: bool,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) enum Tool {
    #[default]
    Select,
    Split,
    CreateCurve,
    Hand,
    Zoom,
    Merge,
}

impl ToolState {
    /// Check if currently using the specified tool
    pub fn is_currently_using_tool(&self, tool: Tool) -> bool {
        self.current_tool == tool
    }

    /// Switch to a specific tool
    pub fn switch_to(&mut self, tool: Tool) {
        self.current_tool = tool;
    }

    /// Get the current tool
    pub fn current(&self) -> Tool {
        self.current_tool.clone()
    }
}

pub(crate) struct ToolPlugin;

impl Plugin for ToolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ToolState>();
        app.add_systems(Update, (handle_tool_switching,).before(InputSet));
    }
}

fn handle_tool_switching(mut tool_state: ResMut<ToolState>, keyboard: Res<ButtonInput<KeyCode>>) {
    // Handle spacebar temporary hand mode
    if keyboard.just_pressed(KeyCode::Space) && !tool_state.is_space_held {
        tool_state.previous_tool = tool_state.current_tool.clone();
        tool_state.switch_to(Tool::Hand);
        tool_state.is_space_held = true;
        return;
    }

    if keyboard.just_released(KeyCode::Space) && tool_state.is_space_held {
        let previous = tool_state.previous_tool.clone();
        tool_state.switch_to(previous);
        tool_state.is_space_held = false;
        return;
    }

    // Handle permanent tool switching (only when not in temporary hand mode)
    if !tool_state.is_space_held {
        let mut new_tool = None;
        if keyboard.just_pressed(KeyCode::KeyV) {
            new_tool = Some(Tool::Select)
        }

        if keyboard.just_pressed(KeyCode::KeyS) {
            new_tool = Some(Tool::Split)
        }

        if keyboard.just_pressed(KeyCode::KeyC) {
            new_tool = Some(Tool::CreateCurve)
        }

        if keyboard.just_pressed(KeyCode::KeyG) {
            new_tool = Some(Tool::Hand)
        }

        if keyboard.just_pressed(KeyCode::KeyZ) {
            new_tool = Some(Tool::Zoom)
        }

        if keyboard.just_pressed(KeyCode::KeyM) {
            new_tool = Some(Tool::Merge)
        }

        if let Some(new_tool) = new_tool {
            tool_state.switch_to(new_tool);
        }
    }
}
