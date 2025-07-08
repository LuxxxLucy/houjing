use super::common::point_finding::find_closest_control_point;
use super::common::selected::SelectedControlPoint;
use super::cursor::*;
use super::tool::{Tool, ToolState};
use crate::component::curve::BezierCurve;
use crate::{InputSet, ShowSet};
use bevy::prelude::*;
use log::debug;

#[derive(Resource, Default)]
pub struct SelectionToolState {
    pub selected_points: Vec<SelectedControlPoint>,
}

impl SelectionToolState {
    pub fn reset(
        &mut self,
        commands: &mut Commands,
        selected_query: &Query<Entity, With<SelectedControlPoint>>,
    ) {
        // Clear any existing selection entities
        for entity in selected_query.iter() {
            commands.entity(entity).despawn();
        }
        self.selected_points.clear();
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
    cursor_pos: Res<CursorWorldPos>,
    tool_state: Res<ToolState>,
    mut selection_state: ResMut<SelectionToolState>,
    config: Res<SelectionConfig>,
    curve_query: Query<(Entity, &BezierCurve)>,
    selected_query: Query<Entity, With<SelectedControlPoint>>,
) {
    // Check if tool is active, reset state if not
    if !tool_state.is_currently_using_tool(Tool::Select) {
        selection_state.reset(&mut commands, &selected_query);
        return;
    }

    if !cursor_state.cursor_just_pressed {
        return;
    }

    // Clear existing selections
    selection_state.reset(&mut commands, &selected_query);

    // Find closest control point using shared utility
    if let Some(found_point) =
        find_closest_control_point(cursor_pos.0, &curve_query, config.selection_radius)
    {
        let curve_entity = found_point.curve_entity;
        let point_index = found_point.point_index;
        let selected_point = SelectedControlPoint {
            curve_entity,
            point_index,
        };

        // Add to our state
        selection_state.selected_points.push(SelectedControlPoint {
            curve_entity,
            point_index,
        });

        // Spawn entity for other systems to query
        commands.spawn(selected_point);
        debug!("Selected control point {point_index} of curve {curve_entity:?}");
    }
}

fn render_selection_control_points(
    mut gizmos: Gizmos,
    config: Res<SelectionConfig>,
    curve_query: Query<(Entity, &BezierCurve)>,
    selected_query: Query<&SelectedControlPoint>,
) {
    let selected_points: Vec<(Entity, usize)> = selected_query
        .iter()
        .map(|scp| (scp.curve_entity, scp.point_index))
        .collect();

    for (curve_entity, curve) in curve_query.iter() {
        for (i, &point_pos) in curve.control_points.iter().enumerate() {
            let is_selected = selected_points.contains(&(curve_entity, i));
            let color = if is_selected {
                config.selected_point_color
            } else {
                config.control_point_color
            };

            gizmos.circle_2d(point_pos, config.control_point_radius, color);
        }
    }
}
