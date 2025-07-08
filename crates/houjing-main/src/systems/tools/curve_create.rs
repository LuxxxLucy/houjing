use super::common::point_finding::snap_to_closest_point;
use super::cursor::*;
use super::tool::{Tool, ToolState};
use crate::component::curve::BezierCurve;
use crate::rendering::render_simple_circle;
use crate::{EditSet, ShowSet};
use bevy::prelude::*;
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
    pub point_entities: Vec<Entity>,
}

impl CurveCreationToolState {
    pub fn reset(&mut self, commands: &mut Commands) {
        // Clear all point entities when resetting
        for &entity in &self.point_entities {
            commands.entity(entity).despawn();
        }

        self.curve_creation_state = CurveCreationState::Idle;
        self.curve_creation_points.clear();
        self.last_point = None;
        self.point_entities.clear();
    }
}

// Default curve creation configuration constants
const DEFAULT_POINT_COLOR: Color = Color::BLUE;
const DEFAULT_POINT_RADIUS: f32 = 6.0;
const DEFAULT_DUPLICATE_THRESHOLD: f32 = 1.0;
const DEFAULT_SNAP_THRESHOLD: f32 = 15.0; // Distance threshold for snapping to existing points
const DEFAULT_CURVE_CREATION_Z_LAYER: f32 = 2.0;

#[derive(Resource)]
struct CurveCreationConfig {
    pub point_color: Color,
    pub point_radius: f32,
    pub duplicate_threshold: f32,
    pub snap_threshold: f32,
    pub z_layer: f32,
}

impl Default for CurveCreationConfig {
    fn default() -> Self {
        Self {
            point_color: DEFAULT_POINT_COLOR,
            point_radius: DEFAULT_POINT_RADIUS,
            duplicate_threshold: DEFAULT_DUPLICATE_THRESHOLD,
            snap_threshold: DEFAULT_SNAP_THRESHOLD,
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
    existing_curves: Query<(Entity, &BezierCurve)>,
) {
    // Check if tool is active, reset state if not
    if !tool_state.is_currently_using_tool(Tool::CreateCurve) {
        curve_creation_state.reset(&mut commands);
        return;
    }

    if !cursor_state.cursor_just_pressed {
        return;
    }

    // Find closest existing point within snap threshold using shared utility
    let target_pos = snap_to_closest_point(cursor_pos.0, &existing_curves, config.snap_threshold);

    // Check if this is the same point as the last one
    if let Some(last_point) = curve_creation_state.last_point {
        if target_pos.distance(last_point) < config.duplicate_threshold {
            debug!("Ignoring duplicate point at {target_pos:?}");
            return;
        }
    }

    // Log snapping behavior
    if (target_pos - cursor_pos.0).length() > 0.1 {
        debug!(
            "Snapped cursor from {:?} to existing point {:?}",
            cursor_pos.0, target_pos
        );
    }

    debug!(
        "Tool: {:?}, State: {:?}, Points: {}/4",
        tool_state.current(),
        curve_creation_state.curve_creation_state,
        curve_creation_state.curve_creation_points.len()
    );

    match curve_creation_state.curve_creation_state {
        CurveCreationState::Idle => {
            // Start collecting points - should have 0 points here
            debug!("In Idle state, clearing points and starting new curve");
            curve_creation_state.reset(&mut commands);
            curve_creation_state.curve_creation_points.push(target_pos);
            curve_creation_state.last_point = Some(target_pos);
            curve_creation_state.curve_creation_state = CurveCreationState::CollectingPoints;
            debug!("Started cubic Bézier curve creation. Added point: {target_pos:?} (total: 1/4)");
        }
        CurveCreationState::CollectingPoints => {
            curve_creation_state.curve_creation_points.push(target_pos);
            curve_creation_state.last_point = Some(target_pos);
            let point_count = curve_creation_state.curve_creation_points.len();
            debug!("Added point: {target_pos:?} (total: {point_count}/4)");

            if point_count == 4 {
                // Automatically create the curve
                let curve = BezierCurve::new(curve_creation_state.curve_creation_points.clone());
                commands.spawn(curve);

                // Reset state for next curve
                curve_creation_state.reset(&mut commands);
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
    mut curve_creation_state: ResMut<CurveCreationToolState>,
    config: Res<CurveCreationConfig>,
    existing_points: Query<(Entity, &CurveCreationPoint)>,
) {
    // Clear existing creation points if not in create mode
    if !tool_state.is_currently_using_tool(Tool::CreateCurve) {
        curve_creation_state.reset(&mut commands);
        return;
    }

    // Only render if we have points
    if curve_creation_state.curve_creation_points.is_empty() {
        return;
    }

    // Check if we need to update the rendered points
    let existing_count = existing_points.iter().count();
    if existing_count == curve_creation_state.curve_creation_points.len() {
        return; // No change needed
    }

    debug!(
        "Updating points. Existing: {}, Current: {}",
        existing_count,
        curve_creation_state.curve_creation_points.len()
    );

    // Clear existing creation points and state entities
    for (entity, _) in existing_points.iter() {
        commands.entity(entity).despawn();
    }
    curve_creation_state.point_entities.clear();

    // Collect points to avoid borrow checker issues
    let points_to_render: Vec<Vec2> = curve_creation_state.curve_creation_points.clone();

    // Render new points
    for point_pos in points_to_render {
        let entity = render_simple_circle(
            &mut commands,
            &mut meshes,
            &mut materials,
            point_pos,
            config.point_radius,
            config.point_color,
            config.z_layer,
        );

        // Add the component marker
        commands.entity(entity).insert(CurveCreationPoint);

        // Store entity in our state
        curve_creation_state.point_entities.push(entity);
    }
}

#[derive(Component)]
pub struct CurveCreationPoint;
