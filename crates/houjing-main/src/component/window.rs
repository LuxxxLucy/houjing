use crate::startup::window::WindowConfig;
use bevy::prelude::*;

pub struct WindowSetupPlugin;

impl Plugin for WindowSetupPlugin {
    fn build(&self, app: &mut App) {
        // Initialize the window config resource
        app.init_resource::<WindowConfig>();
    }
}
