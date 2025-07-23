//! houjing-bezier - A library for working with Bezier curves
//!
//! This library provides utilities for:
//! - Parsing and exporting Bezier curves from various formats (JSON, SVG paths)
//! - Geometric operations on Bezier curves (evaluation, splitting, merging)
//! - Curve fitting algorithms

pub mod constants;
pub mod data;
pub mod error;
pub mod modules;

// Re-export commonly used items
pub use data::{BezierCurve, BezierSegment, Point};
pub use modules::geometry::evaluation::*;
pub use modules::geometry::merge::*;
pub use modules::geometry::split::*;
pub use modules::geometry::utils::*;
pub use modules::parse::*;
