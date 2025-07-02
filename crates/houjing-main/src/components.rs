use bevy::prelude::*;

#[derive(Component, Debug, Clone)]
pub struct BezierCurve {
    pub control_points: Vec<Vec2>,
}

impl BezierCurve {
    pub fn new(points: Vec<Vec2>) -> Self {
        Self {
            control_points: points,
        }
    }

    pub fn evaluate(&self, t: f32) -> Vec2 {
        match self.control_points.len() {
            2 => {
                // Linear interpolation
                self.control_points[0].lerp(self.control_points[1], t)
            }
            3 => {
                // Quadratic Bézier
                let p0 = self.control_points[0];
                let p1 = self.control_points[1];
                let p2 = self.control_points[2];

                let a = p0.lerp(p1, t);
                let b = p1.lerp(p2, t);
                a.lerp(b, t)
            }
            4 => {
                // Cubic Bézier
                let p0 = self.control_points[0];
                let p1 = self.control_points[1];
                let p2 = self.control_points[2];
                let p3 = self.control_points[3];

                let a = p0.lerp(p1, t);
                let b = p1.lerp(p2, t);
                let c = p2.lerp(p3, t);

                let d = a.lerp(b, t);
                let e = b.lerp(c, t);

                d.lerp(e, t)
            }
            _ => {
                // Default to first point if invalid
                self.control_points.first().copied().unwrap_or(Vec2::ZERO)
            }
        }
    }
}

#[derive(Component)]
pub struct Selected;

#[derive(Component, Debug, Clone)]
pub struct SelectedControlPoint {
    pub curve_entity: Entity,
    pub point_index: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_curve_control_points_data() {
        let expected_points = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(50.0, 100.0),
            Vec2::new(100.0, 0.0),
        ];
        let curve = BezierCurve::new(expected_points.clone());

        assert_eq!(curve.control_points.len(), 3);
        for (i, &expected_point) in expected_points.iter().enumerate() {
            assert_eq!(curve.control_points[i], expected_point);
        }
    }

    #[test]
    fn test_curve_evaluation() {
        let curve = BezierCurve::new(vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(50.0, 100.0),
            Vec2::new(100.0, 0.0),
        ]);

        // Test evaluation at t=0 (should be first control point)
        let start = curve.evaluate(0.0);
        assert_eq!(start, Vec2::new(0.0, 0.0));

        // Test evaluation at t=1 (should be last control point)
        let end = curve.evaluate(1.0);
        assert_eq!(end, Vec2::new(100.0, 0.0));

        // Test evaluation at t=0.5 (should be somewhere in between)
        let mid = curve.evaluate(0.5);
        assert!(mid.x > 0.0 && mid.x < 100.0);
    }
}
