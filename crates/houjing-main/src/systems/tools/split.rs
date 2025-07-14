use bevy::prelude::*;

/// Clean mathematical functions for curve splitting
/// These functions work with Vec2 control points and don't depend on Bevy
pub mod curve_math {
    use bevy::prelude::Vec2;

    /// Find the closest point on a Bezier curve to a target point using binary search
    /// Returns the parameter t that gives the closest point on the curve
    pub fn find_closest_t_on_curve(control_points: &[Vec2], target: Vec2) -> f32 {
        const MAX_ITERATIONS: usize = 50;
        const TOLERANCE: f32 = 1e-6;

        let mut t_min = 0.0;
        let mut t_max = 1.0;

        for _ in 0..MAX_ITERATIONS {
            let t_mid = (t_min + t_max) * 0.5;

            if t_max - t_min < TOLERANCE {
                return t_mid;
            }

            // Sample three points to determine search direction
            let t1 = t_min + (t_max - t_min) * 0.333;
            let t2 = t_min + (t_max - t_min) * 0.667;

            let p1 = evaluate_bezier(control_points, t1);
            let p2 = evaluate_bezier(control_points, t2);

            let dist1 = target.distance_squared(p1);
            let dist2 = target.distance_squared(p2);

            if dist1 < dist2 {
                t_max = t_mid;
            } else {
                t_min = t_mid;
            }
        }

        (t_min + t_max) * 0.5
    }

    /// Evaluate a Bezier curve at parameter t
    pub fn evaluate_bezier(control_points: &[Vec2], t: f32) -> Vec2 {
        match control_points.len() {
            2 => {
                // Linear interpolation
                control_points[0].lerp(control_points[1], t)
            }
            3 => evaluate_quadratic_bezier(control_points, t),
            4 => evaluate_cubic_bezier(control_points, t),
            _ => panic!(
                "Unsupported number of control points: {}",
                control_points.len()
            ),
        }
    }

    fn evaluate_quadratic_bezier(control_points: &[Vec2], t: f32) -> Vec2 {
        let p0 = control_points[0];
        let p1 = control_points[1];
        let p2 = control_points[2];

        let one_minus_t = 1.0 - t;
        let one_minus_t_sq = one_minus_t * one_minus_t;
        let t_sq = t * t;

        one_minus_t_sq * p0 + 2.0 * one_minus_t * t * p1 + t_sq * p2
    }

    fn evaluate_cubic_bezier(control_points: &[Vec2], t: f32) -> Vec2 {
        let p0 = control_points[0];
        let p1 = control_points[1];
        let p2 = control_points[2];
        let p3 = control_points[3];

        let one_minus_t = 1.0 - t;
        let one_minus_t_sq = one_minus_t * one_minus_t;
        let one_minus_t_cu = one_minus_t_sq * one_minus_t;
        let t_sq = t * t;
        let t_cu = t_sq * t;

        one_minus_t_cu * p0
            + 3.0 * one_minus_t_sq * t * p1
            + 3.0 * one_minus_t * t_sq * p2
            + t_cu * p3
    }

    /// Split a curve at parameter t using De Casteljau's algorithm
    /// Returns (left_curve_points, right_curve_points)
    pub fn split_curve_at_t(control_points: &[Vec2], t: f32) -> (Vec<Vec2>, Vec<Vec2>) {
        match control_points.len() {
            2 => split_linear_curve(control_points, t),
            3 => split_quadratic_curve(control_points, t),
            4 => split_cubic_curve(control_points, t),
            _ => panic!(
                "Unsupported number of control points: {}",
                control_points.len()
            ),
        }
    }

    fn split_linear_curve(control_points: &[Vec2], t: f32) -> (Vec<Vec2>, Vec<Vec2>) {
        let p0 = control_points[0];
        let p1 = control_points[1];

        let split_point = p0.lerp(p1, t);

        let left = vec![p0, split_point];
        let right = vec![split_point, p1];

        (left, right)
    }

    fn split_quadratic_curve(control_points: &[Vec2], t: f32) -> (Vec<Vec2>, Vec<Vec2>) {
        let p0 = control_points[0];
        let p1 = control_points[1];
        let p2 = control_points[2];

        // De Casteljau's algorithm for quadratic curves
        let q0 = p0.lerp(p1, t);
        let q1 = p1.lerp(p2, t);
        let split_point = q0.lerp(q1, t);

        let left = vec![p0, q0, split_point];
        let right = vec![split_point, q1, p2];

        (left, right)
    }

    fn split_cubic_curve(control_points: &[Vec2], t: f32) -> (Vec<Vec2>, Vec<Vec2>) {
        let p0 = control_points[0];
        let p1 = control_points[1];
        let p2 = control_points[2];
        let p3 = control_points[3];

        // De Casteljau's algorithm for cubic curves
        // First level
        let q0 = p0.lerp(p1, t);
        let q1 = p1.lerp(p2, t);
        let q2 = p2.lerp(p3, t);

        // Second level
        let r0 = q0.lerp(q1, t);
        let r1 = q1.lerp(q2, t);

        // Third level (split point)
        let split_point = r0.lerp(r1, t);

        let left = vec![p0, q0, r0, split_point];
        let right = vec![split_point, r1, q2, p3];

        (left, right)
    }

    /// Calculate perpendicular line from a point to the curve at the closest position
    /// Returns (line_start, line_end) for visualization
    pub fn get_perpendicular_line_to_curve(
        control_points: &[Vec2],
        target: Vec2,
        line_length: f32,
    ) -> (Vec2, Vec2) {
        let t = find_closest_t_on_curve(control_points, target);
        let closest_point = evaluate_bezier(control_points, t);

        // Calculate tangent at t (derivative)
        let tangent = calculate_tangent_at_t(control_points, t);

        // Perpendicular is 90 degrees rotated tangent
        let perpendicular = Vec2::new(-tangent.y, tangent.x).normalize();

        let half_length = line_length * 0.5;
        let line_start = closest_point - perpendicular * half_length;
        let line_end = closest_point + perpendicular * half_length;

        (line_start, line_end)
    }

    /// Calculate the tangent vector at parameter t
    fn calculate_tangent_at_t(control_points: &[Vec2], t: f32) -> Vec2 {
        match control_points.len() {
            2 => {
                // Linear curve - constant tangent
                control_points[1] - control_points[0]
            }
            3 => {
                // Quadratic curve derivative
                let p0 = control_points[0];
                let p1 = control_points[1];
                let p2 = control_points[2];

                2.0 * ((1.0 - t) * (p1 - p0) + t * (p2 - p1))
            }
            4 => {
                // Cubic curve derivative
                let p0 = control_points[0];
                let p1 = control_points[1];
                let p2 = control_points[2];
                let p3 = control_points[3];

                let one_minus_t = 1.0 - t;
                let one_minus_t_sq = one_minus_t * one_minus_t;
                let t_sq = t * t;

                3.0 * (one_minus_t_sq * (p1 - p0)
                    + 2.0 * one_minus_t * t * (p2 - p1)
                    + t_sq * (p3 - p2))
            }
            _ => panic!(
                "Unsupported number of control points: {}",
                control_points.len()
            ),
        }
    }
}

// Bevy-specific implementation
use super::cursor::CursorState;
use super::tool::{Tool, ToolState};
use crate::component::curve::{BezierCurve, Point};
use crate::{EditSet, InputSet, ShowSet};
use curve_math::*;

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
            .add_systems(Update, (update_split_preview,).in_set(InputSet))
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

            let t = find_closest_t_on_curve(&control_points, cursor_pos);
            let closest_point = curve_math::evaluate_bezier(&control_points, t);
            let distance = cursor_pos.distance(closest_point);

            if distance < closest_distance {
                closest_distance = distance;
                let perpendicular_line = get_perpendicular_line_to_curve(
                    &control_points,
                    cursor_pos,
                    config.perpendicular_line_length,
                );

                closest_preview = Some(SplitPreviewData {
                    curve_entity,
                    closest_point,
                    perpendicular_line,
                    split_t: t,
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
                let (left_points, right_points) =
                    split_curve_at_t(&control_points, preview.split_t);

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
                commands.spawn(BezierCurve::new(left_point_entities));
                commands.spawn(BezierCurve::new(right_point_entities));

                // Delete the original curve
                commands.entity(preview.curve_entity).despawn();

                // Delete only the intermediate control points from original curve
                // Keep original start and end points as they are reused
                for (i, &point_entity) in curve.point_entities.iter().enumerate() {
                    if i > 0 && i < curve.point_entities.len() - 1 {
                        commands.entity(point_entity).despawn();
                    }
                }
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
        render_animated_dashed_line(
            &mut gizmos,
            line_start,
            line_end,
            config.split_preview_color,
            &time,
        );
    }
}

fn render_animated_dashed_line(
    gizmos: &mut Gizmos,
    start: Vec2,
    end: Vec2,
    color: Color,
    time: &Time,
) {
    let line_vec = end - start;
    let line_length = line_vec.length();
    let line_dir = line_vec.normalize();

    let elapsed = time.elapsed_seconds();
    let dash_offset = (elapsed * ANIMATION_SPEED) % (DASH_LENGTH + GAP_LENGTH);

    let mut current_pos = -dash_offset;
    while current_pos < line_length {
        let dash_start = current_pos.max(0.0);
        let dash_end = (current_pos + DASH_LENGTH).min(line_length);

        if dash_start < line_length && dash_end > 0.0 {
            let start_point = start + line_dir * dash_start;
            let end_point = start + line_dir * dash_end;
            gizmos.line_2d(start_point, end_point, color);
        }

        current_pos += DASH_LENGTH + GAP_LENGTH;
    }
}
