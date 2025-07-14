use crate::ShowSet;
use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use houjing_bezier::evaluate_bezier_curve_segment;

// Default curve rendering configuration constants
const DEFAULT_CURVE_COLOR: Color = Color::WHITE;
const DEFAULT_CURVE_SEGMENTS: u32 = 50;
const DEFAULT_CURVE_Z_LAYER: f32 = 0.0;

/// Component representing a control point position
/// Points are now separate entities that can be shared between curves
#[derive(Component, Debug, Clone, Copy)]
pub struct Point(pub Vec2);

impl Point {
    pub fn new(position: Vec2) -> Self {
        Self(position)
    }

    pub fn position(&self) -> Vec2 {
        self.0
    }

    pub fn set_position(&mut self, position: Vec2) {
        self.0 = position;
    }
}

/// Get the position of a point entity
pub fn get_position(point_entity: Entity, point_query: &Query<(Entity, &Point)>) -> Option<Vec2> {
    point_query
        .get(point_entity)
        .ok()
        .map(|(_, point)| point.position())
}

#[derive(Component)]
pub struct BezierCurve {
    pub point_entities: Vec<Entity>,
}

impl BezierCurve {
    pub fn new(point_entities: Vec<Entity>) -> Self {
        Self { point_entities }
    }

    /// Resolve point entities to their actual positions
    /// Returns None if any point entity is missing or invalid
    pub fn resolve_positions(&self, point_query: &Query<&Point>) -> Option<Vec<Vec2>> {
        let mut positions = Vec::with_capacity(self.point_entities.len());

        for &entity in &self.point_entities {
            let point = point_query.get(entity).ok()?;
            positions.push(point.position());
        }

        Some(positions)
    }

    /// Evaluate a Bezier curve at parameter t given control points
    pub fn evaluate_bezier(control_points: &[Vec2], t: f32) -> Vec2 {
        evaluate_bezier_curve_segment(control_points, t)
    }
}

/// Find which curve contains the given point entity and at what index
/// Returns the curve entity and the index of the point within that curve
pub fn find_curve_containing_point(
    point_entity: Entity,
    curve_query: &Query<(Entity, &BezierCurve)>,
) -> Option<(Entity, usize)> {
    for (curve_entity, curve) in curve_query.iter() {
        if let Some(index) = curve.point_entities.iter().position(|&e| e == point_entity) {
            return Some((curve_entity, index));
        }
    }
    None
}

#[derive(Resource)]
pub struct CurveRenderingConfig {
    pub color: Color,
    pub segments: u32,
    pub z_layer: f32,
}

impl Default for CurveRenderingConfig {
    fn default() -> Self {
        Self {
            color: DEFAULT_CURVE_COLOR,
            segments: DEFAULT_CURVE_SEGMENTS,
            z_layer: DEFAULT_CURVE_Z_LAYER,
        }
    }
}

pub struct CurveRenderingPlugin;

impl Plugin for CurveRenderingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurveRenderingConfig>().add_systems(
            Update,
            (create_new_curves, update_curve_if_needed).in_set(ShowSet),
        );
    }
}

fn create_new_curves(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    config: Res<CurveRenderingConfig>,
    curve_query: Query<(Entity, &BezierCurve), Without<Handle<Mesh>>>,
    point_query: Query<&Point>,
) {
    for (entity, curve) in curve_query.iter() {
        if let Some(mesh) = create_curve_mesh(curve, &config, &point_query) {
            let mesh_handle = meshes.add(mesh);
            let material_handle = materials.add(ColorMaterial::from(config.color));

            commands.entity(entity).insert((MaterialMesh2dBundle {
                mesh: Mesh2dHandle(mesh_handle),
                material: material_handle,
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, config.z_layer)),
                ..default()
            },));
        }
    }
}

fn update_curve_if_needed(
    mut meshes: ResMut<Assets<Mesh>>,
    config: Res<CurveRenderingConfig>,
    curve_query: Query<(Entity, &BezierCurve, &Mesh2dHandle)>,
    changed_points: Query<Entity, (With<Point>, Changed<Point>)>,
    all_points: Query<&Point>,
) {
    // If no points have changed, skip update
    if changed_points.is_empty() {
        return;
    }

    // Get all changed point entities
    let changed_point_entities: std::collections::HashSet<Entity> = changed_points.iter().collect();

    for (_entity, curve, mesh_handle) in curve_query.iter() {
        // Check if this curve references any changed points
        let needs_update = curve
            .point_entities
            .iter()
            .any(|&point_entity| changed_point_entities.contains(&point_entity));

        if needs_update {
            if let Some(new_mesh) = create_curve_mesh(curve, &config, &all_points) {
                meshes.insert(mesh_handle.0.clone(), new_mesh);
            }
        }
    }
}

fn create_curve_mesh(
    curve: &BezierCurve,
    config: &CurveRenderingConfig,
    point_query: &Query<&Point>,
) -> Option<Mesh> {
    let control_points = curve.resolve_positions(point_query)?;
    let segments = config.segments;
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let point = BezierCurve::evaluate_bezier(&control_points, t);
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

    Some(mesh)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::MinimalPlugins;

    #[test]
    fn test_curve_evaluation() {
        let control_points = vec![Vec2::ZERO, Vec2::new(50.0, 100.0), Vec2::new(100.0, 0.0)];

        let start_point = BezierCurve::evaluate_bezier(&control_points, 0.0);
        let end_point = BezierCurve::evaluate_bezier(&control_points, 1.0);
        let mid_point = BezierCurve::evaluate_bezier(&control_points, 0.5);

        assert_eq!(start_point, Vec2::ZERO);
        assert_eq!(end_point, Vec2::new(100.0, 0.0));
        assert_eq!(mid_point, Vec2::new(50.0, 50.0));
    }

    #[test]
    fn test_point_position_component() {
        let point = Point::new(Vec2::new(10.0, 20.0));
        assert_eq!(point.position(), Vec2::new(10.0, 20.0));

        let mut point = Point::new(Vec2::ZERO);
        point.set_position(Vec2::new(30.0, 40.0));
        assert_eq!(point.position(), Vec2::new(30.0, 40.0));
    }

    #[test]
    fn test_curve_system_integration() {
        let mut app = App::new();

        app.add_plugins(MinimalPlugins)
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<ColorMaterial>>()
            .init_resource::<CurveRenderingConfig>()
            .add_systems(Update, (create_new_curves, update_curve_if_needed));

        // Create test point
        let point2 = app.world.spawn(Point::new(Vec2::new(50.0, 100.0))).id();

        // Run one update to create the mesh components
        app.update();

        // Modify a point position
        if let Some(mut point) = app.world.get_mut::<Point>(point2) {
            point.set_position(Vec2::new(60.0, 110.0));
        }

        // Run update to process the automatic change detection
        app.update();

        // Verify point has updated position
        let point = app.world.get::<Point>(point2).unwrap();
        assert_eq!(point.position(), Vec2::new(60.0, 110.0));
    }
}
