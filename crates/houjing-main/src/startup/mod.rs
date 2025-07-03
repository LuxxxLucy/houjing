pub mod log;
pub mod window;

use bevy::prelude::*;

pub fn add_startup_plugins(app: &mut App) {
    app.add_plugins(
        DefaultPlugins
            .set(log::get_log_plugin())
            .set(window::get_window_plugin()),
    );
}
