mod startup;
mod tools;
mod ui;

use bevy::prelude::*;

use crate::systems::startup::add_startup_plugins;
use crate::systems::ui::add_ui_plugins;
use crate::{EditSet, InputSet, ShowSet};
use tools::add_tools_plugins;

pub(crate) fn add_systems_plugins(app: &mut App) {
    // Configure system sets to run in order: Input → Edit → Show
    app.configure_sets(Update, (InputSet, EditSet, ShowSet).chain());

    add_startup_plugins(app);
    add_ui_plugins(app);
    add_tools_plugins(app);
}
