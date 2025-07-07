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
