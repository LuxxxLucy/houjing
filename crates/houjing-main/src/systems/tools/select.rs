use super::common::point_finding::find_closest_point;
use super::common::selected::SelectedControlPoint;
use super::cursor::*;
use super::tool::{Tool, ToolState};
use crate::component::curve::{BezierCurve, Point, find_curve_containing_point};
use crate::{InputSet, ShowSet};
use bevy::prelude::*;
use log::debug;

#[derive(Resource, Default)]
pub struct SelectionToolState {
    pub selected_points: Vec<SelectedControlPoint>,
}

impl SelectionToolState {
    pub fn reset(&mut self, _commands: &mut Commands) {
        self.selected_points.clear();
    }

    pub fn clear_selected_points(
        commands: &mut Commands,
        selected_query: &Query<Entity, With<SelectedControlPoint>>,
    ) {
        // Clear any existing selection entities
        for entity in selected_query.iter() {
            commands.entity(entity).despawn();
        }
    }
}

// Default selection configuration constants
const DEFAULT_CONTROL_POINT_COLOR: Color = Color::RED;
const DEFAULT_SELECTED_POINT_COLOR: Color = Color::YELLOW;
const DEFAULT_CONTROL_POINT_RADIUS: f32 = 8.0;
const DEFAULT_SELECTION_RADIUS: f32 = 15.0;

#[derive(Resource)]
pub struct SelectionConfig {
    pub control_point_color: Color,
    pub selected_point_color: Color,
    pub control_point_radius: f32,
    pub selection_radius: f32,
}

impl Default for SelectionConfig {
    fn default() -> Self {
        Self {
            control_point_color: DEFAULT_CONTROL_POINT_COLOR,
            selected_point_color: DEFAULT_SELECTED_POINT_COLOR,
            control_point_radius: DEFAULT_CONTROL_POINT_RADIUS,
            selection_radius: DEFAULT_SELECTION_RADIUS,
        }
    }
}

pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectionToolState>()
            .init_resource::<SelectionConfig>()
            .add_systems(Update, (handle_point_selection,).in_set(InputSet))
            .add_systems(Update, (render_selection_control_points,).in_set(ShowSet));
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_point_selection(
    mut commands: Commands,
    cursor_state: Res<CursorState>,
    tool_state: Res<ToolState>,
    mut selection_state: ResMut<SelectionToolState>,
    config: Res<SelectionConfig>,
    curve_query: Query<(Entity, &BezierCurve)>,
    point_query: Query<(Entity, &Point)>,
    selected_query: Query<Entity, With<SelectedControlPoint>>,
) {
    // Check if tool is active, reset state if not
    if !tool_state.is_currently_using_tool(Tool::Select) {
        // merge tool needs selection, so we don't reset the selection state
        if !tool_state.is_currently_using_tool(Tool::Merge) {
            SelectionToolState::clear_selected_points(&mut commands, &selected_query);
            selection_state.reset(&mut commands);
        }
        return;
    }

    if !cursor_state.mouse_just_pressed {
        return;
    }

    // Clear existing selections
    SelectionToolState::clear_selected_points(&mut commands, &selected_query);
    selection_state.reset(&mut commands);

    // Find closest point using shared utility
    if let Some(point_entity) = find_closest_point(
        cursor_state.cursor_position,
        &point_query,
        config.selection_radius,
    ) {
        // Check if this point is part of a curve
        if let Some((curve_entity, point_index)) =
            find_curve_containing_point(point_entity, &curve_query)
        {
            debug!(
                "Found point: curve {curve_entity:?}, index {point_index}, entity {point_entity:?}"
            );

            let selected_point = SelectedControlPoint {
                curve_entity,
                point_index,
                point_entity,
            };

            // Add to our state
            selection_state.selected_points.push(selected_point);

            // Spawn entity for other systems to query
            commands.spawn(selected_point);
        }
    }
}

fn render_selection_control_points(
    mut gizmos: Gizmos,
    config: Res<SelectionConfig>,
    curve_query: Query<(Entity, &BezierCurve)>,
    point_query: Query<(Entity, &Point)>,
    selected_query: Query<&SelectedControlPoint>,
) {
    let selected_points: Vec<(Entity, usize)> = selected_query
        .iter()
        .map(|scp| (scp.curve_entity, scp.point_index))
        .collect();

    for (curve_entity, curve) in curve_query.iter() {
        for (i, &point_entity) in curve.point_entities.iter().enumerate() {
            if let Ok((_, point_pos)) = point_query.get(point_entity) {
                let is_selected = selected_points.contains(&(curve_entity, i));
                let color = if is_selected {
                    config.selected_point_color
                } else {
                    config.control_point_color
                };

                gizmos.circle_2d(point_pos.position(), config.control_point_radius, color);
            }
        }
    }
}
