use crate::components::*;
use crate::input::*;
use crate::params::*;
use bevy::prelude::*;
use log::debug;

pub fn handle_point_selection(
    mut commands: Commands,
    input_state: Res<InputState>,
    mouse_pos: Res<MouseWorldPos>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    point_query: Query<(Entity, &ControlPoint), Without<Selected>>,
    selected_query: Query<(Entity, &ControlPoint), With<Selected>>,
    mut mesh_materials: Query<&mut Handle<ColorMaterial>>,
) {
    if !input_state.mouse_just_pressed {
        return;
    }

    // First, check if we're clicking on an already selected point
    let mut clicking_on_selected = false;
    for (_, control_point) in selected_query.iter() {
        let distance = mouse_pos.0.distance(control_point.position);
        if distance < SELECTION_RADIUS {
            clicking_on_selected = true;
            break;
        }
    }

    // If clicking on already selected point, don't change selection
    if clicking_on_selected {
        return;
    }

    // Clear existing selections
    for (entity, _) in selected_query.iter() {
        commands.entity(entity).remove::<Selected>();
        if let Ok(mut material_handle) = mesh_materials.get_mut(entity) {
            *material_handle = materials.add(ColorMaterial::from(CONTROL_POINT_COLOR));
        }
    }

    // Find closest control point
    let mut closest_point = None;
    let mut closest_distance = f32::INFINITY;

    for (entity, control_point) in point_query.iter() {
        let distance = mouse_pos.0.distance(control_point.position);
        if distance < SELECTION_RADIUS && distance < closest_distance {
            closest_distance = distance;
            closest_point = Some(entity);
        }
    }

    // Select closest point if found
    if let Some(entity) = closest_point {
        commands.entity(entity).insert(Selected);
        if let Ok(mut material_handle) = mesh_materials.get_mut(entity) {
            *material_handle = materials.add(ColorMaterial::from(SELECTED_POINT_COLOR));
        }
        debug!("Selected control point");
    }
}

pub fn handle_point_dragging(
    input_state: Res<InputState>,
    mouse_pos: Res<MouseWorldPos>,
    mut point_query: Query<(&mut ControlPoint, &mut Transform), With<Selected>>,
    mut curve_query: Query<&mut BezierCurve>,
) {
    if !input_state.dragging {
        return;
    }

    for (mut control_point, mut transform) in point_query.iter_mut() {
        // Update control point position
        control_point.position = mouse_pos.0;
        transform.translation = mouse_pos.0.extend(1.0);

        // Update the curve data
        if let Ok(mut curve) = curve_query.get_mut(control_point.curve_entity) {
            if let Some(point) = curve.control_points.get_mut(control_point.point_index) {
                *point = mouse_pos.0;
            }
        }
    }
}
