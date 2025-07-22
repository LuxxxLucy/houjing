use super::common::selected::SelectedControlPoint;
use super::select::SelectionToolState;
use super::tool::{Tool, ToolState};
use crate::EditSet;
use crate::component::curve::BezierCurve;
use bevy::prelude::*;
use log::debug;
use std::collections::HashMap;

#[derive(Resource, Default)]
pub struct DeleteConfig;

pub struct DeletePlugin;

impl Plugin for DeletePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DeleteConfig>()
            .add_systems(Update, (handle_delete_action,).in_set(EditSet));
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_delete_action(
    mut commands: Commands,
    tool_state: Res<ToolState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    curve_query: Query<(Entity, &BezierCurve)>,

    mut selection_state: ResMut<SelectionToolState>,
    selected_query: Query<Entity, With<SelectedControlPoint>>,
) {
    // Only allow delete action when in Select tool (where selections are made)
    if !tool_state.is_currently_using_tool(Tool::Select) {
        return;
    }

    // Check if Delete or Backspace key was just pressed
    if !keyboard.just_pressed(KeyCode::Delete) && !keyboard.just_pressed(KeyCode::Backspace) {
        return;
    }

    debug!(
        "Delete key pressed! Selected points: {}",
        selection_state.selected_points.len(),
    );

    // Check if we have selected points
    if selection_state.selected_points.is_empty() {
        debug!(
            "Cannot perform delete: no points selected. Use selection tool to select points to delete."
        );
        return;
    }

    // Group selected points by curve
    let mut points_by_curve: HashMap<Entity, Vec<usize>> = HashMap::new();
    for selected_point in &selection_state.selected_points {
        points_by_curve
            .entry(selected_point.curve_entity)
            .or_default()
            .push(selected_point.point_index);
    }

    // Process each affected curve
    for (&curve_entity, selected_indices) in &points_by_curve {
        if let Ok((_, curve)) = curve_query.get(curve_entity) {
            let total_points = curve.point_entities.len();

            debug!(
                "Processing curve {curve_entity:?}: {} selected out of {} total points",
                selected_indices.len(),
                total_points
            );

            // If all points are selected, delete the entire curve
            if selected_indices.len() == total_points {
                debug!("All points selected, deleting entire curve {curve_entity:?}");

                // Delete only the curve entity, keep point entities
                commands.entity(curve_entity).despawn();
            } else {
                // Only some points selected, try to update the curve
                debug!(
                    "Partial deletion: removing {} points from curve {curve_entity:?}",
                    selected_indices.len()
                );

                // Sort indices in descending order to remove from back to front
                let mut sorted_indices = selected_indices.clone();
                sorted_indices.sort_unstable();
                sorted_indices.reverse();

                // Create new point entities list excluding selected points
                let mut new_point_entities = curve.point_entities.clone();
                for &index in &sorted_indices {
                    if index < new_point_entities.len() {
                        new_point_entities.remove(index);
                        debug!("Removed point at index {index} from curve");
                    }
                }

                // Check if the remaining curve would be valid (at least 2 points)
                if new_point_entities.len() < 2 {
                    debug!(
                        "Curve {curve_entity:?} would have only {} points after deletion - deleting entire curve",
                        new_point_entities.len()
                    );

                    // Delete only the curve entity, keep point entities
                    commands.entity(curve_entity).despawn();
                } else {
                    // Update the curve with remaining points
                    commands
                        .entity(curve_entity)
                        .insert(BezierCurve::new(new_point_entities));
                    debug!("Updated curve {curve_entity:?} with remaining points");
                }
            }
        }
    }

    // Clear selection after successful delete
    SelectionToolState::clear_selected_points(&mut commands, &selected_query);
    selection_state.reset(&mut commands);
    debug!("Delete operation completed successfully");
}
