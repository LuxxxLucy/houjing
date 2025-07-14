use super::common::point_finding::find_or_create_point_for_snapping;
use super::cursor::*;
use super::tool::{Tool, ToolState};
use crate::component::curve::{BezierCurve, Point, get_position};
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
    pub curve_creation_point_entities: Vec<Entity>,
    pub curve_creation_state: CurveCreationState,
    pub last_point_entity: Option<Entity>,
}

impl CurveCreationToolState {
    pub fn reset(&mut self, _commands: &mut Commands) {
        self.curve_creation_state = CurveCreationState::Idle;
        self.curve_creation_point_entities.clear();
        self.last_point_entity = None;
    }
}

// Default curve creation configuration constants
const DEFAULT_POINT_COLOR: Color = Color::BLUE;
const DEFAULT_POINT_RADIUS: f32 = 6.0;
const DEFAULT_SNAP_THRESHOLD: f32 = 15.0; // Distance threshold for snapping to existing points
const DEFAULT_CURVE_CREATION_Z_LAYER: f32 = 2.0;

#[derive(Resource)]
struct CurveCreationConfig {
    pub point_color: Color,
    pub point_radius: f32,
    pub snap_threshold: f32,
    pub z_layer: f32,
}

impl Default for CurveCreationConfig {
    fn default() -> Self {
        Self {
            point_color: DEFAULT_POINT_COLOR,
            point_radius: DEFAULT_POINT_RADIUS,
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
    config: Res<CurveCreationConfig>,
    point_query: Query<(Entity, &Point)>,
) {
    // Check if tool is active, reset state if not
    if !tool_state.is_currently_using_tool(Tool::CreateCurve) {
        curve_creation_state.reset(&mut commands);
        return;
    }

    if !cursor_state.mouse_just_pressed {
        return;
    }

    // Find or create point entity for the cursor position, with snapping
    let point_entity = find_or_create_point_for_snapping(
        cursor_state.cursor_position,
        &mut commands,
        &point_query,
        config.snap_threshold,
    );

    // Get the position of the point
    let target_pos =
        get_position(point_entity, &point_query).unwrap_or(cursor_state.cursor_position);

    // Check if this is the same point as the last one
    if let Some(last_point_entity) = curve_creation_state.last_point_entity {
        if point_entity == last_point_entity {
            debug!("Ignoring duplicate point entity {point_entity:?}");
            return;
        }
    }

    // Log snapping behavior
    if (target_pos - cursor_state.cursor_position).length() > 0.1 {
        debug!(
            "Snapped cursor from {:?} to existing point {:?}",
            cursor_state.cursor_position, target_pos
        );
    }

    debug!(
        "Tool: {:?}, State: {:?}, Points: {}/4",
        tool_state.current(),
        curve_creation_state.curve_creation_state,
        curve_creation_state.curve_creation_point_entities.len()
    );

    match curve_creation_state.curve_creation_state {
        CurveCreationState::Idle => {
            // Start collecting points - should have 0 points here
            debug!("In Idle state, clearing points and starting new curve");
            curve_creation_state.reset(&mut commands);
            curve_creation_state
                .curve_creation_point_entities
                .push(point_entity);
            curve_creation_state.last_point_entity = Some(point_entity);
            curve_creation_state.curve_creation_state = CurveCreationState::CollectingPoints;
            debug!(
                "Started cubic Bézier curve creation. Added point entity: {point_entity:?} at {target_pos:?} (total: 1/4)"
            );
        }
        CurveCreationState::CollectingPoints => {
            curve_creation_state
                .curve_creation_point_entities
                .push(point_entity);
            curve_creation_state.last_point_entity = Some(point_entity);
            let point_count = curve_creation_state.curve_creation_point_entities.len();
            debug!(
                "Added point entity: {point_entity:?} at {target_pos:?} (total: {point_count}/4)"
            );

            if point_count == 4 {
                // Automatically create the curve
                let curve =
                    BezierCurve::new(curve_creation_state.curve_creation_point_entities.clone());
                commands.spawn(curve);

                // Reset state for next curve
                curve_creation_state.reset(&mut commands);
                debug!("Created cubic Bézier curve! State reset to Idle. Ready for next curve.")
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn render_curve_creation_points(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    tool_state: Res<ToolState>,
    mut curve_creation_state: ResMut<CurveCreationToolState>,
    config: Res<CurveCreationConfig>,
    existing_previews: Query<(Entity, &CurveCreationPoint)>,
    point_query: Query<&Point>,
) {
    // Clear existing creation points if not in create mode
    if !tool_state.is_currently_using_tool(Tool::CreateCurve) {
        curve_creation_state.reset(&mut commands);
        return;
    }

    // Only render if we have points
    if curve_creation_state
        .curve_creation_point_entities
        .is_empty()
    {
        return;
    }

    // Check if we need to update the rendered points
    let existing_count = existing_previews.iter().count();
    if existing_count == curve_creation_state.curve_creation_point_entities.len() {
        return; // No change needed
    }

    debug!(
        "Updating points. Existing: {}, Current: {}",
        existing_count,
        curve_creation_state.curve_creation_point_entities.len()
    );

    // Clear existing creation preview entities
    for (entity, _) in existing_previews.iter() {
        commands.entity(entity).despawn();
    }

    // Collect point entities to avoid borrow checker issues
    let point_entities_to_render = curve_creation_state.curve_creation_point_entities.clone();

    // Render new preview points
    for point_entity in point_entities_to_render {
        if let Ok(point_pos) = point_query.get(point_entity) {
            let preview_entity = render_simple_circle(
                &mut commands,
                &mut meshes,
                &mut materials,
                point_pos.position(),
                config.point_radius,
                config.point_color,
                config.z_layer,
            );

            // Add the component marker
            commands.entity(preview_entity).insert(CurveCreationPoint);
        }
    }
}

#[derive(Component)]
pub struct CurveCreationPoint;
