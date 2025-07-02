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
    mut gizmos: Gizmos,
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
                SELECTED_POINT_COLOR
            } else {
                CONTROL_POINT_COLOR
            };

            gizmos.circle_2d(point_pos, CONTROL_POINT_RADIUS, color);
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

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::MinimalPlugins;

    #[test]
    fn test_needs_update_component() {
        use bevy::ecs::world::World;

        let mut world = World::new();
        let entity = world.spawn(BezierCurve::new(vec![Vec2::ZERO])).id();

        // Add NeedsUpdate
        world.entity_mut(entity).insert(NeedsUpdate);
        assert!(world.get::<NeedsUpdate>(entity).is_some());

        // Remove NeedsUpdate
        world.entity_mut(entity).remove::<NeedsUpdate>();
        assert!(world.get::<NeedsUpdate>(entity).is_none());
    }

    #[test]
    fn test_curve_system_integration() {
        let mut app = App::new();

        app.add_plugins(MinimalPlugins)
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<ColorMaterial>>()
            .add_systems(Update, (render_curves, update_curve_meshes));

        // Create a test curve
        let initial_points = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(50.0, 100.0),
            Vec2::new(100.0, 0.0),
        ];
        let curve = BezierCurve::new(initial_points);
        let curve_entity = app.world.spawn(curve).id();

        // Run one update to create the mesh components
        app.update();

        // Modify the curve control points
        let new_points = vec![
            Vec2::new(10.0, 10.0),
            Vec2::new(60.0, 110.0),
            Vec2::new(110.0, 10.0),
        ];

        if let Some(mut curve) = app.world.get_mut::<BezierCurve>(curve_entity) {
            curve.control_points = new_points.clone();
        }

        // Add NeedsUpdate component to trigger re-rendering
        app.world.entity_mut(curve_entity).insert(NeedsUpdate);

        // Run update to process the mesh update
        app.update();

        // Verify curve has updated control points
        let curve = app.world.get::<BezierCurve>(curve_entity).unwrap();
        assert_eq!(curve.control_points.len(), 3);

        for (i, &expected_point) in new_points.iter().enumerate() {
            assert_eq!(curve.control_points[i], expected_point);
        }

        // Verify NeedsUpdate was removed after processing
        assert!(app.world.get::<NeedsUpdate>(curve_entity).is_none());
    }
}
