use crate::ShowSet;
use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};

// Default curve rendering configuration constants
const DEFAULT_CURVE_COLOR: Color = Color::WHITE;
const DEFAULT_CURVE_SEGMENTS: u32 = 50;
const DEFAULT_CURVE_Z_LAYER: f32 = 0.0;

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

#[derive(Component)]
pub struct BezierCurve {
    pub control_points: Vec<Vec2>,
}

// mark the curve as needing update
#[derive(Component)]
pub struct CurveNeedsUpdate;

pub struct CurveRenderingPlugin;

impl Plugin for CurveRenderingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurveRenderingConfig>()
            .add_systems(
                Update,
                (create_new_curves, update_curve_if_needed).in_set(ShowSet),
            )
            .add_systems(Startup, setup_test_curve);
    }
}

fn create_new_curves(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    config: Res<CurveRenderingConfig>,
    curve_query: Query<(Entity, &BezierCurve), Without<Handle<Mesh>>>,
) {
    for (entity, curve) in curve_query.iter() {
        let mesh = create_curve_mesh(curve, &config);
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

fn update_curve_if_needed(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    config: Res<CurveRenderingConfig>,
    curve_query: Query<(Entity, &BezierCurve, &Mesh2dHandle), With<CurveNeedsUpdate>>,
) {
    for (entity, curve, mesh_handle) in curve_query.iter() {
        let new_mesh = create_curve_mesh(curve, &config);
        meshes.insert(mesh_handle.0.clone(), new_mesh);
        commands.entity(entity).remove::<CurveNeedsUpdate>();
    }
}

fn create_curve_mesh(curve: &BezierCurve, config: &CurveRenderingConfig) -> Mesh {
    let segments = config.segments;
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
    fn test_curve_evaluation() {
        let quadratic_curve = BezierCurve::new(vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(50.0, 100.0),
            Vec2::new(100.0, 0.0),
        ]);

        let start_point = quadratic_curve.evaluate(0.0);
        let end_point = quadratic_curve.evaluate(1.0);
        let mid_point = quadratic_curve.evaluate(0.5);

        assert_eq!(start_point, Vec2::new(0.0, 0.0));
        assert_eq!(end_point, Vec2::new(100.0, 0.0));
        assert_eq!(mid_point, Vec2::new(50.0, 50.0));
    }

    #[test]
    fn test_curve_control_points_data() {
        let points = vec![Vec2::new(0.0, 0.0), Vec2::new(100.0, 50.0)];
        let curve = BezierCurve::new(points.clone());

        assert_eq!(curve.control_points.len(), 2);
        assert_eq!(curve.control_points[0], Vec2::new(0.0, 0.0));
        assert_eq!(curve.control_points[1], Vec2::new(100.0, 50.0));
    }

    #[test]
    fn test_curve_system_integration() {
        let mut app = App::new();

        app.add_plugins(MinimalPlugins)
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<ColorMaterial>>()
            .init_resource::<CurveRenderingConfig>()
            .add_systems(Update, (create_new_curves, update_curve_if_needed));

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

        // Add CurveNeedsUpdate component to trigger re-rendering
        app.world.entity_mut(curve_entity).insert(CurveNeedsUpdate);

        // Run update to process the mesh update
        app.update();

        // Verify curve has updated control points
        let curve = app.world.get::<BezierCurve>(curve_entity).unwrap();
        assert_eq!(curve.control_points.len(), 3);

        for (i, &expected_point) in new_points.iter().enumerate() {
            assert_eq!(curve.control_points[i], expected_point);
        }

        // Verify CurveNeedsUpdate was removed after processing
        assert!(app.world.get::<CurveNeedsUpdate>(curve_entity).is_none());
    }
}

// remove it?
fn setup_test_curve(mut commands: Commands) {
    let curve = BezierCurve::new(vec![
        Vec2::new(-200.0, 0.0),
        Vec2::new(0.0, 200.0),
        Vec2::new(200.0, 0.0),
    ]);

    commands.spawn(curve);
}
impl BezierCurve {
    pub fn new(control_points: Vec<Vec2>) -> Self {
        Self { control_points }
    }

    pub fn evaluate(&self, t: f32) -> Vec2 {
        match self.control_points.len() {
            2 => {
                // Linear interpolation
                self.control_points[0].lerp(self.control_points[1], t)
            }
            3 => self.evaluate_quadratic(t),
            4 => self.evaluate_cubic(t),
            _ => panic!("Unsupported number of control points"),
        }
    }

    fn evaluate_quadratic(&self, t: f32) -> Vec2 {
        let p0 = self.control_points[0];
        let p1 = self.control_points[1];
        let p2 = self.control_points[2];

        let one_minus_t = 1.0 - t;
        let one_minus_t_sq = one_minus_t * one_minus_t;
        let t_sq = t * t;

        one_minus_t_sq * p0 + 2.0 * one_minus_t * t * p1 + t_sq * p2
    }

    fn evaluate_cubic(&self, t: f32) -> Vec2 {
        let p0 = self.control_points[0];
        let p1 = self.control_points[1];
        let p2 = self.control_points[2];
        let p3 = self.control_points[3];

        let one_minus_t = 1.0 - t;
        let one_minus_t_sq = one_minus_t * one_minus_t;
        let one_minus_t_cu = one_minus_t_sq * one_minus_t;
        let t_sq = t * t;
        let t_cu = t_sq * t;

        one_minus_t_cu * p0
            + 3.0 * one_minus_t_sq * t * p1
            + 3.0 * one_minus_t * t_sq * p2
            + t_cu * p3
    }
}
