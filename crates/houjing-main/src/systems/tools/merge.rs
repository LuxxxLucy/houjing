use super::common::selected::SelectedControlPoint;
use super::select::SelectionToolState;
use super::tool::{Tool, ToolState};
use crate::component::curve::{BezierCurve, Point};
use crate::{EditSet, InputSet};
use bevy::prelude::*;
use houjing_bezier::merge_split_bezier_curves;
use log::debug;
use std::collections::HashSet;

#[derive(Resource, Default)]
pub struct MergeConfig;

#[derive(Resource, Default)]
pub struct MergeToolState {
    pub can_merge: bool,
    pub mergeable_curves: Option<(Entity, Entity)>,
}

impl MergeToolState {
    pub fn reset(&mut self, _commands: &mut Commands) {
        self.can_merge = false;
        self.mergeable_curves = None;
    }
}

pub struct MergePlugin;

impl Plugin for MergePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MergeConfig>()
            .init_resource::<MergeToolState>()
            .add_systems(Update, (update_merge_preview,).in_set(InputSet))
            .add_systems(Update, (handle_merge_action,).in_set(EditSet));
    }
}

#[allow(clippy::too_many_arguments)]
fn update_merge_preview(
    mut merge_state: ResMut<MergeToolState>,
    tool_state: Res<ToolState>,
    selection_state: Res<SelectionToolState>,
    curve_query: Query<(Entity, &BezierCurve)>,
    point_query: Query<&Point>,
    mut commands: Commands,
) {
    // Allow merge to work when in Select tool (since that's where selections are made)
    // or when explicitly in Merge tool
    if !tool_state.is_currently_using_tool(Tool::Merge)
        && !tool_state.is_currently_using_tool(Tool::Select)
    {
        merge_state.reset(&mut commands);
        return;
    }

    // Reset merge state
    merge_state.can_merge = false;
    merge_state.mergeable_curves = None;

    // Need selected points to work with
    if selection_state.selected_points.is_empty() {
        return;
    }

    // Find all unique curves that contain the selected points
    let mut involved_curves: HashSet<Entity> = HashSet::new();
    for selected_point in &selection_state.selected_points {
        involved_curves.insert(selected_point.curve_entity);
    }

    // We need exactly 2 curves to merge
    if involved_curves.len() != 2 {
        return;
    }

    let curves: Vec<Entity> = involved_curves.into_iter().collect();
    let curve1_entity = curves[0];
    let curve2_entity = curves[1];

    // Get the curve data
    let curve1_data = curve_query.get(curve1_entity);
    let curve2_data = curve_query.get(curve2_entity);

    if let (Ok((_, curve1)), Ok((_, curve2))) = (curve1_data, curve2_data) {
        // Check if the curves can be merged
        if let Some((left_curve, right_curve)) = determine_merge_order(curve1, curve2, &point_query)
        {
            // Try to merge using the bezier math
            if let Some(_merged_points) = merge_split_bezier_curves(&left_curve, &right_curve) {
                merge_state.can_merge = true;
                merge_state.mergeable_curves = Some((curve1_entity, curve2_entity));
                debug!("Curves can be merged!");
            } else {
                debug!("Curves cannot be merged mathematically");
            }
        }
    }
}

/// Determine the correct order for merging two curves
/// Returns (left_curve_points, right_curve_points) if they can be merged, None otherwise
fn determine_merge_order(
    curve1: &BezierCurve,
    curve2: &BezierCurve,
    point_query: &Query<&Point>,
) -> Option<(Vec<Vec2>, Vec<Vec2>)> {
    let curve1_points = curve1.resolve_positions(point_query)?;
    let curve2_points = curve2.resolve_positions(point_query)?;

    // Both curves must have the same number of control points
    if curve1_points.len() != curve2_points.len() {
        debug!(
            "Cannot merge: different number of control points ({} vs {})",
            curve1_points.len(),
            curve2_points.len()
        );
        return None;
    }

    // Use more forgiving tolerance for floating point precision
    let tolerance = 1e-3;

    // Check if curve1 end connects to curve2 start (curve1 is left, curve2 is right)
    let curve1_end = curve1_points[curve1_points.len() - 1];
    let curve2_start = curve2_points[0];
    let distance1 = (curve1_end - curve2_start).length();

    if distance1 < tolerance {
        return Some((curve1_points, curve2_points));
    }

    // Check if curve2 end connects to curve1 start (curve2 is left, curve1 is right)
    let curve2_end = curve2_points[curve2_points.len() - 1];
    let curve1_start = curve1_points[0];
    let distance2 = (curve2_end - curve1_start).length();

    if distance2 < tolerance {
        return Some((curve2_points, curve1_points));
    }

    debug!("No connection found between curves. Distances: {distance1} and {distance2}");
    debug!("Curve1 points: {curve1_points:?}");
    debug!("Curve2 points: {curve2_points:?}");
    None
}

#[allow(clippy::too_many_arguments)]
fn handle_merge_action(
    mut commands: Commands,
    merge_state: Res<MergeToolState>,
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
        "M key pressed! Current tool: {:?}, Selected points: {}, Can merge: {}",
        tool_state.current(),
        selection_state.selected_points.len(),
        merge_state.can_merge
    );

    // Check if we can merge
    if !merge_state.can_merge {
        // Provide more specific error message based on selection state
        if selection_state.selected_points.is_empty() {
            debug!(
                "Cannot perform merge: no points selected. Use wireframe selection to select points from both curves."
            );
        } else {
            let mut involved_curves: HashSet<Entity> = HashSet::new();
            for selected_point in &selection_state.selected_points {
                involved_curves.insert(selected_point.curve_entity);
            }

            if involved_curves.len() != 2 {
                debug!(
                    "Cannot perform merge: need points from exactly 2 curves, found {} curves. Selected {} points.",
                    involved_curves.len(),
                    selection_state.selected_points.len()
                );
            } else {
                debug!(
                    "Cannot perform merge: curves are not adjacent or mathematically incompatible. Make sure the curves were created by splitting a single curve."
                );
            }
        }
        return;
    }

    if let Some((curve1_entity, curve2_entity)) = merge_state.mergeable_curves {
        // Get the curve data
        let curve1_data = curve_query.get(curve1_entity);
        let curve2_data = curve_query.get(curve2_entity);

        if let (Ok((_, curve1)), Ok((_, curve2))) = (curve1_data, curve2_data) {
            if let Some((left_curve, right_curve)) =
                determine_merge_order(curve1, curve2, &point_query)
            {
                if let Some(merged_points) = merge_split_bezier_curves(&left_curve, &right_curve) {
                    debug!(
                        "Merging 2 curves: left curve has {} points, right curve has {} points",
                        left_curve.len(),
                        right_curve.len()
                    );

                    // Determine which curve is left and which is right based on the order
                    let (left_curve_entity, right_curve_entity) =
                        if left_curve == curve1.resolve_positions(&point_query).unwrap() {
                            (curve1_entity, curve2_entity)
                        } else {
                            (curve2_entity, curve1_entity)
                        };

                    // Get the original curves to find the original start and end points
                    let left_curve_data = curve_query.get(left_curve_entity).unwrap().1;
                    let right_curve_data = curve_query.get(right_curve_entity).unwrap().1;

                    let original_start_entity = left_curve_data.point_entities[0];
                    let original_end_entity =
                        right_curve_data.point_entities[right_curve_data.point_entities.len() - 1];

                    // Create new intermediate control points for the merged curve
                    let mut merged_point_entities = vec![original_start_entity];

                    // Create entities for intermediate control points (skip first and last)
                    for &pos in &merged_points[1..merged_points.len() - 1] {
                        let point_entity = commands.spawn(Point::new(pos)).id();
                        merged_point_entities.push(point_entity);
                    }

                    merged_point_entities.push(original_end_entity);

                    // Create the new merged curve
                    commands.spawn(BezierCurve::new(merged_point_entities));

                    // Clean up the original curves and their intermediate control points
                    cleanup_curve_and_intermediate_points(
                        &mut commands,
                        left_curve_entity,
                        &left_curve_data.point_entities,
                        &[original_start_entity, original_end_entity],
                    );
                    cleanup_curve_and_intermediate_points(
                        &mut commands,
                        right_curve_entity,
                        &right_curve_data.point_entities,
                        &[original_start_entity, original_end_entity],
                    );

                    // Clear selection
                    selection_state.reset(&mut commands, &selected_query);

                    debug!("Merge completed successfully");
                } else {
                    debug!("Failed to merge curves mathematically");
                }
            }
        }
    }
}

/// Clean up a curve entity and its intermediate control points
/// Preserves the specified points (typically original start/end points)
fn cleanup_curve_and_intermediate_points(
    commands: &mut Commands,
    curve_entity: Entity,
    point_entities: &[Entity],
    points_to_preserve: &[Entity],
) {
    // Delete the curve entity
    commands.entity(curve_entity).despawn();

    // Delete intermediate control points, but preserve the specified points
    for &point_entity in point_entities {
        if !points_to_preserve.contains(&point_entity) {
            commands.entity(point_entity).despawn();
        }
    }
}
