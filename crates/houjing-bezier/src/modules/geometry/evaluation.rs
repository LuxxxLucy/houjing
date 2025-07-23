use crate::data::BezierSegment;
use crate::data::Point;

/// Evaluate a Bezier curve segment at parameter t
pub fn evaluate_bezier_curve_segment(control_points: &[Point], t: f64) -> Point {
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
pub fn evaluate_quadratic_bezier_curve_segment(control_points: &[Point], t: f64) -> Point {
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
pub fn evaluate_cubic_bezier_curve_segment(control_points: &[Point], t: f64) -> Point {
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
pub fn calculate_tangent_at_t_on_bezier_curve_segment(control_points: &[Point], t: f64) -> Point {
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

impl BezierSegment {
    /// Get a point on the bezier curve at parameter t (0 <= t <= 1)
    pub fn point_at(&self, t: f64) -> Point {
        match self {
            Self::Line { points } => points[0].lerp(points[1], t),
            Self::Cubic { points } => evaluate_cubic_bezier_curve_segment(points, t),
            Self::Quadratic { points } => evaluate_quadratic_bezier_curve_segment(points, t),
            Self::Arc {
                start: _,
                end: _,
                rx: _,
                ry: _,
                angle: _,
                large_arc: _,
                sweep: _,
            } => {
                panic!("Arc point_at not implemented yet - needs proper elliptical arc parameterization")
            }
        }
    }

    /// Sample points at specific t values
    pub fn point_at_vec(&self, t_values: &[f64]) -> Vec<Point> {
        t_values.iter().map(|&t| self.point_at(t)).collect()
    }

    /// Generate a series of points along the bezier curve
    pub fn sample_n_uniform_points(&self, num_points: usize) -> Vec<Point> {
        let ts: Vec<f64> = (0..num_points)
            .map(|i| i as f64 / (num_points - 1) as f64)
            .collect();
        self.point_at_vec(&ts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::Point;

    #[test]
    fn test_evaluate_linear_bezier_curve_segment() {
        let control_points = vec![Point::ZERO, Point::new(10.0, 0.0)];

        let start = evaluate_bezier_curve_segment(&control_points, 0.0);
        let end = evaluate_bezier_curve_segment(&control_points, 1.0);
        let mid = evaluate_bezier_curve_segment(&control_points, 0.5);

        assert_eq!(start, Point::ZERO);
        assert_eq!(end, Point::new(10.0, 0.0));
        assert_eq!(mid, Point::new(5.0, 0.0));
    }

    #[test]
    fn test_evaluate_quadratic_bezier_curve_segment() {
        let control_points = vec![Point::ZERO, Point::new(50.0, 100.0), Point::new(100.0, 0.0)];

        let start = evaluate_bezier_curve_segment(&control_points, 0.0);
        let end = evaluate_bezier_curve_segment(&control_points, 1.0);
        let mid = evaluate_bezier_curve_segment(&control_points, 0.5);

        assert_eq!(start, Point::ZERO);
        assert_eq!(end, Point::new(100.0, 0.0));
        assert_eq!(mid, Point::new(50.0, 50.0));
    }

    #[test]
    fn test_tangent_calculation() {
        // Linear case - constant tangent
        let control_points = vec![Point::ZERO, Point::new(10.0, 5.0)];
        let tangent = calculate_tangent_at_t_on_bezier_curve_segment(&control_points, 0.5);
        assert_eq!(tangent, Point::new(10.0, 5.0));
    }
}
