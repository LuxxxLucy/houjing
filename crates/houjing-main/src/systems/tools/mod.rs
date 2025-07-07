mod cursor;
mod curve_create;
mod drag;
mod hand;
mod select;
mod tool;

use bevy::prelude::*;
use cursor::CursorPlugin;
use curve_create::CurveCreationPlugin;
use drag::DragPlugin;
use hand::HandPlugin;
use select::SelectionPlugin;
use tool::ToolPlugin;

pub(crate) fn add_tools_plugins(app: &mut App) {
    app.add_plugins((
        ToolPlugin,
        CursorPlugin,
        SelectionPlugin,
        DragPlugin,
        CurveCreationPlugin,
        HandPlugin,
    ));
}
