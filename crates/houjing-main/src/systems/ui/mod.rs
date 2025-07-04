mod camera;

use bevy::prelude::*;

use camera::CameraPlugin;

pub(crate) fn add_ui_plugins(app: &mut App) {
    app.add_plugins(CameraPlugin);
}
