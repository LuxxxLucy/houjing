use super::cursor::{CursorState, CursorWorldPos};
use super::select::SelectedControlPoint;
use crate::component::curve::{BezierCurve, CurveNeedsUpdate};
use crate::systems::tools::cursor::CursorVisualizationConfig;
use crate::{InputSet, ShowSet};
use bevy::prelude::*;
use bevy::sprite::{ColorMaterial, ColorMesh2dBundle};
use std::collections::HashMap;

#[derive(Resource, Default)]
pub struct DragState {
    /// Drag state when there is no selected point
    pub rectangle: NoSelectedPointDragState,
    /// Drag state when there is a selected point
    pub selected_points: SelectedPointDragState,
}

#[derive(Default)]
pub struct SelectedPointDragState {
    /// Original curve states before dragging
    pub original_curves: HashMap<Entity, Vec<Vec2>>,
    /// Current positions of selected points during drag
    pub current_positions: HashMap<(Entity, usize), Vec2>,
    /// Whether point dragging is active
    pub is_active: bool,
}

#[derive(Default)]
pub struct NoSelectedPointDragState {
    /// Entity of the rectangle mesh
    pub entity: Option<Entity>,
    /// Current rectangle
    pub rect: Option<DragRect>,
}

/// Rectangle for drag selection
#[derive(Clone, Copy)]
pub struct DragRect {
    pub origin: Vec2,
    pub width: f32,
    pub height: f32,
}

/// Component marker for no selected point drag rectangle entity
#[derive(Component)]
pub struct NoSelectedPointDragRectangle;

pub struct DragPlugin;

impl Plugin for DragPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DragState>()
            .add_systems(
                Update,
                (
                    handle_selected_point_drag_state,
                    handle_no_selected_point_drag_state,
                )
                    .in_set(InputSet),
            )
            .add_systems(
                Update,
                (render_selected_point_drag, render_no_selected_point_drag).in_set(ShowSet),
            );
    }
}

fn handle_selected_point_drag_state(
    cursor_state: Res<CursorState>,
    cursor_pos: Res<CursorWorldPos>,
    mut commands: Commands,
    selected_query: Query<&SelectedControlPoint>,
    mut curve_query: Query<&mut BezierCurve>,
    mut drag_state: ResMut<DragState>,
) {
    let has_selected_points = !selected_query.is_empty();
    let is_dragging = cursor_state.dragging && has_selected_points;

    if is_dragging {
        // Initialize original curves if not already done
        if drag_state.selected_points.original_curves.is_empty() {
            debug!(
                "Capturing original curve states for {} selected points",
                selected_query.iter().count()
            );
            for selected_point in selected_query.iter() {
                if let Ok(curve) = curve_query.get(selected_point.curve_entity) {
                    debug!(
                        "Storing original state for curve entity {:?} with {} control points",
                        selected_point.curve_entity,
                        curve.control_points.len()
                    );
                    drag_state
                        .selected_points
                        .original_curves
                        .insert(selected_point.curve_entity, curve.control_points.clone());
                }
            }
            debug!(
                "Total curves stored: {}",
                drag_state.selected_points.original_curves.len()
            );
        }

        // Update current positions for selected points
        for selected_point in selected_query.iter() {
            let key = (selected_point.curve_entity, selected_point.point_index);
            drag_state
                .selected_points
                .current_positions
                .insert(key, cursor_pos.0);
        }

        drag_state.selected_points.is_active = true;

        // Update curve points using current positions from drag state
        for selected_point in selected_query.iter() {
            if let Ok(mut curve) = curve_query.get_mut(selected_point.curve_entity) {
                if let Some(point) = curve.control_points.get_mut(selected_point.point_index) {
                    let key = (selected_point.curve_entity, selected_point.point_index);
                    if let Some(&current_pos) =
                        drag_state.selected_points.current_positions.get(&key)
                    {
                        *point = current_pos;

                        // Mark curve for mesh update
                        commands
                            .entity(selected_point.curve_entity)
                            .insert(CurveNeedsUpdate);
                    }
                }
            }
        }
    } else {
        // Clear state when not dragging
        drag_state.selected_points.original_curves.clear();
        drag_state.selected_points.current_positions.clear();
        drag_state.selected_points.is_active = false;
    }
}

fn render_selected_point_drag(
    mut gizmos: Gizmos,
    cursor_state: Res<CursorState>,
    config: Res<CursorVisualizationConfig>,
    selected_query: Query<&SelectedControlPoint>,
    drag_state: Res<DragState>,
) {
    // Only show when dragging is active
    if !drag_state.selected_points.is_active {
        return;
    }

    fn render_diamond_cursor(
        gizmos: &mut Gizmos,
        cursor_pos: Vec2,
        config: &CursorVisualizationConfig,
    ) {
        let color = config.drag_color;
        let half_size = config.cursor_size / 2.0;

        // Draw diamond shape for drag cursor
        let corners = [
            cursor_pos + Vec2::new(0.0, half_size),  // top
            cursor_pos + Vec2::new(half_size, 0.0),  // right
            cursor_pos + Vec2::new(0.0, -half_size), // bottom
            cursor_pos + Vec2::new(-half_size, 0.0), // left
        ];

        for i in 0..4 {
            gizmos.line_2d(corners[i], corners[(i + 1) % 4], color);
        }
    }

    fn render_drag_start_indicator(
        gizmos: &mut Gizmos,
        start_pos: Vec2,
        config: &CursorVisualizationConfig,
    ) {
        gizmos.circle_2d(start_pos, 4.0, config.drag_color.with_a(0.5));
    }

    fn render_original_curve(
        gizmos: &mut Gizmos,
        control_points: &[Vec2],
        config: &CursorVisualizationConfig,
    ) {
        if control_points.len() < 2 {
            return;
        }

        // Create a temporary curve for evaluation
        let temp_curve = BezierCurve {
            control_points: control_points.to_vec(),
        };

        // Render original curve as simple low-opacity line
        let samples = 100;

        for i in 0..samples {
            let t1 = i as f32 / samples as f32;
            let t2 = (i + 1) as f32 / samples as f32;
            let p1 = temp_curve.evaluate(t1);
            let p2 = temp_curve.evaluate(t2);

            // Draw with low opacity to distinguish from current curve
            gizmos.line_2d(p1, p2, config.drag_color.with_a(0.3));
        }
    }

    // Render original curves as low-opacity lines
    if !drag_state.selected_points.original_curves.is_empty() {
        for original_curve_points in drag_state.selected_points.original_curves.values() {
            render_original_curve(&mut gizmos, original_curve_points, &config);
        }
    }

    // Draw diamond cursors at current positions of selected points
    for selected_point in selected_query.iter() {
        let key = (selected_point.curve_entity, selected_point.point_index);
        if let Some(&current_pos) = drag_state.selected_points.current_positions.get(&key) {
            render_diamond_cursor(&mut gizmos, current_pos, &config);
        }
    }

    // Draw drag start position indicator
    render_drag_start_indicator(&mut gizmos, cursor_state.drag_start_pos, &config);
}

fn handle_no_selected_point_drag_state(
    cursor_state: Res<CursorState>,
    cursor_pos: Res<CursorWorldPos>,
    selected_query: Query<&SelectedControlPoint>,
    mut drag_state: ResMut<DragState>,
) {
    let should_have_no_selected_point_drag = cursor_state.dragging && selected_query.is_empty();

    if should_have_no_selected_point_drag {
        // Calculate no selected point drag rectangle
        let start = cursor_state.drag_start_pos;
        let end = cursor_pos.0;
        let min_x = start.x.min(end.x);
        let max_x = start.x.max(end.x);
        let min_y = start.y.min(end.y);
        let max_y = start.y.max(end.y);

        drag_state.rectangle.rect = Some(DragRect {
            origin: Vec2::new(min_x, min_y),
            width: max_x - min_x,
            height: max_y - min_y,
        });
    } else {
        drag_state.rectangle.rect = None;
    }
}

#[allow(clippy::too_many_arguments)]
fn render_no_selected_point_drag(
    mut commands: Commands,
    mut gizmos: Gizmos,
    config: Res<CursorVisualizationConfig>,
    time: Res<Time>,
    mut drag_state: ResMut<DragState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut no_selected_point_drag_query: Query<&mut Transform, With<NoSelectedPointDragRectangle>>,
) {
    if let Some(no_selected_point_drag_rect) = drag_state.rectangle.rect {
        // Render no selected point drag filled background
        render_no_selected_point_drag_fill(
            &mut commands,
            no_selected_point_drag_rect,
            &config,
            &mut drag_state.rectangle,
            &mut meshes,
            &mut materials,
            &mut no_selected_point_drag_query,
        );

        // Render no selected point drag animated wireframe
        render_no_selected_point_drag_wireframe(
            &mut gizmos,
            no_selected_point_drag_rect,
            &config,
            &time,
        );
    } else if let Some(entity) = drag_state.rectangle.entity {
        // Remove no selected point drag rectangle when not dragging
        commands.entity(entity).despawn();
        drag_state.rectangle.entity = None;
    }
}

fn render_no_selected_point_drag_fill(
    commands: &mut Commands,
    no_selected_point_drag_rect: DragRect,
    config: &CursorVisualizationConfig,
    no_selected_point_drag_state: &mut NoSelectedPointDragState,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    no_selected_point_drag_query: &mut Query<&mut Transform, With<NoSelectedPointDragRectangle>>,
) {
    if let Some(entity) = no_selected_point_drag_state.entity {
        // Update existing no selected point drag rectangle
        if let Ok(mut transform) = no_selected_point_drag_query.get_mut(entity) {
            let center = Vec2::new(
                no_selected_point_drag_rect.origin.x + no_selected_point_drag_rect.width / 2.0,
                no_selected_point_drag_rect.origin.y + no_selected_point_drag_rect.height / 2.0,
            );
            transform.translation = center.extend(0.0);
            transform.scale = Vec2::new(
                no_selected_point_drag_rect.width,
                no_selected_point_drag_rect.height,
            )
            .extend(1.0);
        }
    } else {
        // Create new no selected point drag rectangle
        let background_color = config.drag_color.with_a(0.1);
        let material = materials.add(ColorMaterial::from(background_color));
        let mesh = meshes.add(Rectangle::new(1.0, 1.0));

        let center = Vec2::new(
            no_selected_point_drag_rect.origin.x + no_selected_point_drag_rect.width / 2.0,
            no_selected_point_drag_rect.origin.y + no_selected_point_drag_rect.height / 2.0,
        );
        let entity = commands
            .spawn((
                ColorMesh2dBundle {
                    mesh: mesh.into(),
                    material,
                    transform: Transform::from_translation(center.extend(0.0)).with_scale(
                        Vec2::new(
                            no_selected_point_drag_rect.width,
                            no_selected_point_drag_rect.height,
                        )
                        .extend(1.0),
                    ),
                    ..default()
                },
                NoSelectedPointDragRectangle,
            ))
            .id();

        no_selected_point_drag_state.entity = Some(entity);
    }
}

fn render_no_selected_point_drag_wireframe(
    gizmos: &mut Gizmos,
    no_selected_point_drag_rect: DragRect,
    config: &CursorVisualizationConfig,
    time: &Time,
) {
    // Calculate no selected point drag rectangle corners from rect
    let min_x = no_selected_point_drag_rect.origin.x;
    let max_x = no_selected_point_drag_rect.origin.x + no_selected_point_drag_rect.width;
    let min_y = no_selected_point_drag_rect.origin.y;
    let max_y = no_selected_point_drag_rect.origin.y + no_selected_point_drag_rect.height;

    let top_left = Vec2::new(min_x, max_y);
    let top_right = Vec2::new(max_x, max_y);
    let bottom_right = Vec2::new(max_x, min_y);
    let bottom_left = Vec2::new(min_x, min_y);

    // Animation parameters for no selected point drag rectangle border
    let dash_length = 6.0;
    let gap_length = 4.0;
    let pattern_length = dash_length + gap_length;
    let animation_speed = 40.0; // pixels per second
    let time_offset = (time.elapsed_seconds() * animation_speed) % pattern_length;
    let selection_color = config.drag_color.with_a(0.8);

    // Function to draw dashed line between two points
    let draw_dashed_line = |gizmos: &mut Gizmos, start: Vec2, end: Vec2, offset: f32| {
        let direction = end - start;
        let distance = direction.length();

        if distance > 0.0 {
            let normalized_direction = direction / distance;
            let mut current_distance = -offset;

            while current_distance < distance {
                let dash_start = current_distance.max(0.0);
                let dash_end = (current_distance + dash_length).min(distance);

                if dash_start < dash_end {
                    let start_pos = start + normalized_direction * dash_start;
                    let end_pos = start + normalized_direction * dash_end;
                    gizmos.line_2d(start_pos, end_pos, selection_color);
                }

                current_distance += pattern_length;
            }
        }
    };

    // Draw animated dashed no selected point drag rectangle border
    draw_dashed_line(gizmos, top_left, top_right, time_offset);
    draw_dashed_line(gizmos, top_right, bottom_right, time_offset);
    draw_dashed_line(gizmos, bottom_right, bottom_left, time_offset);
    draw_dashed_line(gizmos, bottom_left, top_left, time_offset);
}
