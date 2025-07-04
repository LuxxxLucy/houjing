pub mod curve;

use bevy::prelude::*;
use curve::CurveRenderingPlugin;

pub(crate) fn add_component_plugins(app: &mut App) {
    app.add_plugins(CurveRenderingPlugin);
}
