use crate::component::curve::Point;
use bevy::prelude::*;

/// Component representing a selected control point on a curve
#[derive(Component, Clone, Copy)]
pub struct SelectedControlPoint {
    pub curve_entity: Entity,
    pub point_index: usize,
    pub point_entity: Entity,
}

/// Shared utility function to move selected points by an offset
/// This can be used by both drag and nudge tools
pub fn move_selected_points(
    selected_query: &Query<&SelectedControlPoint>,
    point_query: &mut Query<&mut Point>,
    offset: Vec2,
) {
    for selected_point in selected_query.iter() {
        // Move the point entity directly
        if let Ok(mut point) = point_query.get_mut(selected_point.point_entity) {
            let current_pos = point.position();
            point.set_position(current_pos + offset);
            // Point position change will be detected by Bevy's change detection system
        }
    }
}
