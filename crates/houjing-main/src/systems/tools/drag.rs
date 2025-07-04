use super::cursor::{CursorState, CursorWorldPos};
use super::select::SelectedControlPoint;
use crate::EditSet;
use crate::component::curve::{BezierCurve, CurveNeedsUpdate};
use bevy::prelude::*;

pub struct DragPlugin;

impl Plugin for DragPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (handle_selected_point_dragging,).in_set(EditSet));
    }
}

fn handle_selected_point_dragging(
    input_state: Res<CursorState>,
    cursor_pos: Res<CursorWorldPos>,
    mut commands: Commands,
    selected_query: Query<&SelectedControlPoint>,
    mut curve_query: Query<&mut BezierCurve>,
) {
    if !input_state.dragging {
        return;
    }

    for selected_point in selected_query.iter() {
        if let Ok(mut curve) = curve_query.get_mut(selected_point.curve_entity) {
            if let Some(point) = curve.control_points.get_mut(selected_point.point_index) {
                *point = cursor_pos.0;

                // Mark curve for mesh update
                commands
                    .entity(selected_point.curve_entity)
                    .insert(CurveNeedsUpdate);
            }
        }
    }
}
