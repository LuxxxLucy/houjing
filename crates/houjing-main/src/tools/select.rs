use crate::component::curve::{BezierCurve, CurveNeedsUpdate};
use crate::input::mouse::*;
use crate::tools::{Tool, ToolState};
use crate::{EditSet, InputSet, ShowSet};
use bevy::prelude::*;
use log::debug;

// Default selection configuration constants
const DEFAULT_CONTROL_POINT_COLOR: Color = Color::RED;
const DEFAULT_SELECTED_POINT_COLOR: Color = Color::YELLOW;
const DEFAULT_CONTROL_POINT_RADIUS: f32 = 8.0;
const DEFAULT_SELECTION_RADIUS: f32 = 15.0;
const DEFAULT_SELECTION_Z_LAYER: f32 = 1.0;

#[derive(Resource)]
pub struct SelectionConfig {
    pub control_point_color: Color,
    pub selected_point_color: Color,
    pub control_point_radius: f32,
    pub selection_radius: f32,
    pub z_layer: f32,
}

impl Default for SelectionConfig {
    fn default() -> Self {
        Self {
            control_point_color: DEFAULT_CONTROL_POINT_COLOR,
            selected_point_color: DEFAULT_SELECTED_POINT_COLOR,
            control_point_radius: DEFAULT_CONTROL_POINT_RADIUS,
            selection_radius: DEFAULT_SELECTION_RADIUS,
            z_layer: DEFAULT_SELECTION_Z_LAYER,
        }
    }
}

#[derive(Component)]
pub struct SelectedControlPoint {
    pub curve_entity: Entity,
    pub point_index: usize,
}

pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectionConfig>()
            .add_systems(Update, (handle_point_selection,).in_set(InputSet))
            .add_systems(Update, (handle_point_dragging,).in_set(EditSet))
            .add_systems(Update, (render_control_points,).in_set(ShowSet));
    }
}

fn handle_point_selection(
    mut commands: Commands,
    input_state: Res<MouseState>,
    mouse_pos: Res<MouseWorldPos>,
    tool_state: Res<ToolState>,
    config: Res<SelectionConfig>,
    curve_query: Query<(Entity, &BezierCurve)>,
    selected_query: Query<Entity, With<SelectedControlPoint>>,
) {
    // Only handle selection when in select tool
    if tool_state.current_tool != Tool::Select {
        return;
    }

    if !input_state.mouse_just_pressed {
        return;
    }

    // Clear existing selections
    for entity in selected_query.iter() {
        commands.entity(entity).despawn();
    }

    // Find closest control point
    let mut closest_point = None;
    let mut closest_distance = f32::INFINITY;

    for (curve_entity, curve) in curve_query.iter() {
        for (point_index, &point_pos) in curve.control_points.iter().enumerate() {
            let distance = mouse_pos.0.distance(point_pos);
            if distance < config.selection_radius && distance < closest_distance {
                closest_distance = distance;
                closest_point = Some((curve_entity, point_index));
            }
        }
    }

    // Select closest point if found
    if let Some((curve_entity, point_index)) = closest_point {
        commands.spawn(SelectedControlPoint {
            curve_entity,
            point_index,
        });
        debug!("Selected control point {point_index} of curve {curve_entity:?}");
    }
}

fn handle_point_dragging(
    input_state: Res<MouseState>,
    mouse_pos: Res<MouseWorldPos>,
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
                *point = mouse_pos.0;

                // Mark curve for mesh update
                commands
                    .entity(selected_point.curve_entity)
                    .insert(CurveNeedsUpdate);
            }
        }
    }
}

fn render_control_points(
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
