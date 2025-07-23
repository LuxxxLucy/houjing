//! Bezier segment: quadratic or cubic bezier curve segment

use crate::data::point::Point;
use std::fmt;

/// A bezier segment, either cubic or quadratic
#[derive(Clone, PartialEq)] // we deliberately don't derive Debug
pub enum BezierSegment {
    /// Line segment with 2 points
    Line {
        /// Points: start point, end point
        points: [Point; 2],
    },
    /// Cubic bezier with 4 control points
    Cubic {
        /// Control points: start point, control1, control2, end point
        points: [Point; 4],
    },
    /// Quadratic bezier with 3 control points
    Quadratic {
        /// Control points: start point, control point, end point
        points: [Point; 3],
    },
    /// Elliptical arc segment
    Arc {
        /// Start point
        start: Point,
        /// End point
        end: Point,
        /// Radii of the ellipse
        rx: f64,
        ry: f64,
        /// Rotation angle in degrees
        angle: f64,
        /// Whether to use the large arc
        large_arc: bool,
        /// Whether to sweep clockwise
        sweep: bool,
    },
}

impl BezierSegment {
    pub fn new(points: &[Point]) -> Self {
        match points.len() {
            2 => Self::line(points[0], points[1]),
            3 => Self::quadratic(points[0], points[1], points[2]),
            4 => Self::cubic(points[0], points[1], points[2], points[3]),
            _ => panic!("Invalid number of points for bezier segment"),
        }
    }

    /// Create a line segment with 2 points
    pub fn line(p1: Point, p2: Point) -> Self {
        Self::Line { points: [p1, p2] }
    }

    /// Create a cubic segment with 4 control points
    pub fn cubic(p1: Point, p2: Point, p3: Point, p4: Point) -> Self {
        Self::Cubic {
            points: [p1, p2, p3, p4],
        }
    }

    /// Create a quadratic segment with 3 control points
    pub fn quadratic(p1: Point, p2: Point, p3: Point) -> Self {
        Self::Quadratic {
            points: [p1, p2, p3],
        }
    }

    /// Create an elliptical arc segment
    pub fn arc(
        start: Point,
        end: Point,
        rx: f64,
        ry: f64,
        angle: f64,
        large_arc: bool,
        sweep: bool,
    ) -> Self {
        Self::Arc {
            start,
            end,
            rx,
            ry,
            angle,
            large_arc,
            sweep,
        }
    }

    /// Get all control points for this segment
    pub fn points(&self) -> Vec<Point> {
        match self {
            Self::Line { points } => points.to_vec(),
            Self::Cubic { points } => points.to_vec(),
            Self::Quadratic { points } => points.to_vec(),
            Self::Arc { start, end, .. } => vec![*start, *end],
        }
    }
}

impl fmt::Display for BezierSegment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BezierSegment::Line { points } => {
                write!(f, "Line[{0} -> {1}]", points[0], points[1])
            }
            BezierSegment::Cubic { points } => {
                write!(
                    f,
                    "Cubic[{0} -> {1} -> {2} -> {3}]",
                    points[0], points[1], points[2], points[3]
                )
            }
            BezierSegment::Quadratic { points } => {
                write!(
                    f,
                    "Quadratic[{0} -> {1} -> {2}]",
                    points[0], points[1], points[2]
                )
            }
            BezierSegment::Arc {
                start,
                end,
                rx,
                ry,
                angle,
                large_arc,
                sweep,
            } => {
                write!(
                    f,
                    "Arc[{start} -> {end}, rx: {rx:.2}, ry: {ry:.2}, angle: {angle:.2}, large_arc: {large_arc}, sweep: {sweep}]"
                )
            }
        }
    }
}
