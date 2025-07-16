use bevy_math::Vec2;

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

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_math::Vec2;

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
}
