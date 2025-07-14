use crate::component::curve::Point;
use bevy::prelude::*;

/// Find the closest point to a given position within a maximum distance
/// Searches both standalone points and curve control points, returning the closest one
pub fn find_closest_point(
    target_pos: Vec2,
    point_query: &Query<(Entity, &Point)>,
    max_distance: f32,
) -> Option<Entity> {
    let mut closest_point = None;
    let mut closest_distance = max_distance;

    // Search standalone points
    for (entity, point_pos) in point_query.iter() {
        let distance = target_pos.distance(point_pos.position());
        if distance < closest_distance {
            closest_distance = distance;
            closest_point = Some(entity);
        }
    }

    closest_point
}

/// Find the closest point for snapping purposes, whether it's part of a curve or standalone
/// Returns the point entity if found, otherwise creates a new point at cursor position
pub fn find_or_create_point_for_snapping(
    cursor_pos: Vec2,
    commands: &mut Commands,
    point_query: &Query<(Entity, &Point)>,
    snap_threshold: f32,
) -> Entity {
    // First try to find existing point entity
    if let Some(point_entity) = find_closest_point(cursor_pos, point_query, snap_threshold) {
        return point_entity;
    }

    // No existing point found, create new one
    commands.spawn(Point::new(cursor_pos)).id()
}
