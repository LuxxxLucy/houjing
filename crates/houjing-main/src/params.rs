// Window parameters
pub const WINDOW_TITLE: &str = "Houjing"; // Title displayed in the application window
pub const WINDOW_WIDTH: f32 = 1024.0; // Initial width of the application window in pixels
pub const WINDOW_HEIGHT: f32 = 768.0; // Initial height of the application window in pixels

// Colors for different UI elements
pub const CURVE_COLOR: bevy::prelude::Color = bevy::prelude::Color::WHITE; // Color of Bézier curves when rendered
pub const CONTROL_POINT_COLOR: bevy::prelude::Color = bevy::prelude::Color::RED; // Color of control points (unselected state)
pub const SELECTED_POINT_COLOR: bevy::prelude::Color = bevy::prelude::Color::YELLOW; // Color of control points when selected
pub const CREATION_POINT_COLOR: bevy::prelude::Color = bevy::prelude::Color::BLUE; // Color of temporary points shown during curve creation

// Sizes and radius for visual elements
pub const CONTROL_POINT_RADIUS: f32 = 8.0; // Radius of control point circles in pixels
pub const CREATION_POINT_RADIUS: f32 = 6.0; // Radius of creation point circles in pixels (smaller than control points)
pub const SELECTION_RADIUS: f32 = 15.0; // Distance in pixels within which clicking will select a control point
pub const DUPLICATE_POINT_THRESHOLD: f32 = 1.0; // Minimum distance between points to prevent accidental duplicates during creation

// Z-layers for rendering order (higher values render on top)
pub const CURVE_Z_LAYER: f32 = 0.0; // Z-coordinate for curves (rendered first/behind)
pub const CONTROL_POINT_Z_LAYER: f32 = 1.0; // Z-coordinate for control points (rendered above curves)
pub const CREATION_POINT_Z_LAYER: f32 = 2.0; // Z-coordinate for creation points (rendered on top)
