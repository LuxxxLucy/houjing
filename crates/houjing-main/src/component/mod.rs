pub mod camera;
pub mod curve;
pub mod window;

use bevy::prelude::*;
use camera::CameraPlugin;
use curve::CurveRenderingPlugin;
use window::WindowSetupPlugin;

pub(crate) fn add_component_plugins(app: &mut App) {
    app.add_plugins((WindowSetupPlugin, CameraPlugin, CurveRenderingPlugin));
}
