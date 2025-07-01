use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct MouseWorldPos(pub Vec2);

#[derive(Resource, Default)]
pub struct InputState {
    pub mouse_pressed: bool,
    pub mouse_just_pressed: bool,
    pub dragging: bool,
    pub drag_start_pos: Vec2,
}

pub fn update_mouse_world_position(
    mut mouse_world_pos: ResMut<MouseWorldPos>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
) {
    let window = windows.single();
    let (camera, camera_transform) = camera_q.single();

    if let Some(cursor_pos) = window.cursor_position() {
        if let Some(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
            mouse_world_pos.0 = world_pos;
        }
    }
}

pub fn handle_mouse_input(
    mut input_state: ResMut<InputState>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mouse_pos: Res<MouseWorldPos>,
) {
    let just_pressed = mouse_input.just_pressed(MouseButton::Left);
    let pressed = mouse_input.pressed(MouseButton::Left);
    let just_released = mouse_input.just_released(MouseButton::Left);

    if just_pressed {
        input_state.drag_start_pos = mouse_pos.0;
        input_state.dragging = false;
    }

    if pressed && !input_state.dragging {
        let drag_threshold = 5.0;
        if mouse_pos.0.distance(input_state.drag_start_pos) > drag_threshold {
            input_state.dragging = true;
        }
    }

    if just_released {
        input_state.dragging = false;
    }

    input_state.mouse_just_pressed = just_pressed;
    input_state.mouse_pressed = pressed;
}
