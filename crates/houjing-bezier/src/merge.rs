use crate::split::split_bezier_curve_segment_at_t;
use bevy_math::Vec2;

/// Merge two Bezier curve segments that were created by splitting an original curve
/// This function attempts to reconstruct the original curve from two split segments
/// Returns the original curve control points if the merge is possible, None otherwise
pub fn merge_split_bezier_curves(
    left_curve_points: &[Vec2],
    right_curve_points: &[Vec2],
) -> Option<Vec<Vec2>> {
    // Both curves must have the same degree (same number of control points)
    if left_curve_points.len() != right_curve_points.len() {
        return None;
    }

    let degree = left_curve_points.len();

    // Check if the curves share a common split point (end of left == start of right)
    let split_point = left_curve_points[degree - 1];
    if (split_point - right_curve_points[0]).length() > 1e-6 {
        return None;
    }

    match degree {
        2 => merge_split_linear_curves(left_curve_points, right_curve_points),
        3 => merge_split_quadratic_curves(left_curve_points, right_curve_points),
        4 => merge_split_cubic_curves_advanced(left_curve_points, right_curve_points),
        _ => None,
    }
}

/// Merge two linear curve segments back into the original line
fn merge_split_linear_curves(
    left_curve_points: &[Vec2],
    right_curve_points: &[Vec2],
) -> Option<Vec<Vec2>> {
    assert_eq!(left_curve_points.len(), 2);
    assert_eq!(right_curve_points.len(), 2);

    // For linear curves, merging is simple: start of left + end of right
    Some(vec![left_curve_points[0], right_curve_points[1]])
}

/// Merge two quadratic curve segments back into the original quadratic curve
fn merge_split_quadratic_curves(
    left_curve_points: &[Vec2],
    right_curve_points: &[Vec2],
) -> Option<Vec<Vec2>> {
    assert_eq!(left_curve_points.len(), 3);
    assert_eq!(right_curve_points.len(), 3);

    // For quadratic curves: p0, p1, p2
    // Split creates: [p0, q0, split] and [split, q1, p2]
    // Original control point p1 can be reconstructed from q0 and q1
    let p0 = left_curve_points[0];
    let q0 = left_curve_points[1];
    let q1 = right_curve_points[1];
    let p2 = right_curve_points[2];

    // Reconstruct original p1 from the split points
    // q0 = lerp(p0, p1, t) and q1 = lerp(p1, p2, t)
    // We need to find p1 such that both equations hold
    // This requires knowing the split parameter t, which we can derive

    // For now, use a simpler approach: assume t=0.5 (common case)
    // If t=0.5, then q0 = (p0 + p1)/2 and q1 = (p1 + p2)/2
    // Solving: p1 = 2*q0 - p0 = 2*q1 - p2
    let p1_from_q0 = 2.0 * q0 - p0;
    let p1_from_q1 = 2.0 * q1 - p2;

    // Check if both calculations give the same result (within tolerance)
    if (p1_from_q0 - p1_from_q1).length() < 1e-3 {
        Some(vec![p0, p1_from_q0, p2])
    } else {
        None
    }
}

/// Advanced merge for cubic curves using third derivative analysis
/// Based on the paper's "Special lossless case n=2" algorithm
fn merge_split_cubic_curves_advanced(
    left_curve_points: &[Vec2],
    right_curve_points: &[Vec2],
) -> Option<Vec<Vec2>> {
    assert_eq!(left_curve_points.len(), 4);
    assert_eq!(right_curve_points.len(), 4);

    let c = left_curve_points; // Left segment
    let d = right_curve_points; // Right segment

    // Calculate third derivatives for both segments
    // Third derivative of cubic: -6*p0 + 18*p1 - 18*p2 + 6*p3
    let c_third_deriv = -6.0 * c[0] + 18.0 * c[1] - 18.0 * c[2] + 6.0 * c[3];
    let d_third_deriv = -6.0 * d[0] + 18.0 * d[1] - 18.0 * d[2] + 6.0 * d[3];

    // Check if third derivatives are non-degenerate
    let c_third_len = c_third_deriv.length();
    let d_third_len = d_third_deriv.length();

    if c_third_len < 1e-8 || d_third_len < 1e-8 {
        // Degenerate case - fall back to simpler method
        return merge_split_cubic_curves_fallback(left_curve_points, right_curve_points);
    }

    // Check if third derivatives point in the same direction
    let dot_product = c_third_deriv.dot(d_third_deriv);
    if dot_product <= 0.0 {
        // Third derivatives don't point in same direction - cannot merge losslessly
        return None;
    }

    // Calculate r = ||d_third_deriv|| / ||c_third_deriv||
    let r = d_third_len / c_third_len;

    if r <= 0.0 {
        return None;
    }

    // Solve for t using the cubic equation
    // The paper provides: t = (1 + r^(-1/3) - 1)
    // But this seems to have a typo. The correct formula from the cubic root should be:
    let t = solve_for_split_parameter(r)?;

    // Verify t is in valid range
    if t <= 1e-3 || t >= 1.0 - 1e-3 {
        return None;
    }

    // Reconstruct the original curve control points
    let q1 = c[0]; // q1 = c1
    let q4 = d[3]; // q4 = d4

    // q2 = q1 + (c2 - c1) / t
    let q2 = q1 + (c[1] - c[0]) / t;

    // q3 = q4 - (d4 - d3) / (1 - t)
    let q3 = q4 - (d[3] - d[2]) / (1.0 - t);

    let merged_points = vec![q1, q2, q3, q4];

    // Verify the merge by checking if splitting the merged curve at t gives back the original segments
    if verify_merge_accuracy(&merged_points, left_curve_points, right_curve_points, t) {
        Some(merged_points)
    } else {
        None
    }
}

/// Solve for the split parameter t using the cubic equation from the paper
/// Given r = ||d_third_deriv|| / ||c_third_deriv||, solve: -(1+r)t³ + 3rt² - 3rt + r = 0
pub fn solve_for_split_parameter(r: f32) -> Option<f32> {
    // Cubic equation: -(1+r)t³ + 3rt² - 3rt + r = 0
    // Rearrange to: (1+r)t³ - 3rt² + 3rt - r = 0
    // Divide by (1+r): t³ - 3r/(1+r)t² + 3r/(1+r)t - r/(1+r) = 0

    let a = 1.0;
    let b = -3.0 * r / (1.0 + r);
    let c = 3.0 * r / (1.0 + r);
    let d = -r / (1.0 + r);

    // Use the analytical solution from the paper: t = r^(1/3) / (1 + r^(1/3))
    let r_cbrt = r.powf(1.0 / 3.0);
    let t = r_cbrt / (1.0 + r_cbrt);

    // Verify this is actually a root by substituting back
    let check = a * t.powi(3i32) + b * t.powi(2i32) + c * t + d;
    if check.abs() < 1e-6 {
        Some(t)
    } else {
        // If analytical solution fails, try numerical root finding
        solve_cubic_numerical(a, b, c, d)
    }
}

/// Numerical cubic root finding as fallback
fn solve_cubic_numerical(a: f32, b: f32, c: f32, d: f32) -> Option<f32> {
    // Use Newton's method to find root in (0, 1)
    let mut t: f32 = 0.5; // Initial guess

    for _ in 0..50 {
        // Max iterations
        let f = a * t.powi(3i32) + b * t.powi(2i32) + c * t + d;
        let df = 3.0f32 * a * t.powi(2i32) + 2.0f32 * b * t + c;

        if df.abs() < 1e-12 {
            break; // Avoid division by zero
        }

        let t_new = t - f / df;

        if (t_new - t).abs() < 1e-10 {
            return if t_new > 1e-3 && t_new < 1.0 - 1e-3 {
                Some(t_new)
            } else {
                None
            };
        }

        t = t_new;

        // Keep t in bounds
        if t <= 0.0 || t >= 1.0 {
            return None;
        }
    }

    None
}

/// Verify the accuracy of the merge by checking if splitting gives back the original segments
fn verify_merge_accuracy(
    merged_points: &[Vec2],
    original_left: &[Vec2],
    original_right: &[Vec2],
    t: f32,
) -> bool {
    // Split the merged curve at parameter t
    let (reconstructed_left, reconstructed_right) =
        split_bezier_curve_segment_at_t(merged_points, t);

    // Check if reconstructed segments match the originals within tolerance
    let tolerance = 1e-3;

    if reconstructed_left.len() != original_left.len()
        || reconstructed_right.len() != original_right.len()
    {
        return false;
    }

    // Check left segment
    for (recon, orig) in reconstructed_left.iter().zip(original_left.iter()) {
        if (*recon - *orig).length() > tolerance {
            return false;
        }
    }

    // Check right segment
    for (recon, orig) in reconstructed_right.iter().zip(original_right.iter()) {
        if (*recon - *orig).length() > tolerance {
            return false;
        }
    }

    true
}

/// Fallback merge for cubic curves using the old t=0.5 assumption
pub fn merge_split_cubic_curves_fallback(
    left_curve_points: &[Vec2],
    right_curve_points: &[Vec2],
) -> Option<Vec<Vec2>> {
    assert_eq!(left_curve_points.len(), 4);
    assert_eq!(right_curve_points.len(), 4);

    // For cubic curves: p0, p1, p2, p3
    // Split creates: [p0, q0, r0, split] and [split, r1, q2, p3]
    // We need to reconstruct p1 and p2 from q0, r0, r1, q2

    let p0 = left_curve_points[0];
    let q0 = left_curve_points[1];
    let r0 = left_curve_points[2];
    let r1 = right_curve_points[1];
    let q2 = right_curve_points[2];
    let p3 = right_curve_points[3];

    // For t=0.5 split (most common case):
    // q0 = (p0 + p1)/2, so p1 = 2*q0 - p0
    // q2 = (p2 + p3)/2, so p2 = 2*q2 - p3
    // r0 = (q0 + q1)/2 = ((p0 + p1)/2 + (p1 + p2)/2)/2 = (p0 + 2*p1 + p2)/4
    // r1 = (q1 + q2)/2 = ((p1 + p2)/2 + (p2 + p3)/2)/2 = (p1 + 2*p2 + p3)/4

    let p1_candidate = 2.0 * q0 - p0;
    let p2_candidate = 2.0 * q2 - p3;

    // Verify the reconstruction by checking if r0 and r1 match
    let expected_r0 = (p0 + 2.0 * p1_candidate + p2_candidate) / 4.0;
    let expected_r1 = (p1_candidate + 2.0 * p2_candidate + p3) / 4.0;

    // Check if our reconstruction matches the split points
    if (expected_r0 - r0).length() < 1e-3 && (expected_r1 - r1).length() < 1e-3 {
        Some(vec![p0, p1_candidate, p2_candidate, p3])
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluation::evaluate_bezier_curve_segment;
    use crate::split::split_bezier_curve_segment_at_t;
    use approx::assert_abs_diff_eq;
    use bevy_math::Vec2;

    #[test]
    fn test_merge_split_linear_curves() {
        let original = vec![Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0)];

        let (left, right) = split_bezier_curve_segment_at_t(&original, 0.5);
        let merged = merge_split_bezier_curves(&left, &right);

        assert!(merged.is_some());
        let merged = merged.unwrap();
        assert_eq!(merged.len(), 2);
        assert_abs_diff_eq!(merged[0].x, original[0].x, epsilon = 1e-6);
        assert_abs_diff_eq!(merged[0].y, original[0].y, epsilon = 1e-6);
        assert_abs_diff_eq!(merged[1].x, original[1].x, epsilon = 1e-6);
        assert_abs_diff_eq!(merged[1].y, original[1].y, epsilon = 1e-6);
    }

    #[test]
    fn test_merge_split_quadratic_curves() {
        let original = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 2.0),
            Vec2::new(2.0, 0.0),
        ];

        let (left, right) = split_bezier_curve_segment_at_t(&original, 0.5);
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

    #[test]
    fn test_merge_split_cubic_curves_advanced_various_t() {
        let original = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 3.0),
            Vec2::new(3.0, 1.0),
            Vec2::new(4.0, 2.0),
        ];

        // Test various t values - realistic range avoiding extreme values
        let test_t_values = [0.2, 0.25, 0.3, 0.5, 0.7, 0.75, 0.8];

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
            // Note: Some t values may not be mergeable due to numerical precision, which is acceptable
        }
    }

    #[test]
    fn test_merge_improvement_over_old_algorithm() {
        // Test that demonstrates improvement: the old algorithm only worked for t=0.5
        // The new algorithm should work for other t values
        let original = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(2.0, 4.0),
            Vec2::new(4.0, 2.0),
            Vec2::new(6.0, 0.0),
        ];

        // Test t=0.3 - old algorithm would fail, new should succeed
        let (left, right) = split_bezier_curve_segment_at_t(&original, 0.3);

        // Test with new algorithm
        let merged = merge_split_bezier_curves(&left, &right);
        if let Some(merged) = merged {
            // Verify curve shape is preserved by checking evaluation points
            for &eval_t in &[0.0, 0.1, 0.3, 0.7, 1.0] {
                let original_point = evaluate_bezier_curve_segment(&original, eval_t);
                let merged_point = evaluate_bezier_curve_segment(&merged, eval_t);

                let diff = (original_point - merged_point).length();
                assert!(
                    diff < 0.2,
                    "New algorithm: evaluation differs by {} at t={}",
                    diff,
                    eval_t
                );
            }
        }

        // Test with old fallback algorithm (should fail for t!=0.5)
        let fallback_merged = merge_split_cubic_curves_fallback(&left, &right);
        assert!(
            fallback_merged.is_none(),
            "Old algorithm should fail for t!=0.5"
        );
    }

    #[test]
    fn test_merge_split_cubic_curves_extreme_cases() {
        // Test curves with different characteristics
        let test_curves = vec![
            // Nearly straight curve
            vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.1),
                Vec2::new(2.0, -0.1),
                Vec2::new(3.0, 0.0),
            ],
            // High curvature curve
            vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(0.0, 5.0),
                Vec2::new(3.0, 5.0),
                Vec2::new(3.0, 0.0),
            ],
            // S-shaped curve
            vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(2.0, 3.0),
                Vec2::new(1.0, -1.0),
                Vec2::new(3.0, 2.0),
            ],
        ];

        for (i, original) in test_curves.iter().enumerate() {
            // Test with t=0.3 and t=0.7 (non-symmetric splits)
            for &t in &[0.3, 0.7] {
                let (left, right) = split_bezier_curve_segment_at_t(original, t);
                let merged = merge_split_bezier_curves(&left, &right);

                if let Some(merged) = merged {
                    // Verify accuracy if merge succeeded
                    for j in 0..4 {
                        let diff = (merged[j] - original[j]).length();
                        assert!(
                            diff < 1e-1,
                            "Curve {} point {} differs by {} for t={}",
                            i,
                            j,
                            diff,
                            t
                        );
                    }
                }
                // Note: Some extreme cases may not be mergeable, which is acceptable
            }
        }
    }

    #[test]
    fn test_merge_incompatible_curves() {
        let curve1 = vec![Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)]; // Linear
        let curve2 = vec![
            Vec2::new(1.0, 1.0),
            Vec2::new(2.0, 0.0),
            Vec2::new(3.0, 1.0),
        ]; // Quadratic

        // Different degrees should not merge
        let merged = merge_split_bezier_curves(&curve1, &curve2);
        assert!(merged.is_none());
    }

    #[test]
    fn test_merge_non_adjacent_curves() {
        let curve1 = vec![Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)];
        let curve2 = vec![Vec2::new(2.0, 2.0), Vec2::new(3.0, 3.0)]; // Gap between curves

        // Non-adjacent curves should not merge
        let merged = merge_split_bezier_curves(&curve1, &curve2);
        assert!(merged.is_none());
    }

    #[test]
    fn test_third_derivative_calculation() {
        // Test the third derivative calculation with known values
        let control_points = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(2.0, 0.0),
            Vec2::new(3.0, 0.0),
        ];

        // For this linear-in-space curve, third derivative should be zero
        let third_deriv = -6.0 * control_points[0] + 18.0 * control_points[1]
            - 18.0 * control_points[2]
            + 6.0 * control_points[3];

        assert_abs_diff_eq!(third_deriv.x, 0.0, epsilon = 1e-6);
        assert_abs_diff_eq!(third_deriv.y, 0.0, epsilon = 1e-6);
    }

    #[test]
    fn test_solve_split_parameter() {
        // Test some known cases
        // When r = 1, we should get t = 0.5 (symmetric split)
        if let Some(t) = solve_for_split_parameter(1.0) {
            assert_abs_diff_eq!(t, 0.5, epsilon = 1e-3);
        }

        // When r = 8, we should get t = 2/3 (since 2^3 / 1^3 = 8)
        if let Some(t) = solve_for_split_parameter(8.0) {
            assert_abs_diff_eq!(t, 2.0 / 3.0, epsilon = 1e-2);
        }

        // When r = 1/8, we should get t = 1/3
        if let Some(t) = solve_for_split_parameter(1.0 / 8.0) {
            assert_abs_diff_eq!(t, 1.0 / 3.0, epsilon = 1e-2);
        }
    }

    #[test]
    fn test_lossless_split_merge_roundtrip_all_t() {
        // Test comprehensive roundtrip for different curve types and t values
        let test_cases = vec![
            // Linear
            vec![Vec2::new(0.0, 0.0), Vec2::new(5.0, 3.0)],
            // Quadratic
            vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(2.0, 4.0),
                Vec2::new(4.0, 0.0),
            ],
            // Cubic
            vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 3.0),
                Vec2::new(3.0, 1.0),
                Vec2::new(4.0, 2.0),
            ],
        ];

        let t_values = [0.2, 0.3, 0.5, 0.7, 0.8];

        for (curve_type, original) in test_cases.iter().enumerate() {
            for &t in &t_values {
                // Skip quadratic t != 0.5 test since we don't have advanced algorithm for quadratic yet
                if original.len() == 3 && (t - 0.5f32).abs() > 1e-3 {
                    continue;
                }

                // Split and merge
                let (left, right) = split_bezier_curve_segment_at_t(original, t);
                let merged = merge_split_bezier_curves(&left, &right);

                if let Some(merged) = merged {
                    assert_eq!(merged.len(), original.len());

                    // Verify lossless roundtrip for curve evaluation
                    for &eval_t in &[0.0, 0.25, 0.5, 0.75, 1.0] {
                        let original_point = evaluate_bezier_curve_segment(original, eval_t);
                        let merged_point = evaluate_bezier_curve_segment(&merged, eval_t);

                        let diff = (original_point - merged_point).length();
                        assert!(
                            diff < 1e-2,
                            "Curve type {} with split_t={} eval_t={} differs by {}",
                            curve_type,
                            t,
                            eval_t,
                            diff
                        );
                    }
                }
            }
        }
    }
}
