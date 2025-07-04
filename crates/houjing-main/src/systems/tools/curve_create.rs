use super::cursor::*;
use super::tool::{Tool, ToolState};
use crate::component::curve::BezierCurve;
use crate::{EditSet, ShowSet};
use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use log::debug;

#[derive(Debug, Clone, Default, PartialEq)]
pub enum CurveCreationState {
    #[default]
    Idle,
    CollectingPoints,
}

#[derive(Resource, Default)]
pub struct CurveCreationToolState {
    pub curve_creation_points: Vec<Vec2>,
    pub curve_creation_state: CurveCreationState,
    pub last_point: Option<Vec2>,
}

impl CurveCreationToolState {
    pub fn reset(&mut self) {
        self.curve_creation_points.clear();
        self.curve_creation_state = CurveCreationState::Idle;
        self.last_point = None;
    }
}

// Default curve creation configuration constants
const DEFAULT_POINT_COLOR: Color = Color::BLUE;
const DEFAULT_POINT_RADIUS: f32 = 6.0;
const DEFAULT_DUPLICATE_THRESHOLD: f32 = 1.0;
const DEFAULT_CURVE_CREATION_Z_LAYER: f32 = 2.0;

#[derive(Resource)]
struct CurveCreationConfig {
    pub point_color: Color,
    pub point_radius: f32,
    pub duplicate_threshold: f32,
    pub z_layer: f32,
}

impl Default for CurveCreationConfig {
    fn default() -> Self {
        Self {
            point_color: DEFAULT_POINT_COLOR,
            point_radius: DEFAULT_POINT_RADIUS,
            duplicate_threshold: DEFAULT_DUPLICATE_THRESHOLD,
            z_layer: DEFAULT_CURVE_CREATION_Z_LAYER,
        }
    }
}
pub struct CurveCreationPlugin;

impl Plugin for CurveCreationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurveCreationToolState>()
            .init_resource::<CurveCreationConfig>()
            .add_systems(Update, (handle_curve_creation,).in_set(EditSet))
            .add_systems(Update, (render_curve_creation_points,).in_set(ShowSet));
    }
}

fn handle_curve_creation(
    mut commands: Commands,
    mut curve_creation_state: ResMut<CurveCreationToolState>,
    tool_state: Res<ToolState>,
    cursor_state: Res<CursorState>,
    cursor_pos: Res<CursorWorldPos>,
    config: Res<CurveCreationConfig>,
) {
    if tool_state.current_tool != Tool::CreateCurve {
        return;
    }

    if !cursor_state.cursor_just_pressed {
        return;
    }

    // Check if this is the same point as the last one
    if let Some(last_point) = curve_creation_state.last_point {
        if cursor_pos.0.distance(last_point) < config.duplicate_threshold {
            debug!("DEBUG: Ignoring duplicate point at {:?}", cursor_pos.0);
            return;
        }
    }

    debug!(
        "DEBUG: Tool: {:?}, State: {:?}, Points: {}/4",
        tool_state.current_tool,
        curve_creation_state.curve_creation_state,
        curve_creation_state.curve_creation_points.len()
    );

    match curve_creation_state.curve_creation_state {
        CurveCreationState::Idle => {
            // Start collecting points - should have 0 points here
            debug!("DEBUG: In Idle state, clearing points and starting new curve");
            curve_creation_state.reset(); // Ensure we start fresh
            curve_creation_state
                .curve_creation_points
                .push(cursor_pos.0);
            curve_creation_state.last_point = Some(cursor_pos.0);
            curve_creation_state.curve_creation_state = CurveCreationState::CollectingPoints;
            debug!(
                "Started cubic Bézier curve creation. Added point: {:?} (total: 1/4)",
                cursor_pos.0
            );
        }
        CurveCreationState::CollectingPoints => {
            curve_creation_state
                .curve_creation_points
                .push(cursor_pos.0);
            curve_creation_state.last_point = Some(cursor_pos.0);
            let point_count = curve_creation_state.curve_creation_points.len();
            debug!("Added point: {:?} (total: {}/4)", cursor_pos.0, point_count);

            if point_count == 4 {
                // Automatically create the curve
                let curve = BezierCurve::new(curve_creation_state.curve_creation_points.clone());
                commands.spawn(curve);

                // Reset state for next curve
                curve_creation_state.reset();
                debug!("Created cubic Bézier curve! State reset to Idle. Ready for next curve.")
            }
        }
    }
}

fn render_curve_creation_points(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    tool_state: Res<ToolState>,
    curve_creation_state: Res<CurveCreationToolState>,
    config: Res<CurveCreationConfig>,
    existing_points: Query<(Entity, &CurveCreationPoint)>,
) {
    // Clear existing creation points if not in create mode
    if tool_state.current_tool != Tool::CreateCurve {
        for (entity, _) in existing_points.iter() {
            commands.entity(entity).despawn();
        }
        return;
    }

    // Only render if we have points
    if curve_creation_state.curve_creation_points.is_empty() {
        // Clear existing points if we have none
        for (entity, _) in existing_points.iter() {
            commands.entity(entity).despawn();
        }
        return;
    }

    // Check if we need to update the rendered points
    let existing_count = existing_points.iter().count();
    if existing_count == curve_creation_state.curve_creation_points.len() {
        return; // No change needed
    }

    debug!(
        "DEBUG RENDER: Updating points. Existing: {}, Current: {}",
        existing_count,
        curve_creation_state.curve_creation_points.len()
    );

    // Clear existing creation points
    for (entity, _) in existing_points.iter() {
        commands.entity(entity).despawn();
    }

    // Render new points
    for &point_pos in curve_creation_state.curve_creation_points.iter() {
        let circle_mesh = Circle::new(config.point_radius);
        let mesh_handle = meshes.add(circle_mesh);
        let material_handle = materials.add(ColorMaterial::from(config.point_color));

        commands.spawn((
            MaterialMesh2dBundle {
                mesh: Mesh2dHandle(mesh_handle),
                material: material_handle,
                transform: Transform::from_translation(point_pos.extend(config.z_layer)),
                ..default()
            },
            CurveCreationPoint,
        ));
    }
}

#[derive(Component)]
pub struct CurveCreationPoint;
