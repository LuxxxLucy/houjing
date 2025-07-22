//! Houjing Bezier Curve Library
//!
//! This library provides functions for working with Bezier curves including:
//! - Evaluation at parameter t
//! - Splitting curves using De Casteljau's algorithm  
//! - Merging split curves back together (lossless for arbitrary t values)
//! - Utility functions for finding closest points and perpendiculars

pub mod constants;
pub mod data;
pub mod error;
pub mod modules;

// Re-export to public API
pub use modules::geometry::{
    // evaluation
    calculate_tangent_at_t_on_bezier_curve_segment,
    evaluate_bezier_curve_segment,
    evaluate_cubic_bezier_curve_segment,
    evaluate_quadratic_bezier_curve_segment,
    // utils
    find_closest_t_on_bezier_curve_segment,
    get_perpendicular_line_to_bezier_curve_segment,
    // merge
    merge_curves_sequentially,
    merge_split_bezier_curves,
    // split
    split_bezier_curve_segment_at_t,
    split_cubic_bezier_curve_segment,
    split_linear_bezier_curve_segment,
    split_quadratic_bezier_curve_segment,
};
