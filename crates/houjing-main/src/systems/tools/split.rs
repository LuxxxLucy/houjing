use bevy::prelude::*;
use houjing_bezier::{
    evaluate_bezier_curve_segment, find_closest_t_on_bezier_curve_segment,
    get_perpendicular_line_to_bezier_curve_segment, split_bezier_curve_segment_at_t,
};

// Bevy-specific implementation
use super::cursor::CursorState;
use super::tool::{Tool, ToolState};
use crate::compat;
use crate::component::curve::{BezierCurve, Point};
use crate::rendering::{DashedLineConfig, render_animated_dashed_line};
use crate::{EditSet, InputSet, ShowSet};

// Configuration constants
const DEFAULT_PERPENDICULAR_LINE_LENGTH: f32 = 60.0;
const DEFAULT_CLOSEST_POINT_RADIUS: f32 = 8.0;
const DEFAULT_SPLIT_PREVIEW_COLOR: Color = Color::CYAN;
const DEFAULT_CLOSEST_POINT_COLOR: Color = Color::YELLOW;

// Animation constants for dashed line
const DASH_LENGTH: f32 = 8.0;
const GAP_LENGTH: f32 = 6.0;
const ANIMATION_SPEED: f32 = 50.0; // pixels per second

#[derive(Resource)]
pub struct SplitConfig {
    pub perpendicular_line_length: f32,
    pub closest_point_radius: f32,
    pub split_preview_color: Color,
    pub closest_point_color: Color,
}

impl Default for SplitConfig {
    fn default() -> Self {
        Self {
            perpendicular_line_length: DEFAULT_PERPENDICULAR_LINE_LENGTH,
            closest_point_radius: DEFAULT_CLOSEST_POINT_RADIUS,
            split_preview_color: DEFAULT_SPLIT_PREVIEW_COLOR,
            closest_point_color: DEFAULT_CLOSEST_POINT_COLOR,
        }
    }
}

#[derive(Resource, Default)]
pub struct SplitToolState {
    pub preview_data: Option<SplitPreviewData>,
}

impl SplitToolState {
    pub fn reset(&mut self, _commands: &mut Commands) {
        self.preview_data = None;
    }
}

#[derive(Debug, Clone)]
pub struct SplitPreviewData {
    pub curve_entity: Entity,
    pub closest_point: Vec2,
    pub perpendicular_line: (Vec2, Vec2),
    pub split_t: f32,
}

pub struct SplitPlugin;

impl Plugin for SplitPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SplitToolState>()
            .init_resource::<SplitConfig>()
            .add_systems(
                Update,
                (update_split_preview, update_split_cursor).in_set(InputSet),
            )
            .add_systems(Update, (handle_split_action,).in_set(EditSet))
            .add_systems(Update, (render_split_preview,).in_set(ShowSet));
    }
}

#[allow(clippy::too_many_arguments)]
fn update_split_preview(
    mut split_state: ResMut<SplitToolState>,
    tool_state: Res<ToolState>,
    cursor_state: Res<CursorState>,
    curve_query: Query<(Entity, &BezierCurve)>,
    point_query: Query<&Point>,
    config: Res<SplitConfig>,
    mut commands: Commands,
) {
    // Check if tool is active, reset state if not
    if !tool_state.is_currently_using_tool(Tool::Split) {
        split_state.reset(&mut commands);
        return;
    }

    let cursor_pos = cursor_state.cursor_position;
    let mut closest_preview: Option<SplitPreviewData> = None;
    let mut closest_distance = f32::INFINITY;

    // Find the closest curve to the cursor
    for (curve_entity, curve) in curve_query.iter() {
        if let Some(control_points) = curve.resolve_positions(&point_query) {
            // Skip curves with insufficient points
            if control_points.len() < 2 {
                continue;
            }

            let bezier_points = compat::bevy_vec2_slice_to_hj_bezier_point_vec(&control_points);
            let t = find_closest_t_on_bezier_curve_segment(
                &bezier_points,
                &compat::bevy_vec2_to_hj_bezier_point(cursor_pos),
            );
            let closest_point = compat::hj_bezier_point_to_bevy_vec2(
                evaluate_bezier_curve_segment(&bezier_points, t),
            );
            let distance = cursor_pos.distance(closest_point);

            if distance < closest_distance {
                closest_distance = distance;
                let (line_start, line_end) = get_perpendicular_line_to_bezier_curve_segment(
                    &bezier_points,
                    &compat::bevy_vec2_to_hj_bezier_point(cursor_pos),
                    config.perpendicular_line_length as f64,
                );

                closest_preview = Some(SplitPreviewData {
                    curve_entity,
                    closest_point,
                    split_t: t as f32,
                    perpendicular_line: (
                        compat::hj_bezier_point_to_bevy_vec2(line_start),
                        compat::hj_bezier_point_to_bevy_vec2(line_end),
                    ),
                });
            }
        }
    }

    split_state.preview_data = closest_preview;
}

#[allow(clippy::too_many_arguments)]
fn handle_split_action(
    mut commands: Commands,
    split_state: Res<SplitToolState>,
    tool_state: Res<ToolState>,
    cursor_state: Res<CursorState>,
    curve_query: Query<(Entity, &BezierCurve)>,
    point_query: Query<&Point>,
) {
    // Check if tool is active
    if !tool_state.is_currently_using_tool(Tool::Split) {
        return;
    }

    // Check if mouse was just clicked
    if !cursor_state.mouse_just_pressed {
        return;
    }

    // Check if we have preview data
    if let Some(preview) = &split_state.preview_data {
        // Get the curve we're splitting
        if let Ok((_, curve)) = curve_query.get(preview.curve_entity) {
            if let Some(control_points) = curve.resolve_positions(&point_query) {
                // Split the curve using the calculated t value
                let bezier_points = compat::bevy_vec2_slice_to_hj_bezier_point_vec(&control_points);
                let (left_bezier_points, right_bezier_points) =
                    split_bezier_curve_segment_at_t(&bezier_points, preview.split_t as f64);
                let left_points = compat::hj_bezier_point_vec_to_bevy_vec2_vec(left_bezier_points);
                let right_points =
                    compat::hj_bezier_point_vec_to_bevy_vec2_vec(right_bezier_points);

                // Reuse original start and end points, create new intermediate points
                let original_start = curve.point_entities[0];
                let original_end = curve.point_entities[curve.point_entities.len() - 1];

                // Create split point entity
                let split_point_entity = commands
                    .spawn(Point::new(left_points[left_points.len() - 1]))
                    .id();

                // Build left curve: [original_start, new_intermediates..., split_point]
                let mut left_point_entities = vec![original_start];
                left_point_entities.extend(create_point_entities(
                    &mut commands,
                    &left_points[1..left_points.len() - 1],
                ));
                left_point_entities.push(split_point_entity);

                // Build right curve: [split_point, new_intermediates..., original_end]
                let mut right_point_entities = vec![split_point_entity];
                right_point_entities.extend(create_point_entities(
                    &mut commands,
                    &right_points[1..right_points.len() - 1],
                ));
                right_point_entities.push(original_end);

                // Create new curve entities
                let left_curve_entity = commands
                    .spawn(BezierCurve::new(left_point_entities.clone()))
                    .id();
                let right_curve_entity = commands
                    .spawn(BezierCurve::new(right_point_entities.clone()))
                    .id();

                // Delete the original curve
                commands.entity(preview.curve_entity).despawn();

                // Delete only the intermediate control points from original curve
                // Keep original start and end points as they are reused
                for (i, &point_entity) in curve.point_entities.iter().enumerate() {
                    if i > 0 && i < curve.point_entities.len() - 1 {
                        commands.entity(point_entity).despawn();
                    }
                }

                // now debug show all the point entity id and curve entity id after the split
                println!(
                    "After split, left curve {left_curve_entity:?} points: {left_point_entities:?}, positions: {left_points:?}"
                );
                println!(
                    "After split, right curve {right_curve_entity:?} points: {right_point_entities:?}, positions: {right_points:?}"
                );
            }
        }
    }
}

fn create_point_entities(commands: &mut Commands, points: &[Vec2]) -> Vec<Entity> {
    points
        .iter()
        .map(|&pos| commands.spawn(Point::new(pos)).id())
        .collect()
}

fn render_split_preview(
    mut gizmos: Gizmos,
    split_state: Res<SplitToolState>,
    tool_state: Res<ToolState>,
    config: Res<SplitConfig>,
    time: Res<Time>,
) {
    // Check if tool is active
    if !tool_state.is_currently_using_tool(Tool::Split) {
        return;
    }

    // Render preview if available
    if let Some(preview) = &split_state.preview_data {
        // Render the closest point as a circle
        gizmos.circle_2d(
            preview.closest_point,
            config.closest_point_radius,
            config.closest_point_color,
        );

        // Render the animated dashed perpendicular line
        let (line_start, line_end) = preview.perpendicular_line;
        let dash_config = DashedLineConfig {
            dash_length: DASH_LENGTH,
            gap_length: GAP_LENGTH,
            animation_speed: ANIMATION_SPEED,
        };
        render_animated_dashed_line(
            &mut gizmos,
            line_start,
            line_end,
            config.split_preview_color,
            &dash_config,
            &time,
        );
    }
}

fn update_split_cursor(tool_state: Res<ToolState>, mut windows: Query<&mut Window>) {
    if let Ok(mut window) = windows.get_single_mut() {
        if tool_state.is_currently_using_tool(Tool::Split) {
            // Use crosshair cursor for split tool (closest to scissor precision)
            window.cursor.icon = CursorIcon::Crosshair;
        } else {
            // Reset to default cursor for other tools (if not overridden by other tools)
            if window.cursor.icon == CursorIcon::Crosshair {
                window.cursor.icon = CursorIcon::Default;
            }
        }
    }
}
