use bevy::prelude::*;

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
    let mut new_tool = None;
    if keyboard.just_pressed(KeyCode::KeyS) {
        new_tool = Some(Tool::Select)
    }

    if keyboard.just_pressed(KeyCode::KeyC) {
        new_tool = Some(Tool::Create)
    }

    if let Some(new_tool) = new_tool {
        tool_state.current_tool = new_tool;
        tool_state.reset();
    }
}
