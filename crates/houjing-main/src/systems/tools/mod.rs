mod common;
mod cursor;
mod curve_create;
mod drag;
mod hand;
mod merge;
mod nudge;
mod select;
mod split;
mod tool;
mod zoom;

use bevy::prelude::*;
use cursor::CursorPlugin;
use curve_create::CurveCreationPlugin;
use drag::DragPlugin;
use hand::HandPlugin;
use merge::MergePlugin;
use nudge::NudgePlugin;
use select::SelectionPlugin;
use split::SplitPlugin;
use tool::ToolPlugin;
use zoom::ZoomPlugin;

pub(crate) fn add_tools_plugins(app: &mut App) {
    app.add_plugins((
        ToolPlugin,
        CursorPlugin,
        SelectionPlugin,
        SplitPlugin,
        DragPlugin,
        NudgePlugin,
        CurveCreationPlugin,
        HandPlugin,
        ZoomPlugin,
        MergePlugin,
    ));
}
