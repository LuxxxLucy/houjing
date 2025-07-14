use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};

/// Render a simple circle mesh entity with given position, radius, color and z-layer
pub fn render_simple_circle(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    position: Vec2,
    radius: f32,
    color: Color,
    z_layer: f32,
) -> Entity {
    let circle_mesh = Circle::new(radius);
    let mesh_handle = meshes.add(circle_mesh);
    let material_handle = materials.add(ColorMaterial::from(color));

    commands
        .spawn(MaterialMesh2dBundle {
            mesh: Mesh2dHandle(mesh_handle),
            material: material_handle,
            transform: Transform::from_translation(position.extend(z_layer)),
            ..default()
        })
        .id()
}

/// Render a simple rectangle mesh entity with given center, size, color and z-layer
pub fn render_simple_rectangle(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    center: Vec2,
    size: Vec2,
    color: Color,
    z_layer: f32,
) -> Entity {
    let rectangle_mesh = Rectangle::new(1.0, 1.0);
    let mesh_handle = meshes.add(rectangle_mesh);
    let material_handle = materials.add(ColorMaterial::from(color));

    commands
        .spawn((MaterialMesh2dBundle {
            mesh: Mesh2dHandle(mesh_handle),
            material: material_handle,
            transform: Transform::from_translation(center.extend(z_layer))
                .with_scale(size.extend(1.0)),
            ..default()
        },))
        .id()
}

/// Render a dashed line using gizmos with customizable dash/gap lengths and offset
pub fn render_dashed_line(
    gizmos: &mut Gizmos,
    start: Vec2,
    end: Vec2,
    color: Color,
    dash_length: f32,
    gap_length: f32,
    dash_offset: f32,
) {
    let line_vec = end - start;
    let line_length = line_vec.length();

    // Handle zero-length lines
    if line_length < f32::EPSILON {
        return;
    }

    let line_dir = line_vec.normalize();

    let mut current_pos = -dash_offset;
    while current_pos < line_length {
        let dash_start = current_pos.max(0.0);
        let dash_end = (current_pos + dash_length).min(line_length);

        if dash_start < line_length && dash_end > 0.0 {
            let start_point = start + line_dir * dash_start;
            let end_point = start + line_dir * dash_end;
            gizmos.line_2d(start_point, end_point, color);
        }

        current_pos += dash_length + gap_length;
    }
}

/// Configuration for dashed line rendering
pub struct DashedLineConfig {
    pub dash_length: f32,
    pub gap_length: f32,
    pub animation_speed: f32,
}

/// Render an animated dashed line that moves over time
pub fn render_animated_dashed_line(
    gizmos: &mut Gizmos,
    start: Vec2,
    end: Vec2,
    color: Color,
    config: &DashedLineConfig,
    time: &Time,
) {
    let elapsed = time.elapsed_seconds();
    let dash_offset = (elapsed * config.animation_speed) % (config.dash_length + config.gap_length);

    render_dashed_line(
        gizmos,
        start,
        end,
        color,
        config.dash_length,
        config.gap_length,
        dash_offset,
    );
}
