//! A 2D point with x and y coordinates

use crate::constants::FLOAT_TOLERANCE;

use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, Div, Mul, Sub};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    /// Zero point (origin)
    pub const ZERO: Point = Point { x: 0.0, y: 0.0 };

    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Calculate the Euclidean distance between this point and another
    pub fn distance(&self, other: &Point) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    /// Calculate the squared distance (faster than distance when you only need to compare)
    pub fn distance_squared(&self, other: &Point) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }

    /// Calculate the length (magnitude) of this point as a vector
    pub fn length(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// Calculate the squared length (faster than length when you only need to compare)
    pub fn length_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y
    }

    /// Normalize this point as a vector (make it unit length)
    pub fn normalize(&self) -> Point {
        let len = self.length();
        if len > 0.0 {
            Point::new(self.x / len, self.y / len)
        } else {
            Point::ZERO
        }
    }

    /// Linear interpolation between two points
    pub fn lerp(&self, other: Point, t: f64) -> Point {
        Point::new(
            self.x + t * (other.x - self.x),
            self.y + t * (other.y - self.y),
        )
    }

    /// Calculate the angle of this vector in radians (from positive x-axis)
    pub fn to_angle(&self) -> f64 {
        self.y.atan2(self.x)
    }

    /// Dot product with another point (treating both as vectors)
    pub fn dot(&self, other: &Point) -> f64 {
        self.x * other.x + self.y * other.y
    }

    /// Cross product with another point (treating both as 2D vectors, returns scalar z-component)
    pub fn cross(&self, other: &Point) -> f64 {
        self.x * other.y - self.y * other.x
    }
}

// Mathematical trait implementations

impl Add for Point {
    type Output = Point;

    fn add(self, other: Point) -> Point {
        Point::new(self.x + other.x, self.y + other.y)
    }
}

impl Sub for Point {
    type Output = Point;

    fn sub(self, other: Point) -> Point {
        Point::new(self.x - other.x, self.y - other.y)
    }
}

impl Mul<f64> for Point {
    type Output = Point;

    fn mul(self, scalar: f64) -> Point {
        Point::new(self.x * scalar, self.y * scalar)
    }
}

impl Mul<Point> for f64 {
    type Output = Point;

    fn mul(self, point: Point) -> Point {
        Point::new(self * point.x, self * point.y)
    }
}

impl Div<f64> for Point {
    type Output = Point;

    fn div(self, scalar: f64) -> Point {
        Point::new(self.x / scalar, self.y / scalar)
    }
}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        (self.x - other.x).abs() < FLOAT_TOLERANCE && (self.y - other.y).abs() < FLOAT_TOLERANCE
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:.2}, {:.2})", self.x, self.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_distance() {
        let p1 = Point::ZERO;
        let p2 = Point::new(3.0, 4.0);
        assert_eq!(p1.distance(&p2), 5.0);
    }

    #[test]
    fn test_point_equality() {
        let p1 = Point::new(1.0, 2.0);
        let p2 = Point::new(1.0, 2.0);
        let p3 = Point::new(1.00000000001, 2.00000000001);
        let p4 = Point::new(1.1, 2.1);

        // Exact equality
        assert_eq!(p1, p2);

        // Very small difference (less than tolerance)
        assert_eq!(p1, p3);

        // Larger difference (greater than tolerance)
        assert_ne!(p1, p4);
    }

    #[test]
    fn test_point_arithmetic() {
        let p1 = Point::new(1.0, 2.0);
        let p2 = Point::new(3.0, 4.0);

        // Addition
        let sum = p1 + p2;
        assert_eq!(sum, Point::new(4.0, 6.0));

        // Subtraction
        let diff = p2 - p1;
        assert_eq!(diff, Point::new(2.0, 2.0));

        // Scalar multiplication
        let scaled = p1 * 2.0;
        assert_eq!(scaled, Point::new(2.0, 4.0));

        // Reverse scalar multiplication
        let scaled2 = 3.0 * p1;
        assert_eq!(scaled2, Point::new(3.0, 6.0));

        // Division
        let divided = p2 / 2.0;
        assert_eq!(divided, Point::new(1.5, 2.0));
    }

    #[test]
    fn test_point_vector_operations() {
        let p1 = Point::new(3.0, 4.0);

        // Length
        assert_eq!(p1.length(), 5.0);

        // Normalization
        let normalized = p1.normalize();
        assert!((normalized.length() - 1.0).abs() < 1e-10);

        // Lerp
        let p2 = Point::ZERO;
        let midpoint = p1.lerp(p2, 0.5);
        assert_eq!(midpoint, Point::new(1.5, 2.0));
    }
}
