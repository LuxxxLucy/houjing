mod startup;
mod tools;
mod ui;

use bevy::prelude::*;

use crate::systems::startup::add_startup_plugins;
use crate::systems::ui::add_ui_plugins;
use tools::add_tools_plugins;

pub(crate) fn add_systems_plugins(app: &mut App) {
    add_startup_plugins(app);
    add_ui_plugins(app);
    add_tools_plugins(app);
}
