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

/// Evaluate a Bezier curve segment at parameter t
pub fn evaluate_bezier_curve_segment(control_points: &[Vec2], t: f32) -> Vec2 {
    match control_points.len() {
        2 => {
            // Linear interpolation
            control_points[0].lerp(control_points[1], t)
        }
        3 => evaluate_quadratic_bezier_curve_segment(control_points, t),
        4 => evaluate_cubic_bezier_curve_segment(control_points, t),
        _ => panic!(
            "Unsupported number of control points: {}",
            control_points.len()
        ),
    }
}

/// Evaluate a quadratic Bezier curve segment at parameter t
pub fn evaluate_quadratic_bezier_curve_segment(control_points: &[Vec2], t: f32) -> Vec2 {
    assert_eq!(
        control_points.len(),
        3,
        "Quadratic Bezier requires exactly 3 control points"
    );

    let p0 = control_points[0];
    let p1 = control_points[1];
    let p2 = control_points[2];

    let one_minus_t = 1.0 - t;
    let one_minus_t_sq = one_minus_t * one_minus_t;
    let t_sq = t * t;

    one_minus_t_sq * p0 + 2.0 * one_minus_t * t * p1 + t_sq * p2
}

/// Evaluate a cubic Bezier curve segment at parameter t
pub fn evaluate_cubic_bezier_curve_segment(control_points: &[Vec2], t: f32) -> Vec2 {
    assert_eq!(
        control_points.len(),
        4,
        "Cubic Bezier requires exactly 4 control points"
    );

    let p0 = control_points[0];
    let p1 = control_points[1];
    let p2 = control_points[2];
    let p3 = control_points[3];

    let one_minus_t = 1.0 - t;
    let one_minus_t_sq = one_minus_t * one_minus_t;
    let one_minus_t_cu = one_minus_t_sq * one_minus_t;
    let t_sq = t * t;
    let t_cu = t_sq * t;

    one_minus_t_cu * p0 + 3.0 * one_minus_t_sq * t * p1 + 3.0 * one_minus_t * t_sq * p2 + t_cu * p3
}

/// Split a Bezier curve segment at parameter t using De Casteljau's algorithm
/// Returns (left_curve_segment_points, right_curve_segment_points)
pub fn split_bezier_curve_segment_at_t(control_points: &[Vec2], t: f32) -> (Vec<Vec2>, Vec<Vec2>) {
    match control_points.len() {
        2 => split_linear_bezier_curve_segment(control_points, t),
        3 => split_quadratic_bezier_curve_segment(control_points, t),
        4 => split_cubic_bezier_curve_segment(control_points, t),
        _ => panic!(
            "Unsupported number of control points: {}",
            control_points.len()
        ),
    }
}

/// Split a linear Bezier curve segment (line) at parameter t
pub fn split_linear_bezier_curve_segment(
    control_points: &[Vec2],
    t: f32,
) -> (Vec<Vec2>, Vec<Vec2>) {
    assert_eq!(
        control_points.len(),
        2,
        "Linear Bezier requires exactly 2 control points"
    );

    let p0 = control_points[0];
    let p1 = control_points[1];

    let split_point = p0.lerp(p1, t);

    let left = vec![p0, split_point];
    let right = vec![split_point, p1];

    (left, right)
}

/// Split a quadratic Bezier curve segment at parameter t using De Casteljau's algorithm
pub fn split_quadratic_bezier_curve_segment(
    control_points: &[Vec2],
    t: f32,
) -> (Vec<Vec2>, Vec<Vec2>) {
    assert_eq!(
        control_points.len(),
        3,
        "Quadratic Bezier requires exactly 3 control points"
    );

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

/// Split a cubic Bezier curve segment at parameter t using De Casteljau's algorithm
pub fn split_cubic_bezier_curve_segment(control_points: &[Vec2], t: f32) -> (Vec<Vec2>, Vec<Vec2>) {
    assert_eq!(
        control_points.len(),
        4,
        "Cubic Bezier requires exactly 4 control points"
    );

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

/// Calculate the tangent vector at parameter t on a Bezier curve segment
pub fn calculate_tangent_at_t_on_bezier_curve_segment(control_points: &[Vec2], t: f32) -> Vec2 {
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

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_evaluate_linear_bezier_curve_segment() {
        let control_points = vec![Vec2::ZERO, Vec2::new(10.0, 0.0)];

        let start = evaluate_bezier_curve_segment(&control_points, 0.0);
        let end = evaluate_bezier_curve_segment(&control_points, 1.0);
        let mid = evaluate_bezier_curve_segment(&control_points, 0.5);

        assert_eq!(start, Vec2::ZERO);
        assert_eq!(end, Vec2::new(10.0, 0.0));
        assert_eq!(mid, Vec2::new(5.0, 0.0));
    }

    #[test]
    fn test_evaluate_quadratic_bezier_curve_segment() {
        let control_points = vec![Vec2::ZERO, Vec2::new(50.0, 100.0), Vec2::new(100.0, 0.0)];

        let start = evaluate_bezier_curve_segment(&control_points, 0.0);
        let end = evaluate_bezier_curve_segment(&control_points, 1.0);
        let mid = evaluate_bezier_curve_segment(&control_points, 0.5);

        assert_eq!(start, Vec2::ZERO);
        assert_eq!(end, Vec2::new(100.0, 0.0));
        assert_eq!(mid, Vec2::new(50.0, 50.0));
    }

    #[test]
    fn test_split_cubic_bezier_curve_segment() {
        let control_points = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 2.0),
            Vec2::new(2.0, 2.0),
            Vec2::new(3.0, 0.0),
        ];

        let (left, right) = split_bezier_curve_segment_at_t(&control_points, 0.5);

        // Both should have 4 points (cubic)
        assert_eq!(left.len(), 4);
        assert_eq!(right.len(), 4);

        // Split point should be the same
        assert_eq!(left[3], right[0]);

        // Start and end points should be preserved
        assert_eq!(left[0], control_points[0]);
        assert_eq!(right[3], control_points[3]);
    }

    #[test]
    fn test_find_closest_t_on_bezier_curve_segment() {
        // Simple linear case
        let control_points = vec![Vec2::ZERO, Vec2::new(10.0, 0.0)];
        let target = Vec2::new(5.0, 0.0); // Should be at t=0.5

        let t = find_closest_t_on_bezier_curve_segment(&control_points, target);
        assert_abs_diff_eq!(t, 0.5, epsilon = 1e-3);
    }

    #[test]
    fn test_tangent_calculation() {
        // Linear case - constant tangent
        let control_points = vec![Vec2::ZERO, Vec2::new(10.0, 5.0)];
        let tangent = calculate_tangent_at_t_on_bezier_curve_segment(&control_points, 0.5);
        assert_eq!(tangent, Vec2::new(10.0, 5.0));
    }
}
