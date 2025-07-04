use crate::InputSet;
use bevy::prelude::*;

#[derive(Resource, Default)]
pub(crate) struct ToolState {
    pub current_tool: Tool,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) enum Tool {
    #[default]
    Select,
    CreateCurve,
}

pub(crate) struct ToolPlugin;

impl Plugin for ToolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ToolState>();
        app.add_systems(Update, (handle_tool_switching,).in_set(InputSet));
    }
}

fn handle_tool_switching(mut tool_state: ResMut<ToolState>, keyboard: Res<ButtonInput<KeyCode>>) {
    let mut new_tool = None;
    if keyboard.just_pressed(KeyCode::KeyS) {
        new_tool = Some(Tool::Select)
    }

    if keyboard.just_pressed(KeyCode::KeyC) {
        new_tool = Some(Tool::CreateCurve)
    }

    if let Some(new_tool) = new_tool {
        tool_state.current_tool = new_tool;
    }
}
