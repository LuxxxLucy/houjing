pub mod keyboard;
pub mod mouse;

use bevy::prelude::*;
use keyboard::KeyboardPlugin;
use mouse::MousePlugin;

pub(crate) fn add_input_plugins(app: &mut App) {
    app.add_plugins((MousePlugin, KeyboardPlugin));
}
