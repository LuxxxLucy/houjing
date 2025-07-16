use bevy_math::Vec2;

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
    use bevy_math::Vec2;

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
    fn test_tangent_calculation() {
        // Linear case - constant tangent
        let control_points = vec![Vec2::ZERO, Vec2::new(10.0, 5.0)];
        let tangent = calculate_tangent_at_t_on_bezier_curve_segment(&control_points, 0.5);
        assert_eq!(tangent, Vec2::new(10.0, 5.0));
    }
}
