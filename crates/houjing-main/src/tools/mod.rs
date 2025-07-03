pub mod curve_create;
pub mod select;

use bevy::prelude::*;

pub(crate) struct ToolPlugin;

impl Plugin for ToolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ToolState>();
    }
}

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

use curve_create::CurveCreationPlugin;
use select::SelectionPlugin;

pub(crate) fn add_tools_plugins(app: &mut App) {
    app.add_plugins((ToolPlugin, SelectionPlugin, CurveCreationPlugin));
}
