use super::common::selected::{SelectedControlPoint, move_selected_points};
use super::cursor::{CursorState, CursorVisualizationConfig};
use super::select::SelectionToolState;
use super::tool::{Tool, ToolState};
use crate::component::curve::Point;
use crate::rendering::{render_dashed_line, render_simple_rectangle};
use crate::{InputSet, ShowSet};
use bevy::prelude::*;
use bevy::sprite::ColorMaterial;

// Animation parameters for drag selection wireframe
const DASH_LENGTH: f32 = 6.0;
const GAP_LENGTH: f32 = 4.0;
const ANIMATION_SPEED: f32 = 40.0; // pixels per second

// Visual element sizes
const DRAG_START_INDICATOR_RADIUS: f32 = 4.0;

// Drag behavior configuration constants
const DEFAULT_DRAG_THRESHOLD: f32 = 5.0;

// Default curve visualization during drag configuration constants
const DEFAULT_DRAG_CURVE_COLOR: Color = Color::ORANGE;
const DEFAULT_DRAG_START_INDICATOR_ALPHA: f32 = 0.5;

// Default drag selection rectangle configuration constants
const DEFAULT_DRAG_SELECTION_COLOR: Color = Color::ORANGE;
const DEFAULT_DRAG_SELECTION_BACKGROUND_ALPHA: f32 = 0.1;
const DEFAULT_DRAG_SELECTION_WIREFRAME_ALPHA: f32 = 0.8;

#[derive(Resource)]
pub struct DragConfig {
    pub drag_threshold: f32,
}

impl Default for DragConfig {
    fn default() -> Self {
        Self {
            drag_threshold: DEFAULT_DRAG_THRESHOLD,
        }
    }
}

#[derive(Resource)]
pub struct DragCurveVisualizationConfig {
    pub drag_color: Color,
    pub start_indicator_alpha: f32,
}

impl Default for DragCurveVisualizationConfig {
    fn default() -> Self {
        Self {
            drag_color: DEFAULT_DRAG_CURVE_COLOR,
            start_indicator_alpha: DEFAULT_DRAG_START_INDICATOR_ALPHA,
        }
    }
}

#[derive(Resource)]
pub struct DragSelectionRectangleConfig {
    pub drag_color: Color,
    pub background_alpha: f32,
    pub wireframe_alpha: f32,
}

impl Default for DragSelectionRectangleConfig {
    fn default() -> Self {
        Self {
            drag_color: DEFAULT_DRAG_SELECTION_COLOR,
            background_alpha: DEFAULT_DRAG_SELECTION_BACKGROUND_ALPHA,
            wireframe_alpha: DEFAULT_DRAG_SELECTION_WIREFRAME_ALPHA,
        }
    }
}

#[derive(Resource, Default)]
pub struct DragToolState {
    /// Drag state when there is no selected point
    pub rectangle: NoSelectedPointDragState,
    /// Drag state when there is a selected point
    pub selected_points: SelectedPointDragState,
}

impl DragToolState {
    #[allow(dead_code)]
    pub fn reset(&mut self, commands: &mut Commands) {
        self.rectangle.reset(commands);
        self.selected_points.reset(commands);
    }
}

#[derive(Default)]
pub struct SelectedPointDragState {
    /// Drag start position
    pub drag_start: Option<Vec2>,
    /// Previous cursor position for calculating delta
    pub previous_cursor_position: Option<Vec2>,
    /// Whether point dragging is active
    pub is_active: bool,
    /// Whether we've exceeded the drag threshold (started actual dragging)
    pub is_dragging: bool,
}

impl SelectedPointDragState {
    fn reset(&mut self, _commands: &mut Commands) {
        self.drag_start = None;
        self.previous_cursor_position = None;
        self.is_active = false;
        self.is_dragging = false;
    }
}

#[derive(Default)]
pub struct NoSelectedPointDragState {
    /// Entity of the rectangle mesh
    pub entity: Option<Entity>,
    /// Current rectangle
    pub rect: Option<DragRect>,
}

impl NoSelectedPointDragState {
    fn reset(&mut self, commands: &mut Commands) {
        if let Some(entity) = self.entity {
            commands.entity(entity).despawn();
        }
        self.entity = None;
        self.rect = None;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DragRect {
    pub origin: Vec2,
    pub width: f32,
    pub height: f32,
}

#[derive(Component)]
pub struct NoSelectedPointDragRectangle;

pub struct DragPlugin;

impl Plugin for DragPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DragToolState>()
            .init_resource::<DragConfig>()
            .init_resource::<DragCurveVisualizationConfig>()
            .init_resource::<DragSelectionRectangleConfig>()
            .add_systems(
                Update,
                (
                    handle_selected_point_drag_state,
                    handle_no_selected_point_drag_state,
                )
                    .in_set(InputSet),
            )
            .add_systems(
                Update,
                (render_selected_point_drag, render_no_selected_point_drag).in_set(ShowSet),
            );
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_selected_point_drag_state(
    cursor_state: Res<CursorState>,
    mut commands: Commands,
    selection_state: Res<SelectionToolState>,
    mut drag_state: ResMut<DragToolState>,
    tool_state: Res<ToolState>,
    drag_config: Res<DragConfig>,
    selected_query: Query<&SelectedControlPoint>,
    mut point_query: Query<&mut Point>,
) {
    // Only handle drag when we have selected points and are using select tool
    if !tool_state.is_currently_using_tool(Tool::Select)
        || selection_state.selected_points.is_empty()
    {
        drag_state.selected_points.reset(&mut commands);
        return;
    }

    if cursor_state.mouse_just_pressed {
        // Start drag: store original positions
        drag_state.selected_points.reset(&mut commands);
        drag_state.selected_points.drag_start = Some(cursor_state.cursor_position);
        drag_state.selected_points.previous_cursor_position = Some(cursor_state.cursor_position);

        drag_state.selected_points.is_active = true;
        drag_state.selected_points.is_dragging = false;
    }

    if cursor_state.mouse_pressed && drag_state.selected_points.is_active {
        // Check if we've exceeded the drag threshold
        if let Some(drag_start) = drag_state.selected_points.drag_start {
            let distance = cursor_state.cursor_position.distance(drag_start);
            if !drag_state.selected_points.is_dragging && distance > drag_config.drag_threshold {
                drag_state.selected_points.is_dragging = true;
            }

            // Continue drag: update point positions whenever mouse is held down and we're dragging
            if drag_state.selected_points.is_dragging {
                if let Some(previous_pos) = drag_state.selected_points.previous_cursor_position {
                    let delta = cursor_state.cursor_position - previous_pos;
                    if delta.length() > 0.0 {
                        // Move all selected points by the cursor delta
                        move_selected_points(&selected_query, &mut point_query, delta);
                    }
                }
                // Update previous cursor position for next frame
                drag_state.selected_points.previous_cursor_position =
                    Some(cursor_state.cursor_position);
            }
        }
    } else if cursor_state.mouse_just_released && drag_state.selected_points.is_active {
        // End drag when mouse is released
        drag_state.reset(&mut commands);
    }
}

fn handle_no_selected_point_drag_state(
    cursor_state: Res<CursorState>,
    selection_state: Res<SelectionToolState>,
    mut drag_state: ResMut<DragToolState>,
    tool_state: Res<ToolState>,
    mut commands: Commands,
) {
    // Only handle rectangle drag when no points are selected and using select tool
    if !tool_state.is_currently_using_tool(Tool::Select)
        || !selection_state.selected_points.is_empty()
    {
        drag_state.rectangle.reset(&mut commands);
        return;
    }

    if cursor_state.mouse_just_pressed {
        // Start rectangle selection
        drag_state.rectangle.rect = Some(DragRect {
            origin: cursor_state.cursor_position,
            width: 0.0,
            height: 0.0,
        });
    } else if cursor_state.mouse_pressed {
        // Update rectangle while mouse is held down
        if let Some(ref mut rect) = drag_state.rectangle.rect {
            let delta = cursor_state.cursor_position - rect.origin;
            rect.width = delta.x;
            rect.height = delta.y;
        }
    } else if cursor_state.mouse_just_released && drag_state.rectangle.rect.is_some() {
        // End rectangle selection when mouse is released
        // TODO: Implement point selection within rectangle
        drag_state.rectangle.reset(&mut commands);
    }
}

#[allow(clippy::too_many_arguments)]
fn render_selected_point_drag(
    mut gizmos: Gizmos,
    cursor_state: Res<CursorState>,
    config: Res<CursorVisualizationConfig>,
    drag_config: Res<DragCurveVisualizationConfig>,
    selection_state: Res<SelectionToolState>,
    drag_state: Res<DragToolState>,
    tool_state: Res<ToolState>,
) {
    if !tool_state.is_currently_using_tool(Tool::Select)
        || selection_state.selected_points.is_empty()
    {
        return;
    }

    // Render diamond cursor when dragging selected points
    if drag_state.selected_points.is_dragging {
        render_diamond_cursor(&mut gizmos, cursor_state.cursor_position, &config);
    }

    // Render drag start indicator
    if let Some(drag_start) = drag_state.selected_points.drag_start {
        render_drag_start_indicator(&mut gizmos, drag_start, &drag_config);
    }

    fn render_diamond_cursor(
        gizmos: &mut Gizmos,
        cursor_pos: Vec2,
        config: &CursorVisualizationConfig,
    ) {
        let half_size = config.cursor_size / 2.0;
        let top = cursor_pos + Vec2::new(0.0, half_size);
        let right = cursor_pos + Vec2::new(half_size, 0.0);
        let bottom = cursor_pos + Vec2::new(0.0, -half_size);
        let left = cursor_pos + Vec2::new(-half_size, 0.0);

        gizmos.line_2d(top, right, config.drag_color);
        gizmos.line_2d(right, bottom, config.drag_color);
        gizmos.line_2d(bottom, left, config.drag_color);
        gizmos.line_2d(left, top, config.drag_color);
    }

    fn render_drag_start_indicator(
        gizmos: &mut Gizmos,
        start_pos: Vec2,
        drag_config: &DragCurveVisualizationConfig,
    ) {
        let color = Color::rgba(
            drag_config.drag_color.r(),
            drag_config.drag_color.g(),
            drag_config.drag_color.b(),
            drag_config.start_indicator_alpha,
        );
        gizmos.circle_2d(start_pos, DRAG_START_INDICATOR_RADIUS, color);
    }
}

#[allow(clippy::too_many_arguments)]
fn render_no_selected_point_drag(
    mut commands: Commands,
    mut gizmos: Gizmos,
    drag_selection_config: Res<DragSelectionRectangleConfig>,
    time: Res<Time>,
    mut drag_state: ResMut<DragToolState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut no_selected_point_drag_query: Query<&mut Transform, With<NoSelectedPointDragRectangle>>,
    tool_state: Res<ToolState>,
) {
    if !tool_state.is_currently_using_tool(Tool::Select) {
        return;
    }

    if let Some(rect) = drag_state.rectangle.rect {
        // Render rectangle fill
        render_no_selected_point_drag_fill(
            &mut commands,
            rect,
            &drag_selection_config,
            &mut drag_state.rectangle,
            &mut meshes,
            &mut materials,
            &mut no_selected_point_drag_query,
        );

        // Render animated wireframe
        render_no_selected_point_drag_wireframe(&mut gizmos, rect, &drag_selection_config, &time);
    }
}

fn render_no_selected_point_drag_fill(
    commands: &mut Commands,
    no_selected_point_drag_rect: DragRect,
    drag_selection_config: &DragSelectionRectangleConfig,
    no_selected_point_drag_state: &mut NoSelectedPointDragState,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    no_selected_point_drag_query: &mut Query<&mut Transform, With<NoSelectedPointDragRectangle>>,
) {
    // Create or update rectangle entity
    let rect_center = no_selected_point_drag_rect.origin
        + Vec2::new(
            no_selected_point_drag_rect.width / 2.0,
            no_selected_point_drag_rect.height / 2.0,
        );

    if let Some(entity) = no_selected_point_drag_state.entity {
        // Update existing rectangle
        if let Ok(mut transform) = no_selected_point_drag_query.get_mut(entity) {
            transform.translation = rect_center.extend(1.0);
            transform.scale = Vec3::new(
                no_selected_point_drag_rect.width.abs(),
                no_selected_point_drag_rect.height.abs(),
                1.0,
            );
        }
    } else {
        // Create new rectangle
        let entity = render_simple_rectangle(
            commands,
            meshes,
            materials,
            rect_center,
            Vec2::new(
                no_selected_point_drag_rect.width.abs(),
                no_selected_point_drag_rect.height.abs(),
            ),
            Color::rgba(
                drag_selection_config.drag_color.r(),
                drag_selection_config.drag_color.g(),
                drag_selection_config.drag_color.b(),
                drag_selection_config.background_alpha,
            ),
            1.0,
        );
        commands.entity(entity).insert(NoSelectedPointDragRectangle);
        no_selected_point_drag_state.entity = Some(entity);
    }
}

fn render_no_selected_point_drag_wireframe(
    gizmos: &mut Gizmos,
    no_selected_point_drag_rect: DragRect,
    drag_selection_config: &DragSelectionRectangleConfig,
    time: &Time,
) {
    let color = Color::rgba(
        drag_selection_config.drag_color.r(),
        drag_selection_config.drag_color.g(),
        drag_selection_config.drag_color.b(),
        drag_selection_config.wireframe_alpha,
    );

    // Calculate rectangle corners
    let min = no_selected_point_drag_rect.origin;
    let max = no_selected_point_drag_rect.origin
        + Vec2::new(
            no_selected_point_drag_rect.width,
            no_selected_point_drag_rect.height,
        );

    // Draw animated dashed lines for each edge
    let elapsed = time.elapsed_seconds();
    let dash_offset = (elapsed * ANIMATION_SPEED) % (DASH_LENGTH + GAP_LENGTH);

    // Top edge
    render_dashed_line(
        gizmos,
        Vec2::new(min.x, max.y),
        Vec2::new(max.x, max.y),
        color,
        DASH_LENGTH,
        GAP_LENGTH,
        dash_offset,
    );
    // Right edge
    render_dashed_line(
        gizmos,
        Vec2::new(max.x, max.y),
        Vec2::new(max.x, min.y),
        color,
        DASH_LENGTH,
        GAP_LENGTH,
        dash_offset,
    );
    // Bottom edge
    render_dashed_line(
        gizmos,
        Vec2::new(max.x, min.y),
        Vec2::new(min.x, min.y),
        color,
        DASH_LENGTH,
        GAP_LENGTH,
        dash_offset,
    );
    // Left edge
    render_dashed_line(
        gizmos,
        Vec2::new(min.x, min.y),
        Vec2::new(min.x, max.y),
        color,
        DASH_LENGTH,
        GAP_LENGTH,
        dash_offset,
    );
}
