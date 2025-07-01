use bevy::prelude::*;
use log::debug;

pub mod create;
pub mod select;

#[derive(Resource, Default)]
pub struct ToolState {
    pub current_tool: Tool,
    pub creation_points: Vec<Vec2>,
    pub creation_state: CreationState,
    pub last_point: Option<Vec2>,
}

impl ToolState {
    pub fn reset(&mut self) {
        self.creation_points.clear();
        self.creation_state = CreationState::Idle;
        self.last_point = None;
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum Tool {
    #[default]
    Select,
    Create,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum CreationState {
    #[default]
    Idle,
    CollectingPoints,
}

pub fn handle_tool_switching(
    mut tool_state: ResMut<ToolState>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::KeyS) {
        tool_state.current_tool = Tool::Select;
        tool_state.reset();
        debug!("Switched to Select tool");
    }

    if keyboard.just_pressed(KeyCode::KeyC) {
        tool_state.current_tool = Tool::Create;
        tool_state.reset();
        debug!("Switched to Create tool - State reset");
    }
}
