use crate::component::curve::{BezierCurve, CurveNeedsUpdate};
use bevy::prelude::*;

/// Component representing a selected control point on a curve
#[derive(Component)]
pub struct SelectedControlPoint {
    pub curve_entity: Entity,
    pub point_index: usize,
}

/// Shared utility function to move selected points by an offset
/// This can be used by both drag and nudge tools
pub fn move_selected_points(
    commands: &mut Commands,
    selected_query: &Query<&SelectedControlPoint>,
    curve_query: &mut Query<&mut BezierCurve>,
    offset: Vec2,
) {
    for selected_point in selected_query.iter() {
        if let Ok(mut curve) = curve_query.get_mut(selected_point.curve_entity) {
            if let Some(point) = curve.control_points.get_mut(selected_point.point_index) {
                *point += offset;

                // Mark curve for mesh update
                commands
                    .entity(selected_point.curve_entity)
                    .insert(CurveNeedsUpdate);
            }
        }
    }
}
