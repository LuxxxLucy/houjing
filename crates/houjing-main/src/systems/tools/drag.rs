use super::cursor::{CursorState, CursorWorldPos};
use super::select::SelectedControlPoint;
use crate::component::curve::{BezierCurve, CurveNeedsUpdate};
use crate::systems::tools::cursor::CursorVisualizationConfig;
use crate::{EditSet, ShowSet};
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Resource, Default)]
pub struct OriginalCurveStates {
    pub curves: HashMap<Entity, Vec<Vec2>>,
}

pub struct DragPlugin;

impl Plugin for DragPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OriginalCurveStates>()
            .add_systems(Update, (handle_selected_point_dragging,).in_set(EditSet))
            .add_systems(
                Update,
                (render_selected_point_drag, render_selected_rectangle_drag).in_set(ShowSet),
            );
    }
}

fn handle_selected_point_dragging(
    input_state: Res<CursorState>,
    cursor_pos: Res<CursorWorldPos>,
    mut commands: Commands,
    selected_query: Query<&SelectedControlPoint>,
    mut curve_query: Query<&mut BezierCurve>,
    mut original_states: ResMut<OriginalCurveStates>,
) {
    // Capture original curve states when dragging starts with selected points
    if input_state.dragging && original_states.curves.is_empty() && !selected_query.is_empty() {
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
                original_states
                    .curves
                    .insert(selected_point.curve_entity, curve.control_points.clone());
            }
        }
        debug!("Total curves stored: {}", original_states.curves.len());
    }

    // Clear original states when dragging ends
    if !input_state.dragging {
        original_states.curves.clear();
        return;
    }

    // Update curve points during drag
    for selected_point in selected_query.iter() {
        if let Ok(mut curve) = curve_query.get_mut(selected_point.curve_entity) {
            if let Some(point) = curve.control_points.get_mut(selected_point.point_index) {
                *point = cursor_pos.0;

                // Mark curve for mesh update
                commands
                    .entity(selected_point.curve_entity)
                    .insert(CurveNeedsUpdate);
            }
        }
    }
}

fn render_selected_point_drag(
    mut gizmos: Gizmos,
    cursor_pos: Res<CursorWorldPos>,
    input_state: Res<CursorState>,
    config: Res<CursorVisualizationConfig>,
    selected_query: Query<&SelectedControlPoint>,
    original_states: Res<OriginalCurveStates>,
) {
    // Only show when dragging and points are selected
    if !input_state.dragging || selected_query.is_empty() {
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
        config: &CursorVisualizationConfig,
    ) {
        gizmos.circle_2d(start_pos, 4.0, config.drag_color.with_a(0.5));
    }

    fn render_original_curve(
        gizmos: &mut Gizmos,
        control_points: &[Vec2],
        config: &CursorVisualizationConfig,
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
            gizmos.line_2d(p1, p2, config.drag_color.with_a(0.3));
        }
    }

    // Render original curves as low-opacity lines
    if !original_states.curves.is_empty() {
        for original_curve_points in original_states.curves.values() {
            render_original_curve(&mut gizmos, original_curve_points, &config);
        }
    }

    // Draw diamond cursor at current position
    render_diamond_cursor(&mut gizmos, cursor_pos.0, &config);

    // Draw drag start position indicator
    render_drag_start_indicator(&mut gizmos, input_state.drag_start_pos, &config);
}

fn render_selected_rectangle_drag(
    mut gizmos: Gizmos,
    cursor_pos: Res<CursorWorldPos>,
    input_state: Res<CursorState>,
    config: Res<CursorVisualizationConfig>,
    selected_query: Query<&SelectedControlPoint>,
    time: Res<Time>,
) {
    // Only show when dragging and no points are selected
    if !input_state.dragging || !selected_query.is_empty() {
        return;
    }

    fn render_animated_selection_rectangle(
        gizmos: &mut Gizmos,
        start: Vec2,
        end: Vec2,
        config: &CursorVisualizationConfig,
        time: &Time,
    ) {
        // Calculate rectangle corners
        let min_x = start.x.min(end.x);
        let max_x = start.x.max(end.x);
        let min_y = start.y.min(end.y);
        let max_y = start.y.max(end.y);

        let top_left = Vec2::new(min_x, max_y);
        let top_right = Vec2::new(max_x, max_y);
        let bottom_right = Vec2::new(max_x, min_y);
        let bottom_left = Vec2::new(min_x, min_y);

        // Animation parameters for rectangle border
        let dash_length = 6.0;
        let gap_length = 4.0;
        let pattern_length = dash_length + gap_length;
        let animation_speed = 40.0; // pixels per second
        let time_offset = (time.elapsed_seconds() * animation_speed) % pattern_length;
        let selection_color = config.drag_color.with_a(0.8);

        // Function to draw dashed line between two points
        let draw_dashed_line = |gizmos: &mut Gizmos, start: Vec2, end: Vec2, offset: f32| {
            let direction = end - start;
            let distance = direction.length();

            if distance > 0.0 {
                let normalized_direction = direction / distance;
                let mut current_distance = -offset;

                while current_distance < distance {
                    let dash_start = current_distance.max(0.0);
                    let dash_end = (current_distance + dash_length).min(distance);

                    if dash_start < dash_end {
                        let start_pos = start + normalized_direction * dash_start;
                        let end_pos = start + normalized_direction * dash_end;
                        gizmos.line_2d(start_pos, end_pos, selection_color);
                    }

                    current_distance += pattern_length;
                }
            }
        };

        // Draw animated dashed rectangle border
        draw_dashed_line(gizmos, top_left, top_right, time_offset);
        draw_dashed_line(gizmos, top_right, bottom_right, time_offset);
        draw_dashed_line(gizmos, bottom_right, bottom_left, time_offset);
        draw_dashed_line(gizmos, bottom_left, top_left, time_offset);
    }

    // Draw animated selection rectangle
    render_animated_selection_rectangle(
        &mut gizmos,
        input_state.drag_start_pos,
        cursor_pos.0,
        &config,
        &time,
    );
}
