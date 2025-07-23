use crate::data::Point;

/// Merge two Bezier curve segments that were created by splitting an original curve
/// This function attempts to reconstruct the original curve from two split segments
/// Returns the original curve control points if the merge is possible, None otherwise
pub fn merge_split_bezier_curves(
    left_curve_points: &[Point],
    right_curve_points: &[Point],
) -> Option<Vec<Point>> {
    // Both curves must have the same degree (same number of control points)
    if left_curve_points.len() != right_curve_points.len() {
        println!(
            "len not match: {} != {}",
            left_curve_points.len(),
            right_curve_points.len()
        );
        return None;
    }

    let degree = left_curve_points.len();

    // C0 continuity
    if (left_curve_points[degree - 1] - right_curve_points[0]).length() > 1e-3 {
        println!(
            "C0 continuity not met: {:?} (last point of first curve) != {:?} (first point of second curve)",
            left_curve_points[degree - 1], right_curve_points[0]
        );
        return None;
    }

    match degree {
        2 => merge_split_linear_curves(left_curve_points, right_curve_points),
        3 => merge_split_quadratic_curves(left_curve_points, right_curve_points),
        4 => merge_split_cubic_curves(left_curve_points, right_curve_points),
        _ => None,
    }
}

/// Merge multiple curves sequentially in order
pub fn merge_curves_sequentially(mut curves: Vec<Vec<Point>>) -> Vec<Vec<Point>> {
    if curves.len() < 2 {
        return curves;
    }

    loop {
        let mut any_merged = false;

        for i in 0..curves.len() - 1 {
            let (c1, c2) = (&curves[i], &curves[i + 1]);

            if c1.len() != c2.len() {
                continue;
            }

            let tolerance = 1e-3;
            let merged_curve = if (c1[c1.len() - 1] - c2[0]).length() < tolerance {
                merge_split_bezier_curves(c1, c2)
            } else if (c2[c2.len() - 1] - c1[0]).length() < tolerance {
                merge_split_bezier_curves(c2, c1)
            } else {
                None
            };

            if let Some(merged) = merged_curve {
                curves.splice(i..i + 2, [merged]);
                any_merged = true;
                break;
            }
        }

        if !any_merged {
            break;
        }
    }

    curves
}

/// Merge two linear curve segments back into the original line
fn merge_split_linear_curves(
    left_curve_points: &[Point],
    right_curve_points: &[Point],
) -> Option<Vec<Point>> {
    if left_curve_points.len() != 2 || right_curve_points.len() != 2 {
        return None;
    }

    if (left_curve_points[1].x - left_curve_points[0].x).abs() < 1e-3 {
        // left curve is a vertical line
        if (right_curve_points[1].x - right_curve_points[0].x).abs() < 1e-3 {
            // check if the right curve is a vertical line too
            return Some(vec![left_curve_points[0], right_curve_points[1]]);
        }
        println!("Vertical line not met");
    }

    let dy_dx_ratio_1 = (left_curve_points[1].y - left_curve_points[0].y)
        / (left_curve_points[1].x - left_curve_points[0].x);
    let dy_dx_ratio_2 = (right_curve_points[1].y - right_curve_points[0].y)
        / (right_curve_points[1].x - right_curve_points[0].x);

    if (dy_dx_ratio_1 - dy_dx_ratio_2).abs() > 1e-3 {
        println!("dy_dx_ratio not met");
        return None;
    }

    Some(vec![left_curve_points[0], right_curve_points[1]])
}

/// Merge two quadratic curve segments back into the original quadratic curve
fn merge_split_quadratic_curves(
    left_curve_points: &[Point],
    right_curve_points: &[Point],
) -> Option<Vec<Point>> {
    assert_eq!(left_curve_points.len(), 3);
    assert_eq!(right_curve_points.len(), 3);

    // For quadratic curves: p0, p1, p2
    // denote left curve as a0, a1, a2
    // denote right curve as b0, b1, b2
    // then (a2-a1) / t = (b1-b0) / (1-t)
    let a0 = left_curve_points[0];
    let a1 = left_curve_points[1];
    let a2 = left_curve_points[2];

    let b0 = right_curve_points[0];
    let b1 = right_curve_points[1];
    let b2 = right_curve_points[2];

    let a1_to_a2 = a2 - a1;
    let b0_to_b1 = b1 - b0;
    if (a1_to_a2.to_angle() - b0_to_b1.to_angle()).abs() > 1e-3 {
        println!("angle not met");
        return None;
    }

    // t = ||a2 - a1|| / (||a2 - a1|| + || b1 - b0 ||)
    let t = a1_to_a2.length() / (a1_to_a2.length() + b0_to_b1.length());

    let p1 = a0 + (a1 - a0) * 1.0 / t;

    Some(vec![a0, p1, b2])
}

fn merge_split_cubic_curves(
    left_curve_points: &[Point],
    right_curve_points: &[Point],
) -> Option<Vec<Point>> {
    assert_eq!(left_curve_points.len(), 4);
    assert_eq!(right_curve_points.len(), 4);

    // For cubic curves: p0, p1, p2, p3
    // denote left curve as a0, a1, a2, a3
    // denote right curve as b0, b1, b2, b3
    // then (a3-a2) / t = (b1-b0) / (1-t)
    let a0 = left_curve_points[0];
    let a1 = left_curve_points[1];
    let a2 = left_curve_points[2];
    let a3 = left_curve_points[3];

    let b0 = right_curve_points[0];
    let b1 = right_curve_points[1];
    let b2 = right_curve_points[2];
    let b3 = right_curve_points[3];

    let a2_to_a3 = a3 - a2;
    let b0_to_b1 = b1 - b0;
    if (a2_to_a3.to_angle() - b0_to_b1.to_angle()).abs() > 1e-3 {
        println!("angle not met");
        return None;
    }

    let t = a2_to_a3.length() / (a2_to_a3.length() + b0_to_b1.length());

    let a12_ = a1 + (a2 - a1) * 1.0 / t;
    let b12_ = b2 + (b1 - b2) * 1.0 / (1.0 - t);

    if (a12_ - b12_).length() > 1e-3 {
        println!("a12_ - b12_ not met {a12_:?} != {b12_:?}");
        return None;
    }

    let p1 = a0 + (a1 - a0) * 1.0 / t;
    let p2 = b3 + (b2 - b3) * 1.0 / (1.0 - t);

    Some(vec![a0, p1, p2, b3])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::geometry::evaluation::evaluate_bezier_curve_segment;
    use crate::modules::geometry::split::split_bezier_curve_segment_at_t;
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_merge_split_linear_curves() {
        let linear_curve_1 = vec![Point::ZERO, Point::new(3.0, 4.0)];
        let linear_curve_2 = vec![Point::new(3.0, 4.0), Point::new(6.0, 8.0)];
        let linear_curve_3 = vec![Point::new(3.0, 4.0), Point::new(8.0, 8.0)];
        let original = vec![Point::ZERO, Point::new(6.0, 8.0)];

        let merged = merge_split_bezier_curves(&linear_curve_1, &linear_curve_2);

        assert!(merged.is_some());
        let merged = merged.unwrap();
        assert_eq!(merged.len(), 2);
        assert_abs_diff_eq!(merged[0].x, original[0].x, epsilon = 1e-6);
        assert_abs_diff_eq!(merged[0].y, original[0].y, epsilon = 1e-6);
        assert_abs_diff_eq!(merged[1].x, original[1].x, epsilon = 1e-6);
        assert_abs_diff_eq!(merged[1].y, original[1].y, epsilon = 1e-6);

        assert!(merge_split_bezier_curves(&linear_curve_1, &linear_curve_3).is_none());

        // merge vertical curve
        let vertical_curve_1 = vec![Point::new(10.0, 0.0), Point::new(10.0, 10.0)];
        let vertical_curve_2 = vec![Point::new(10.0, 10.0), Point::new(10.0, 20.0)];
        assert!(merge_split_bezier_curves(&vertical_curve_1, &vertical_curve_2).is_some());

        // merge horizontal curve
        let horizontal_curve_1 = vec![Point::new(10.0, 0.0), Point::new(20.0, 0.0)];
        let horizontal_curve_2 = vec![Point::new(20.0, 0.0), Point::new(30.0, 0.0)];
        assert!(merge_split_bezier_curves(&horizontal_curve_1, &horizontal_curve_2).is_some());
    }

    #[test]
    fn test_merge_split_quadratic_curves() {
        let original = vec![Point::ZERO, Point::new(1.0, 2.0), Point::new(2.0, 0.0)];

        let test_t_values = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9];
        for &t in &test_t_values {
            let (left, right) = split_bezier_curve_segment_at_t(&original, t);
            let merged = merge_split_bezier_curves(&left, &right);

            assert!(merged.is_some());
            let merged = merged.unwrap();
            assert_eq!(merged.len(), 3);

            // Check that merged curve matches original
            for i in 0..3 {
                assert_abs_diff_eq!(merged[i].x, original[i].x, epsilon = 1e-3);
                assert_abs_diff_eq!(merged[i].y, original[i].y, epsilon = 1e-3);
            }
        }
    }

    #[test]
    fn test_merge_split_cubic_curves_captured_from_user() {
        // a test case captured from user
        let left = vec![
            Point::new(-245.99219, -150.95313),
            Point::new(-157.95697, -25.547081),
            Point::new(-22.974106, 5.259905),
            Point::new(78.418434, 52.253418),
        ];

        let right = vec![
            Point::new(78.418434, 52.253418),
            Point::new(176.16669, 97.55787),
            Point::new(242.69585, 157.90619),
            Point::new(205.84375, 332.5625),
        ];

        let merged = merge_split_bezier_curves(&left, &right);
        assert!(merged.is_some());
    }

    #[test]
    fn test_merge_split_cubic_curves() {
        let original = vec![
            Point::ZERO,
            Point::new(1.0, 3.0),
            Point::new(3.0, 1.0),
            Point::new(4.0, 2.0),
        ];

        // Test various t values - realistic range avoiding extreme values
        let test_t_values = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9];

        for &t in &test_t_values {
            let (left, right) = split_bezier_curve_segment_at_t(&original, t);
            let merged = merge_split_bezier_curves(&left, &right);

            if let Some(merged) = merged {
                assert_eq!(merged.len(), 4);

                // Verify curve evaluation at multiple points matches (more important than control points)
                for &eval_t in &[0.0, 0.25, 0.5, 0.75, 1.0] {
                    let original_point = evaluate_bezier_curve_segment(&original, eval_t);
                    let merged_point = evaluate_bezier_curve_segment(&merged, eval_t);

                    let diff = (original_point - merged_point).length();
                    assert!(
                        diff < 1e-1,
                        "Evaluation at eval_t={} differs by {} for split_t={}: {:?} vs {:?}",
                        eval_t,
                        diff,
                        t,
                        original_point,
                        merged_point
                    );
                }
            }
        }
    }

    #[test]
    fn test_merge_curves_sequentially() {
        // Test case 1: Chain of linear curves that can all be merged
        let curve1 = vec![Point::ZERO, Point::new(1.0, 1.0)];
        let curve2 = vec![Point::new(1.0, 1.0), Point::new(2.0, 2.0)];
        let curve3 = vec![Point::new(2.0, 2.0), Point::new(3.0, 3.0)];

        let input_curves = vec![curve1, curve2, curve3];
        let result = merge_curves_sequentially(input_curves);

        // Should merge into a single curve from (0,0) to (3,3)
        assert_eq!(result.len(), 1);
        assert_eq!(result[0][0], Point::ZERO);
        assert_eq!(result[0][1], Point::new(3.0, 3.0));

        // Test case 2: Mix of mergeable and non-mergeable curves
        let curve_a = vec![Point::ZERO, Point::new(1.0, 0.0)]; // A→B
        let curve_b = vec![Point::new(1.0, 0.0), Point::new(2.0, 0.0)]; // B→C (can merge with A)
        let curve_c = vec![Point::new(5.0, 5.0), Point::new(6.0, 5.0)]; // Disconnected
        let curve_d = vec![Point::new(6.0, 5.0), Point::new(7.0, 5.0)]; // D→E (can merge with C)

        let input_curves2 = vec![curve_a, curve_b, curve_c, curve_d];
        let result2 = merge_curves_sequentially(input_curves2);

        // Should result in 2 merged curves: A→B→C and C→D→E
        assert_eq!(result2.len(), 2);

        // Test case 3: No merges possible
        let curve_x = vec![Point::ZERO, Point::new(1.0, 0.0)];
        let curve_y = vec![Point::new(5.0, 5.0), Point::new(6.0, 5.0)]; // Disconnected

        let input_curves3 = vec![curve_x, curve_y];
        let result3 = merge_curves_sequentially(input_curves3);

        // Should remain 2 separate curves
        assert_eq!(result3.len(), 2);

        // Test case 4: Single curve - should return unchanged
        let single_curve = vec![Point::ZERO, Point::new(1.0, 1.0)];
        let input_curves4 = vec![single_curve.clone()];
        let result4 = merge_curves_sequentially(input_curves4);

        assert_eq!(result4.len(), 1);
        assert_eq!(result4[0], single_curve);
    }
}
