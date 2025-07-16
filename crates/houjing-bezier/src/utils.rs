use crate::evaluation::{
    calculate_tangent_at_t_on_bezier_curve_segment, evaluate_bezier_curve_segment,
};
use bevy_math::Vec2;

/// Find the closest point on a Bezier curve segment to a target point using binary search
/// Returns the parameter t that gives the closest point on the curve segment
pub fn find_closest_t_on_bezier_curve_segment(control_points: &[Vec2], target: Vec2) -> f32 {
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

        let p1 = evaluate_bezier_curve_segment(control_points, t1);
        let p2 = evaluate_bezier_curve_segment(control_points, t2);

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

/// Calculate perpendicular line from a point to the Bezier curve segment at the closest position
/// Returns (line_start, line_end) for visualization
pub fn get_perpendicular_line_to_bezier_curve_segment(
    control_points: &[Vec2],
    target: Vec2,
    line_length: f32,
) -> (Vec2, Vec2) {
    let t = find_closest_t_on_bezier_curve_segment(control_points, target);
    let closest_point = evaluate_bezier_curve_segment(control_points, t);

    // Calculate tangent at t (derivative)
    let tangent = calculate_tangent_at_t_on_bezier_curve_segment(control_points, t);

    // Perpendicular is 90 degrees rotated tangent
    let perpendicular = Vec2::new(-tangent.y, tangent.x).normalize();

    let half_length = line_length * 0.5;
    let line_start = closest_point - perpendicular * half_length;
    let line_end = closest_point + perpendicular * half_length;

    (line_start, line_end)
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;
    use bevy_math::Vec2;

    #[test]
    fn test_find_closest_t_on_bezier_curve_segment() {
        // Simple linear case
        let control_points = vec![Vec2::ZERO, Vec2::new(10.0, 0.0)];
        let target = Vec2::new(5.0, 0.0); // Should be at t=0.5

        let t = find_closest_t_on_bezier_curve_segment(&control_points, target);
        assert_abs_diff_eq!(t, 0.5, epsilon = 1e-3);
    }
}
