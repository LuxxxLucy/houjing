use super::common::selected::SelectedControlPoint;
use super::cursor::{CursorState, CursorWorldPos};
use super::tool::{Tool, ToolState};
use crate::component::curve::{BezierCurve, CurveNeedsUpdate};
use crate::rendering::render_simple_rectangle;
use crate::systems::tools::cursor::CursorVisualizationConfig;
use crate::{InputSet, ShowSet};
use bevy::prelude::*;
use bevy::sprite::ColorMaterial;
use std::collections::HashMap;

// Animation parameters for drag selection wireframe
const DASH_LENGTH: f32 = 6.0;
const GAP_LENGTH: f32 = 4.0;
const ANIMATION_SPEED: f32 = 40.0; // pixels per second

// Visual element sizes
const DRAG_START_INDICATOR_RADIUS: f32 = 4.0;

// Default curve visualization during drag configuration constants
const DEFAULT_DRAG_CURVE_COLOR: Color = Color::ORANGE;
const DEFAULT_DRAG_ORIGINAL_CURVE_ALPHA: f32 = 0.3;
const DEFAULT_DRAG_START_INDICATOR_ALPHA: f32 = 0.5;

// Default drag selection rectangle configuration constants
const DEFAULT_DRAG_SELECTION_COLOR: Color = Color::ORANGE;
const DEFAULT_DRAG_SELECTION_BACKGROUND_ALPHA: f32 = 0.1;
const DEFAULT_DRAG_SELECTION_WIREFRAME_ALPHA: f32 = 0.8;

#[derive(Resource)]
pub struct DragCurveVisualizationConfig {
    pub drag_color: Color,
    pub original_curve_alpha: f32,
    pub start_indicator_alpha: f32,
}

impl Default for DragCurveVisualizationConfig {
    fn default() -> Self {
        Self {
            drag_color: DEFAULT_DRAG_CURVE_COLOR,
            original_curve_alpha: DEFAULT_DRAG_ORIGINAL_CURVE_ALPHA,
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
    pub fn reset(&mut self, commands: &mut Commands) {
        self.selected_points.reset(commands);
        self.rectangle.reset(commands);
    }
}

#[derive(Default)]
pub struct SelectedPointDragState {
    /// Original curve states before dragging
    pub original_curves: HashMap<Entity, Vec<Vec2>>,
    /// Current positions of selected points during drag
    pub current_positions: HashMap<(Entity, usize), Vec2>,
    /// Whether point dragging is active
    pub is_active: bool,
}

impl SelectedPointDragState {
    /// Reset the selected point drag state
    fn reset(&mut self, _commands: &mut Commands) {
        self.original_curves.clear();
        self.current_positions.clear();
        self.is_active = false;
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
    /// Reset the no selected point drag state
    fn reset(&mut self, commands: &mut Commands) {
        // Clear entity if it exists
        if let Some(entity) = self.entity {
            commands.entity(entity).despawn();
        }
        self.entity = None;
        self.rect = None;
    }
}

/// Rectangle for drag selection
#[derive(Clone, Copy)]
pub struct DragRect {
    pub origin: Vec2,
    pub width: f32,
    pub height: f32,
}

/// Component marker for no selected point drag rectangle entity
#[derive(Component)]
pub struct NoSelectedPointDragRectangle;

pub struct DragPlugin;

impl Plugin for DragPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DragToolState>()
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

fn handle_selected_point_drag_state(
    cursor_state: Res<CursorState>,
    cursor_pos: Res<CursorWorldPos>,
    mut commands: Commands,
    selected_query: Query<&SelectedControlPoint>,
    mut curve_query: Query<&mut BezierCurve>,
    mut drag_state: ResMut<DragToolState>,
    tool_state: Res<ToolState>,
) {
    // Check if we should disable drag (when in hand mode), reset state if so
    if tool_state.is_currently_using_tool(Tool::Hand) {
        drag_state.reset(&mut commands);
        return;
    }

    let has_selected_points = !selected_query.is_empty();
    let is_dragging = cursor_state.dragging && has_selected_points;

    if is_dragging {
        // Initialize original curves if not already done
        if drag_state.selected_points.original_curves.is_empty() {
            debug!(
                "Capturing original curve states for {} selected points",
                selected_query.iter().count()
            );
            for selected_point in selected_query.iter() {
                if let Ok(curve) = curve_query.get(selected_point.curve_entity) {
                    debug!(
                        "Storing original state for curve entity {:?} with {} control points",
                        selected_point.curve_entity,
                        curve.control_points.len()
                    );
                    drag_state
                        .selected_points
                        .original_curves
                        .insert(selected_point.curve_entity, curve.control_points.clone());
                }
            }
            debug!(
                "Total curves stored: {}",
                drag_state.selected_points.original_curves.len()
            );
        }

        // Update current positions for selected points
        for selected_point in selected_query.iter() {
            let key = (selected_point.curve_entity, selected_point.point_index);
            drag_state
                .selected_points
                .current_positions
                .insert(key, cursor_pos.0);
        }

        drag_state.selected_points.is_active = true;

        // Update curve points using current positions from drag state
        for selected_point in selected_query.iter() {
            if let Ok(mut curve) = curve_query.get_mut(selected_point.curve_entity) {
                if let Some(point) = curve.control_points.get_mut(selected_point.point_index) {
                    let key = (selected_point.curve_entity, selected_point.point_index);
                    if let Some(&current_pos) =
                        drag_state.selected_points.current_positions.get(&key)
                    {
                        *point = current_pos;

                        // Mark curve for mesh update
                        commands
                            .entity(selected_point.curve_entity)
                            .insert(CurveNeedsUpdate);
                    }
                }
            }
        }
    } else {
        // Clear state when not dragging
        drag_state.selected_points.original_curves.clear();
        drag_state.selected_points.current_positions.clear();
        drag_state.selected_points.is_active = false;
    }
}

fn handle_no_selected_point_drag_state(
    cursor_state: Res<CursorState>,
    cursor_pos: Res<CursorWorldPos>,
    selected_query: Query<&SelectedControlPoint>,
    mut drag_state: ResMut<DragToolState>,
    tool_state: Res<ToolState>,
    mut commands: Commands,
) {
    // Check if we should disable drag (when in hand mode), reset state if so
    if tool_state.is_currently_using_tool(Tool::Hand) {
        drag_state.reset(&mut commands);
        return;
    }

    let should_have_no_selected_point_drag = cursor_state.dragging && selected_query.is_empty();

    if should_have_no_selected_point_drag {
        // Calculate no selected point drag rectangle
        let start = cursor_state.drag_start_pos;
        let end = cursor_pos.0;
        let min_x = start.x.min(end.x);
        let max_x = start.x.max(end.x);
        let min_y = start.y.min(end.y);
        let max_y = start.y.max(end.y);

        drag_state.rectangle.rect = Some(DragRect {
            origin: Vec2::new(min_x, min_y),
            width: max_x - min_x,
            height: max_y - min_y,
        });
    } else {
        drag_state.rectangle.rect = None;
    }
}

fn render_selected_point_drag(
    mut gizmos: Gizmos,
    cursor_state: Res<CursorState>,
    config: Res<CursorVisualizationConfig>,
    drag_config: Res<DragCurveVisualizationConfig>,
    selected_query: Query<&SelectedControlPoint>,
    drag_state: Res<DragToolState>,
    tool_state: Res<ToolState>,
) {
    // Don't render when in hand mode
    if tool_state.is_currently_using_tool(Tool::Hand) {
        return;
    }

    // Only show when dragging is active
    if !drag_state.selected_points.is_active {
        return;
    }

    fn render_diamond_cursor(
        gizmos: &mut Gizmos,
        cursor_pos: Vec2,
        config: &CursorVisualizationConfig,
    ) {
        let color = config.drag_color;
        let half_size = config.cursor_size / 2.0;

        // Draw diamond shape for drag cursor
        let corners = [
            cursor_pos + Vec2::new(0.0, half_size),  // top
            cursor_pos + Vec2::new(half_size, 0.0),  // right
            cursor_pos + Vec2::new(0.0, -half_size), // bottom
            cursor_pos + Vec2::new(-half_size, 0.0), // left
        ];

        for i in 0..4 {
            gizmos.line_2d(corners[i], corners[(i + 1) % 4], color);
        }
    }

    fn render_drag_start_indicator(
        gizmos: &mut Gizmos,
        start_pos: Vec2,
        drag_config: &DragCurveVisualizationConfig,
    ) {
        gizmos.circle_2d(
            start_pos,
            DRAG_START_INDICATOR_RADIUS,
            drag_config
                .drag_color
                .with_a(drag_config.start_indicator_alpha),
        );
    }

    fn render_original_curve(
        gizmos: &mut Gizmos,
        control_points: &[Vec2],
        drag_config: &DragCurveVisualizationConfig,
    ) {
        if control_points.len() < 2 {
            return;
        }

        // Create a temporary curve for evaluation
        let temp_curve = BezierCurve {
            control_points: control_points.to_vec(),
        };

        // Render original curve as simple low-opacity line
        let samples = 100;

        for i in 0..samples {
            let t1 = i as f32 / samples as f32;
            let t2 = (i + 1) as f32 / samples as f32;
            let p1 = temp_curve.evaluate(t1);
            let p2 = temp_curve.evaluate(t2);

            // Draw with low opacity to distinguish from current curve
            gizmos.line_2d(
                p1,
                p2,
                drag_config
                    .drag_color
                    .with_a(drag_config.original_curve_alpha),
            );
        }
    }

    // Render original curves as low-opacity lines
    if !drag_state.selected_points.original_curves.is_empty() {
        for original_curve_points in drag_state.selected_points.original_curves.values() {
            render_original_curve(&mut gizmos, original_curve_points, &drag_config);
        }
    }

    // Draw diamond cursors at current positions of selected points
    for selected_point in selected_query.iter() {
        let key = (selected_point.curve_entity, selected_point.point_index);
        if let Some(&current_pos) = drag_state.selected_points.current_positions.get(&key) {
            render_diamond_cursor(&mut gizmos, current_pos, &config);
        }
    }

    // Draw drag start position indicator
    render_drag_start_indicator(&mut gizmos, cursor_state.drag_start_pos, &drag_config);
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
    // Don't render when in hand mode
    if tool_state.is_currently_using_tool(Tool::Hand) {
        // Clean up any existing rectangle when switching to hand mode
        if let Some(entity) = drag_state.rectangle.entity {
            commands.entity(entity).despawn();
            drag_state.rectangle.entity = None;
        }
        return;
    }

    if let Some(no_selected_point_drag_rect) = drag_state.rectangle.rect {
        // Render no selected point drag filled background
        render_no_selected_point_drag_fill(
            &mut commands,
            no_selected_point_drag_rect,
            &drag_selection_config,
            &mut drag_state.rectangle,
            &mut meshes,
            &mut materials,
            &mut no_selected_point_drag_query,
        );

        // Render no selected point drag animated wireframe
        render_no_selected_point_drag_wireframe(
            &mut gizmos,
            no_selected_point_drag_rect,
            &drag_selection_config,
            &time,
        );
    } else if let Some(entity) = drag_state.rectangle.entity {
        // Remove no selected point drag rectangle when not dragging
        commands.entity(entity).despawn();
        drag_state.rectangle.entity = None;
    }
}

#[allow(clippy::too_many_arguments)]
fn render_no_selected_point_drag_fill(
    commands: &mut Commands,
    no_selected_point_drag_rect: DragRect,
    drag_selection_config: &DragSelectionRectangleConfig,
    no_selected_point_drag_state: &mut NoSelectedPointDragState,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    no_selected_point_drag_query: &mut Query<&mut Transform, With<NoSelectedPointDragRectangle>>,
) {
    if let Some(entity) = no_selected_point_drag_state.entity {
        // Update existing no selected point drag rectangle
        if let Ok(mut transform) = no_selected_point_drag_query.get_mut(entity) {
            let center = Vec2::new(
                no_selected_point_drag_rect.origin.x + no_selected_point_drag_rect.width / 2.0,
                no_selected_point_drag_rect.origin.y + no_selected_point_drag_rect.height / 2.0,
            );
            transform.translation = center.extend(0.0);
            transform.scale = Vec2::new(
                no_selected_point_drag_rect.width,
                no_selected_point_drag_rect.height,
            )
            .extend(1.0);
        }
    } else {
        // Create new no selected point drag rectangle
        let background_color = drag_selection_config
            .drag_color
            .with_a(drag_selection_config.background_alpha);
        let center = Vec2::new(
            no_selected_point_drag_rect.origin.x + no_selected_point_drag_rect.width / 2.0,
            no_selected_point_drag_rect.origin.y + no_selected_point_drag_rect.height / 2.0,
        );
        let size = Vec2::new(
            no_selected_point_drag_rect.width,
            no_selected_point_drag_rect.height,
        );

        let entity = render_simple_rectangle(
            commands,
            meshes,
            materials,
            center,
            size,
            background_color,
            0.0,
        );

        // Add the component marker
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
    // Calculate no selected point drag rectangle corners from rect
    let min_x = no_selected_point_drag_rect.origin.x;
    let max_x = no_selected_point_drag_rect.origin.x + no_selected_point_drag_rect.width;
    let min_y = no_selected_point_drag_rect.origin.y;
    let max_y = no_selected_point_drag_rect.origin.y + no_selected_point_drag_rect.height;

    let top_left = Vec2::new(min_x, max_y);
    let top_right = Vec2::new(max_x, max_y);
    let bottom_right = Vec2::new(max_x, min_y);
    let bottom_left = Vec2::new(min_x, min_y);

    // Animation parameters for no selected point drag rectangle border
    let pattern_length = DASH_LENGTH + GAP_LENGTH;
    let time_offset = (time.elapsed_seconds() * ANIMATION_SPEED) % pattern_length;
    let selection_color = drag_selection_config
        .drag_color
        .with_a(drag_selection_config.wireframe_alpha);

    // Function to draw dashed line between two points
    let draw_dashed_line = |gizmos: &mut Gizmos, start: Vec2, end: Vec2, offset: f32| {
        let direction = end - start;
        let distance = direction.length();

        if distance > 0.0 {
            let normalized_direction = direction / distance;
            let mut current_distance = -offset;

            while current_distance < distance {
                let dash_start = current_distance.max(0.0);
                let dash_end = (current_distance + DASH_LENGTH).min(distance);

                if dash_start < dash_end {
                    let start_pos = start + normalized_direction * dash_start;
                    let end_pos = start + normalized_direction * dash_end;
                    gizmos.line_2d(start_pos, end_pos, selection_color);
                }

                current_distance += pattern_length;
            }
        }
    };

    // Draw animated dashed no selected point drag rectangle border
    draw_dashed_line(gizmos, top_left, top_right, time_offset);
    draw_dashed_line(gizmos, top_right, bottom_right, time_offset);
    draw_dashed_line(gizmos, bottom_right, bottom_left, time_offset);
    draw_dashed_line(gizmos, bottom_left, top_left, time_offset);
}
