use super::common::selected::SelectedControlPoint;
use super::select::SelectionToolState;
use super::tool::{Tool, ToolState};
use crate::EditSet;
use crate::component::curve::{BezierCurve, Point};
use bevy::prelude::*;
use houjing_bezier::merge_curves_sequentially;
use log::debug;
use std::collections::HashSet;

#[derive(Resource, Default)]
pub struct MergeConfig;

pub struct MergePlugin;

impl Plugin for MergePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MergeConfig>()
            .add_systems(Update, (handle_merge_action,).in_set(EditSet));
    }
}

/// Create new curve entities from merged curve data
fn create_merged_curves(
    merged_curves: Vec<Vec<Vec2>>,
    original_curves: &[Entity],
    curve_query: &Query<(Entity, &BezierCurve)>,
    commands: &mut Commands,
) -> Vec<Entity> {
    let mut new_curve_entities = Vec::new();

    for merged_curve_points in merged_curves {
        // Create point entities for the merged curve
        let mut point_entities = Vec::new();
        for point_pos in merged_curve_points {
            let point_entity = commands.spawn(Point::new(point_pos)).id();
            point_entities.push(point_entity);
        }

        let point_count = point_entities.len();

        // Create the new curve entity
        let curve_entity = commands.spawn(BezierCurve::new(point_entities)).id();
        new_curve_entities.push(curve_entity);

        debug!("Created merged curve {curve_entity:?} with {point_count} points");
    }

    // Clean up original curves and their points
    let mut all_point_entities = HashSet::new();

    // Collect all point entities from all curves (deduplicates shared points)
    for &curve_entity in original_curves {
        if let Ok((_, curve)) = curve_query.get(curve_entity) {
            all_point_entities.extend(curve.point_entities.iter().copied());
        }
    }

    // Batch despawn all curve entities
    for &curve_entity in original_curves {
        commands.entity(curve_entity).despawn();
    }

    // Batch despawn all unique point entities
    for point_entity in all_point_entities {
        commands.entity(point_entity).despawn();
    }

    debug!("Cleaned up {} original curves", original_curves.len());
    new_curve_entities
}

#[allow(clippy::too_many_arguments)]
fn handle_merge_action(
    mut commands: Commands,
    tool_state: Res<ToolState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    curve_query: Query<(Entity, &BezierCurve)>,
    point_query: Query<&Point>,
    mut selection_state: ResMut<SelectionToolState>,
    selected_query: Query<Entity, With<SelectedControlPoint>>,
) {
    // Allow merge action to work when in Select tool (where selections are made)
    // or when explicitly in Merge tool
    if !tool_state.is_currently_using_tool(Tool::Merge)
        && !tool_state.is_currently_using_tool(Tool::Select)
    {
        return;
    }

    // Check if M key was just pressed
    if !keyboard.just_pressed(KeyCode::KeyM) {
        return;
    }

    debug!(
        "M key pressed! Current tool: {:?}, Selected points: {}",
        tool_state.current(),
        selection_state.selected_points.len(),
    );

    // Check if we have selected points
    if selection_state.selected_points.is_empty() {
        debug!(
            "Cannot perform merge: no points selected. Use selection tool to select points from curves to merge."
        );
        return;
    }

    // Find all unique curves that contain the selected points
    let mut involved_curves: HashSet<Entity> = HashSet::new();
    for selected_point in &selection_state.selected_points {
        involved_curves.insert(selected_point.curve_entity);
    }

    if involved_curves.len() < 2 {
        debug!(
            "Need at least 2 curves to merge, found {}",
            involved_curves.len()
        );
        return;
    }

    let curve_entities: Vec<Entity> = involved_curves.into_iter().collect();

    debug!(
        "Attempting to merge {} curves with selected points: {:?}",
        curve_entities.len(),
        curve_entities
    );

    // Extract curve position data for the library function
    let mut curve_positions = Vec::new();
    for &curve_entity in &curve_entities {
        if let Ok((_, curve)) = curve_query.get(curve_entity) {
            if let Some(positions) = curve.resolve_positions(&point_query) {
                curve_positions.push(positions);
            } else {
                debug!("Failed to resolve positions for curve {curve_entity:?}");
                return;
            }
        } else {
            debug!("Failed to get curve data for entity {curve_entity:?}");
            return;
        }
    }

    // Perform the merge using the library function
    let merged_curves = merge_curves_sequentially(curve_positions.clone());

    // Check if any merges actually happened
    if merged_curves.len() == curve_positions.len() {
        debug!("No curves could be merged - they may not be adjacent or compatible");
        return;
    }

    debug!(
        "Merge successful: {} curves merged into {} curves",
        curve_entities.len(),
        merged_curves.len()
    );

    // Create new curve entities and clean up old ones
    let _new_curve_entities =
        create_merged_curves(merged_curves, &curve_entities, &curve_query, &mut commands);

    // Clear selection after successful merge
    selection_state.reset(&mut commands, &selected_query);
    debug!("Merge operation completed successfully");
}
