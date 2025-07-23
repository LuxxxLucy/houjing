//! Compatibility layer for converting between Bevy's Vec2 and houjing-bezier's Point
//!
//! This module provides conversion functions to bridge the gap between Bevy's f32-based Vec2
//! and houjing-bezier's f64-based Point, allowing clean separation between the geometry library
//! and the game engine.

use bevy::prelude::Vec2;
use houjing_bezier::Point as HoujingBezierPoint;

/// Convert a single Vec2 to HoujingBezierPoint
#[must_use]
pub fn bevy_vec2_to_hj_bezier_point(v: Vec2) -> HoujingBezierPoint {
    HoujingBezierPoint::new(v.x as f64, v.y as f64)
}

/// Convert a single HoujingBezierPoint to Vec2
#[must_use]
pub fn hj_bezier_point_to_bevy_vec2(p: HoujingBezierPoint) -> Vec2 {
    Vec2::new(p.x as f32, p.y as f32)
}

/// Convert a slice of Vec2 to Vec of HoujingBezierPoint
#[must_use]
pub fn bevy_vec2_slice_to_hj_bezier_point_vec(slice: &[Vec2]) -> Vec<HoujingBezierPoint> {
    slice
        .iter()
        .map(|&v| bevy_vec2_to_hj_bezier_point(v))
        .collect()
}

/// Convert a Vec of HoujingBezierPoint to Vec of Vec2
#[must_use]
pub fn hj_bezier_point_vec_to_bevy_vec2_vec(points: Vec<HoujingBezierPoint>) -> Vec<Vec2> {
    points
        .into_iter()
        .map(hj_bezier_point_to_bevy_vec2)
        .collect()
}

/// Convert a Vec of Vec2 to Vec of HoujingBezierPoint
#[must_use]
pub fn bevy_vec2_vec_to_hj_bezier_point_vec(vec2s: Vec<Vec2>) -> Vec<HoujingBezierPoint> {
    vec2s
        .into_iter()
        .map(bevy_vec2_to_hj_bezier_point)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_conversions() {
        let vec2 = Vec2::new(1.5, 2.5);
        let bezier = bevy_vec2_to_hj_bezier_point(vec2);
        let back_to_vec2 = hj_bezier_point_to_bevy_vec2(bezier);

        // Check conversion maintains values (with f32 precision)
        assert_eq!(bezier.x, 1.5);
        assert_eq!(bezier.y, 2.5);
        assert_eq!(back_to_vec2, vec2);
    }

    #[test]
    fn test_vec_conversions() {
        let vec2s = vec![Vec2::new(1.0, 2.0), Vec2::new(3.0, 4.0)];
        let beziers = bevy_vec2_slice_to_hj_bezier_point_vec(&vec2s);
        let back_to_vec2s = hj_bezier_point_vec_to_bevy_vec2_vec(beziers);

        assert_eq!(back_to_vec2s, vec2s);
    }

    #[test]
    fn test_precision_handling() {
        // Test edge case with very small numbers
        let vec2 = Vec2::new(0.000001, -0.000001);
        let bezier = bevy_vec2_to_hj_bezier_point(vec2);
        let back_to_vec2 = hj_bezier_point_to_bevy_vec2(bezier);

        // Should maintain f32 precision
        assert!((back_to_vec2.x - vec2.x).abs() < f32::EPSILON);
        assert!((back_to_vec2.y - vec2.y).abs() < f32::EPSILON);
    }

    #[test]
    fn test_zero_values() {
        let zero_vec2 = Vec2::ZERO;
        let zero_bezier = bevy_vec2_to_hj_bezier_point(zero_vec2);
        let back_to_zero = hj_bezier_point_to_bevy_vec2(zero_bezier);

        assert_eq!(zero_bezier, HoujingBezierPoint::ZERO);
        assert_eq!(back_to_zero, Vec2::ZERO);
    }

    #[test]
    fn test_large_values() {
        let large_vec2 = Vec2::new(1e6, -1e6);
        let large_bezier = bevy_vec2_to_hj_bezier_point(large_vec2);
        let back_to_large = hj_bezier_point_to_bevy_vec2(large_bezier);

        // Should handle large values correctly
        assert_eq!(back_to_large, large_vec2);
    }

    #[test]
    fn test_houjing_bezier_integration() {
        // Test that our compat layer works with actual houjing-bezier functions
        use houjing_bezier::evaluate_bezier_curve_segment;

        let control_points_vec2 = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(50.0, 100.0),
            Vec2::new(100.0, 0.0),
        ];

        // Convert to bezier points
        let bezier_points = bevy_vec2_slice_to_hj_bezier_point_vec(&control_points_vec2);

        // Use houjing-bezier function
        let result_bezier = evaluate_bezier_curve_segment(&bezier_points, 0.5);

        // Convert back to Vec2
        let result_vec2 = hj_bezier_point_to_bevy_vec2(result_bezier);

        // Should be the midpoint of the quadratic curve
        assert_eq!(result_vec2, Vec2::new(50.0, 50.0));
    }
}
