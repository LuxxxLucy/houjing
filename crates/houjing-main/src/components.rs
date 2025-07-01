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
pub struct ControlPoint {
    pub position: Vec2,
    pub curve_entity: Entity,
    pub point_index: usize,
}

#[derive(Component)]
pub struct Selected;
