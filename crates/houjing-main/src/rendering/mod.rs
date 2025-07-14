#![allow(dead_code)]

pub mod primitive;

use bevy::prelude::*;

// Re-export primitive functions for convenience
pub use primitive::{
    DashedLineConfig, render_animated_dashed_line, render_dashed_line, render_simple_circle,
    render_simple_rectangle,
};

pub struct ColorPalette {
    pub selection: Color,
    pub control_point: Color,
    pub creation_point: Color,
    pub drag_indicator: Color,
}

/// Common configuration constants for rendering
pub mod constants {
    use super::ColorPalette;
    use bevy::prelude::Color;

    pub const DEFAULT_Z_LAYER: f32 = 1.0;
    pub const SELECTION_Z_LAYER: f32 = 2.0;
    pub const UI_Z_LAYER: f32 = 3.0;

    pub const DEFAULT_POINT_RADIUS: f32 = 6.0;
    pub const DEFAULT_SELECTION_RADIUS: f32 = 15.0;

    pub const COLORS: ColorPalette = ColorPalette {
        selection: Color::YELLOW,
        control_point: Color::RED,
        creation_point: Color::BLUE,
        drag_indicator: Color::ORANGE,
    };
}
