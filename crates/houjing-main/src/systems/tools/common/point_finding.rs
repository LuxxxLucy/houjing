use crate::component::curve::BezierCurve;
use bevy::prelude::*;

/// Information about a found control point
#[derive(Debug, Clone, Copy)]
pub struct FoundControlPoint {
    pub curve_entity: Entity,
    pub point_index: usize,
    pub position: Vec2,
}

/// Find the closest control point to a given position within a maximum distance
pub fn find_closest_control_point(
    target_pos: Vec2,
    curves: &Query<(Entity, &BezierCurve)>,
    max_distance: f32,
) -> Option<FoundControlPoint> {
    let mut closest_point = None;
    let mut closest_distance = max_distance;

    for (curve_entity, curve) in curves.iter() {
        for (point_index, &point_pos) in curve.control_points.iter().enumerate() {
            let distance = target_pos.distance(point_pos);
            if distance < closest_distance {
                closest_distance = distance;
                closest_point = Some(FoundControlPoint {
                    curve_entity,
                    point_index,
                    position: point_pos,
                });
            }
        }
    }

    closest_point
}

/// Find the closest control point position for snapping purposes
/// Returns the snapped position if found, otherwise returns the original position
pub fn snap_to_closest_point(
    cursor_pos: Vec2,
    curves: &Query<(Entity, &BezierCurve)>,
    snap_threshold: f32,
) -> Vec2 {
    find_closest_control_point(cursor_pos, curves, snap_threshold)
        .map(|found| found.position)
        .unwrap_or(cursor_pos)
}
