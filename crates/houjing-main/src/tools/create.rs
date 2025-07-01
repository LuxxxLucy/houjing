use super::{CreationState, Tool, ToolState};
use crate::components::*;
use crate::input::*;
use crate::params::*;
use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use log::debug;

pub fn handle_curve_creation(
    mut commands: Commands,
    mut tool_state: ResMut<ToolState>,
    input_state: Res<InputState>,
    mouse_pos: Res<MouseWorldPos>,
) {
    if tool_state.current_tool != Tool::Create {
        return;
    }

    if !input_state.mouse_just_pressed {
        return;
    }

    // Check if this is the same point as the last one
    if let Some(last_point) = tool_state.last_point {
        if mouse_pos.0.distance(last_point) < DUPLICATE_POINT_THRESHOLD {
            debug!("DEBUG: Ignoring duplicate point at {:?}", mouse_pos.0);
            return;
        }
    }

    debug!(
        "DEBUG: Tool: {:?}, State: {:?}, Points: {}/4",
        tool_state.current_tool,
        tool_state.creation_state,
        tool_state.creation_points.len()
    );

    match tool_state.creation_state {
        CreationState::Idle => {
            // Start collecting points - should have 0 points here
            debug!("DEBUG: In Idle state, clearing points and starting new curve");
            tool_state.reset(); // Ensure we start fresh
            tool_state.creation_points.push(mouse_pos.0);
            tool_state.last_point = Some(mouse_pos.0);
            tool_state.creation_state = CreationState::CollectingPoints;
            debug!(
                "Started cubic Bézier curve creation. Added point: {:?} (total: 1/4)",
                mouse_pos.0
            );
        }
        CreationState::CollectingPoints => {
            tool_state.creation_points.push(mouse_pos.0);
            tool_state.last_point = Some(mouse_pos.0);
            let point_count = tool_state.creation_points.len();
            debug!("Added point: {:?} (total: {}/4)", mouse_pos.0, point_count);

            if point_count == 4 {
                // Automatically create the curve
                let curve = BezierCurve::new(tool_state.creation_points.clone());
                commands.spawn(curve);

                // Reset state for next curve
                tool_state.reset();
                debug!("Created cubic Bézier curve! State reset to Idle. Ready for next curve.")
            }
        }
    }
}

pub fn render_creation_points(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    tool_state: Res<ToolState>,
    existing_points: Query<(Entity, &CreationPoint)>,
) {
    // Clear existing creation points if not in create mode
    if tool_state.current_tool != Tool::Create {
        for (entity, _) in existing_points.iter() {
            commands.entity(entity).despawn();
        }
        return;
    }

    // Only render if we have points
    if tool_state.creation_points.is_empty() {
        // Clear existing points if we have none
        for (entity, _) in existing_points.iter() {
            commands.entity(entity).despawn();
        }
        return;
    }

    // Check if we need to update the rendered points
    let existing_count = existing_points.iter().count();
    if existing_count == tool_state.creation_points.len() {
        return; // No change needed
    }

    debug!(
        "DEBUG RENDER: Updating points. Existing: {}, Current: {}",
        existing_count,
        tool_state.creation_points.len()
    );

    // Clear existing creation points
    for (entity, _) in existing_points.iter() {
        commands.entity(entity).despawn();
    }

    // Render new points
    for (i, &point_pos) in tool_state.creation_points.iter().enumerate() {
        let circle_mesh = Circle::new(CREATION_POINT_RADIUS);
        let mesh_handle = meshes.add(circle_mesh);
        let material_handle = materials.add(ColorMaterial::from(CREATION_POINT_COLOR));

        commands.spawn((
            MaterialMesh2dBundle {
                mesh: Mesh2dHandle(mesh_handle),
                material: material_handle,
                transform: Transform::from_translation(point_pos.extend(CREATION_POINT_Z_LAYER)),
                ..default()
            },
            CreationPoint { index: i },
        ));
    }
}

#[derive(Component)]
pub struct CreationPoint {
    pub index: usize,
}
