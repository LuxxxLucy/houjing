use crate::components::*;
use crate::input::*;
use crate::params::*;
use crate::systems::NeedsUpdate;
use bevy::prelude::*;
use log::debug;

pub fn handle_point_selection(
    mut commands: Commands,
    input_state: Res<InputState>,
    mouse_pos: Res<MouseWorldPos>,
    curve_query: Query<(Entity, &BezierCurve)>,
    selected_query: Query<Entity, With<SelectedControlPoint>>,
) {
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
            if distance < SELECTION_RADIUS && distance < closest_distance {
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

pub fn handle_point_dragging(
    input_state: Res<InputState>,
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
                    .insert(NeedsUpdate);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::world::World;

    #[test]
    fn test_selected_control_point_component() {
        let mut world = World::new();

        // Create a curve entity
        let curve_entity = world
            .spawn(BezierCurve::new(vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(50.0, 100.0),
                Vec2::new(100.0, 0.0),
            ]))
            .id();

        // Create a selected control point entity
        let selected_entity = world
            .spawn(SelectedControlPoint {
                curve_entity,
                point_index: 1,
            })
            .id();

        // Verify the selected control point data
        let selected = world.get::<SelectedControlPoint>(selected_entity).unwrap();
        assert_eq!(selected.curve_entity, curve_entity);
        assert_eq!(selected.point_index, 1);
    }
}
