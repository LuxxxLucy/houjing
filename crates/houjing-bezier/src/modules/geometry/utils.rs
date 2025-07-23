use crate::data::Point;
use crate::modules::geometry::evaluation::{
    calculate_tangent_at_t_on_bezier_curve_segment, evaluate_bezier_curve_segment,
};
use crate::BezierSegment;

/// Find the nearest point on the curve to a given point using a two-step approach:
///     1. Linear sampling to get a good initial guess
///     2. Binary search refinement around the initial guess.
/// this is probably not the best way to do this.
fn find_nearest_point_on_bezier_curve_segment(
    control_points: &[Point],
    target: &Point,
) -> (Point, f64) {
    // Step 1: Linear sampling to get initial guess
    const LUT_SIZE: usize = 100;
    let mut best_t = 0.0;
    let mut best_point = evaluate_bezier_curve_segment(control_points, best_t);
    let mut best_distance = target.distance(&best_point);

    // Create a lookup table of points on the curve
    for i in 0..=LUT_SIZE {
        let t = i as f64 / LUT_SIZE as f64;
        let curve_point = evaluate_bezier_curve_segment(control_points, t);
        let distance = target.distance(&curve_point);

        if distance < best_distance {
            best_distance = distance;
            best_t = t;
            best_point = curve_point;
        }
    }

    // Step 2: Binary search refinement around the initial guess
    let mut left = (best_t - 1.0 / LUT_SIZE as f64).max(0.0);
    let mut right = (best_t + 1.0 / LUT_SIZE as f64).min(1.0);
    let tolerance = 0.001;

    while right - left > tolerance {
        let mid1 = left + (right - left) / 3.0;
        let mid2 = right - (right - left) / 3.0;

        let point1 = evaluate_bezier_curve_segment(control_points, mid1);
        let point2 = evaluate_bezier_curve_segment(control_points, mid2);

        let dist1 = target.distance(&point1);
        let dist2 = target.distance(&point2);

        if dist1 < best_distance {
            best_distance = dist1;
            best_t = mid1;
            best_point = point1;
            right = mid2;
        } else if dist2 < best_distance {
            best_distance = dist2;
            best_t = mid2;
            best_point = point2;
            left = mid1;
        } else {
            left = mid1;
            right = mid2;
        }
    }

    (best_point, best_t)
}

/// Find the closest point on a Bezier curve segment to a target point using binary search
/// Returns the parameter t that gives the closest point on the curve segment
pub fn find_closest_t_on_bezier_curve_segment(control_points: &[Point], target: &Point) -> f64 {
    find_nearest_point_on_bezier_curve_segment(control_points, target).1
}

impl BezierSegment {
    pub fn nearest_point(&self, point: &Point) -> (Point, f64) {
        find_nearest_point_on_bezier_curve_segment(&self.points(), point)
    }
}

/// Calculate perpendicular line from a point to this Bezier curve segment at the closest position
/// Returns (line_start, line_end) for visualization
pub fn get_perpendicular_line_to_bezier_curve_segment(
    control_points: &[Point],
    target: &Point,
    line_length: f64,
) -> (Point, Point) {
    let t = find_closest_t_on_bezier_curve_segment(control_points, target);
    let closest_point = evaluate_bezier_curve_segment(control_points, t);

    // Calculate tangent at t (derivative)
    let tangent = calculate_tangent_at_t_on_bezier_curve_segment(control_points, t);

    // Perpendicular is 90 degrees rotated tangent (-y, x)
    let perpendicular = Point::new(-tangent.y, tangent.x).normalize();

    let half_length = line_length * 0.5;
    let line_start = closest_point - perpendicular * half_length;
    let line_end = closest_point + perpendicular * half_length;

    (line_start, line_end)
}

impl BezierSegment {
    pub fn get_perpendicular_line(&self, point: &Point, line_length: f64) -> (Point, Point) {
        get_perpendicular_line_to_bezier_curve_segment(&self.points(), point, line_length)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::Point;
    use crate::{cubic, pt};

    #[test]
    fn test_find_closest_t_on_bezier_curve_segment() {
        // Simple linear case
        let control_points = vec![Point::ZERO, Point::new(10.0, 0.0)];
        let target = Point::new(5.0, 0.0); // Should be at t=0.5

        let t = find_closest_t_on_bezier_curve_segment(&control_points, &target);
        assert!((t - 0.5).abs() < 1e-3);
    }

    #[test]
    fn test_nearest_point_boundary_cases() {
        // Create a simple cubic bezier segment
        let segment = cubic!(Point::ZERO, pt!(1.0, 1.0), pt!(2.0, 1.0), pt!(3.0, 0.0));

        // Test 1: Point exactly at start (t=0)
        let test_point = Point::ZERO;
        let expected_point = Point::ZERO;
        let (point, t) = segment.nearest_point(&test_point);
        assert_eq!(point, expected_point);
        assert!(t.abs() < 1e-6);

        // Test 2: Point very close to start
        let test_point = pt!(0.01, 0.01);
        let expected_point = Point::ZERO;
        let (point, t) = segment.nearest_point(&test_point);
        assert!(t < 0.1, "t value was {}", t);
        assert!(point.distance(&expected_point) < 0.1);

        // Test 3: Point exactly at end (t=1)
        let test_point = pt!(3.0, 0.0);
        let expected_point = pt!(3.0, 0.0);
        let (point, t) = segment.nearest_point(&test_point);
        assert_eq!(point, expected_point);
        assert!((t - 1.0).abs() < 1e-6);

        // Test 4: Point very close to end
        let test_point = pt!(2.99, 0.01);
        let expected_point = pt!(3.0, 0.0);
        let (point, t) = segment.nearest_point(&test_point);
        assert!(t > 0.9, "t value was {}", t);
        assert!(point.distance(&expected_point) < 0.1);
    }

    #[test]
    fn test_nearest_point_middle_cases() {
        // Create a simple cubic bezier segment
        let segment = cubic!(Point::ZERO, pt!(1.0, 1.0), pt!(2.0, 1.0), pt!(3.0, 0.0));

        // Test 1: Point at middle of curve (t=0.5)
        let middle_point = segment.point_at(0.5);
        let test_point = middle_point;
        let expected_point = middle_point;
        let (point, t) = segment.nearest_point(&test_point);
        assert!((t - 0.5).abs() < 0.1, "t value was {}", t);
        assert!(point.distance(&expected_point) < 1e-6);

        // Test 2: Point above middle of curve
        let test_point = pt!(1.5, 2.0);
        let (_point, t) = segment.nearest_point(&test_point);
        assert!(t > 0.4 && t < 0.6, "t value was {}", t);

        // Test 3: Point below middle of curve
        let test_point = pt!(1.5, 0.5);
        let (_point, t) = segment.nearest_point(&test_point);
        assert!(t > 0.4 && t < 0.6, "t value was {}", t);
    }

    #[test]
    fn test_nearest_point_complex_curve() {
        // Create a more complex cubic bezier segment
        let segment = cubic!(Point::ZERO, pt!(0.5, 1.0), pt!(1.5, -1.0), pt!(2.0, 0.0));

        // Test 1: Point near a local minimum
        let test_point = pt!(1.0, 0.2);
        let (_point, t) = segment.nearest_point(&test_point);
        assert!(t > 0.45 && t < 0.55, "t value was {}", t);

        // Test 2: Point near a local maximum
        let test_point = pt!(0.5, 0.5);
        let (_point, t) = segment.nearest_point(&test_point);
        assert!(t > 0.24 && t < 0.26, "t value was {}", t);

        // Test 3: Point outside the curve's bounding box
        let test_point = pt!(3.0, 2.0);
        let (_point, t) = segment.nearest_point(&test_point);
        assert!(t > 0.9, "t value was {}", t);
    }
}
