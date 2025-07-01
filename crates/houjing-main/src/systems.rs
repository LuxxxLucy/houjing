use crate::components::*;
use crate::params::*;
use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};

#[derive(Component)]
pub struct NeedsUpdate;

pub fn render_curves(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    curve_query: Query<(Entity, &BezierCurve), Without<Handle<Mesh>>>,
) {
    for (entity, curve) in curve_query.iter() {
        let mesh = create_curve_mesh(curve);
        let mesh_handle = meshes.add(mesh);
        let material_handle = materials.add(ColorMaterial::from(CURVE_COLOR));

        commands.entity(entity).insert((MaterialMesh2dBundle {
            mesh: Mesh2dHandle(mesh_handle),
            material: material_handle,
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, CURVE_Z_LAYER)),
            ..default()
        },));
    }
}

pub fn render_control_points(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    curve_query: Query<(Entity, &BezierCurve)>,
    existing_points: Query<(Entity, &ControlPoint)>,
) {
    // Only create control points if they don't exist
    for (curve_entity, curve) in curve_query.iter() {
        for (i, &point_pos) in curve.control_points.iter().enumerate() {
            // Check if control point already exists for this curve and index
            let point_exists = existing_points
                .iter()
                .any(|(_, cp)| cp.curve_entity == curve_entity && cp.point_index == i);

            if !point_exists {
                let circle_mesh = Circle::new(CONTROL_POINT_RADIUS);
                let mesh_handle = meshes.add(circle_mesh);
                let material_handle = materials.add(ColorMaterial::from(CONTROL_POINT_COLOR));

                commands.spawn((
                    MaterialMesh2dBundle {
                        mesh: Mesh2dHandle(mesh_handle),
                        material: material_handle,
                        transform: Transform::from_translation(
                            point_pos.extend(CONTROL_POINT_Z_LAYER),
                        ),
                        ..default()
                    },
                    ControlPoint {
                        position: point_pos,
                        curve_entity,
                        point_index: i,
                    },
                ));
            }
        }
    }
}

pub fn mark_curves_for_update(
    mut commands: Commands,
    selected_points: Query<&ControlPoint, With<Selected>>,
    input_state: Res<crate::input::InputState>,
) {
    if input_state.dragging {
        for control_point in selected_points.iter() {
            commands
                .entity(control_point.curve_entity)
                .insert(NeedsUpdate);
        }
    }
}

pub fn update_curve_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    curve_query: Query<(Entity, &BezierCurve, &Mesh2dHandle), With<NeedsUpdate>>,
) {
    for (entity, curve, mesh_handle) in curve_query.iter() {
        let new_mesh = create_curve_mesh(curve);
        meshes.insert(mesh_handle.0.clone(), new_mesh);
        commands.entity(entity).remove::<NeedsUpdate>();
    }
}

fn create_curve_mesh(curve: &BezierCurve) -> Mesh {
    let segments = 50;
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let point = curve.evaluate(t);
        vertices.push([point.x, point.y, 0.0]);

        if i < segments {
            indices.push(i);
            indices.push(i + 1);
        }
    }

    let mut mesh = Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::LineList,
        bevy::render::render_asset::RenderAssetUsages::MAIN_WORLD
            | bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
    );

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));

    mesh
}
