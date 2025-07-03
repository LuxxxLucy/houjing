use bevy::prelude::*;

// Default window configuration constants
const DEFAULT_WINDOW_TITLE: &str = "Houjing";
const DEFAULT_WINDOW_WIDTH: f32 = 1024.0;
const DEFAULT_WINDOW_HEIGHT: f32 = 768.0;

#[derive(Resource)]
pub struct WindowConfig {
    pub title: String,
    pub width: f32,
    pub height: f32,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: DEFAULT_WINDOW_TITLE.to_string(),
            width: DEFAULT_WINDOW_WIDTH,
            height: DEFAULT_WINDOW_HEIGHT,
        }
    }
}

pub fn get_window_plugin() -> bevy::window::WindowPlugin {
    let config = WindowConfig::default();
    bevy::window::WindowPlugin {
        primary_window: Some(Window {
            title: config.title.clone(),
            resolution: (config.width, config.height).into(),
            ..default()
        }),
        ..default()
    }
}
