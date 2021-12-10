use bevy::{prelude::*, window::WindowResizeConstraints};

pub const ROWS: f32 = 16.0;
pub const COLUMNS: f32 = 9.0;
pub const SIZE_MULTIPLIER: f32 = 20.0;
pub const SIZE: f32 = SIZE_MULTIPLIER * 2.0;

pub const WIDTH: f32 = COLUMNS * SIZE_MULTIPLIER * 2.0;
pub const HEIGHT: f32 = ROWS * SIZE_MULTIPLIER * 2.0;

pub struct WindowPlugin;
impl Plugin for WindowPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WindowDescriptor {
            width: WIDTH,
            height: HEIGHT,
            resize_constraints: WindowResizeConstraints {
                min_width: WIDTH,
                min_height: HEIGHT,
                max_width: WIDTH,
                max_height: HEIGHT,
            },
            scale_factor_override: None,
            title: "Bevy Laser".to_string(),
            vsync: true,
            resizable: true,
            decorations: true,
            cursor_visible: true,
            cursor_locked: false,
            mode: bevy::window::WindowMode::Windowed,
            ..Default::default()
        });
    }
}

pub fn get_3d_from_cord(x: f32, y: f32, z: f32) -> (f32, f32, f32) {
    (
        -WIDTH / 2.0 + SIZE * (x + 0.5),
        HEIGHT / 2.0 - SIZE * (y + 0.5),
        z,
    )
}
