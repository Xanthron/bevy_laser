#![feature(adt_const_params)]
pub mod click;
pub mod collider2d;
pub mod element;
pub mod game_state;
pub mod laser;
pub mod player;
pub mod timer;
pub mod window;

use bevy::prelude::*;

#[derive(Component)]
pub struct MainCamera;

fn main() {
    App::new()
        .add_plugin(window::WindowPlugin)
        .add_plugins(DefaultPlugins)
        .add_startup_system(render_system.system())
        .add_plugin(click::ClickablePlugin)
        .add_plugin(player::PlayerPlugin)
        .add_plugin(element::ElementPlugin)
        .add_plugin(laser::LaserPlugin)
        .add_plugin(game_state::GameStatePlugin)
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
        .run();
}

fn render_system(mut commands: Commands) {
    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(MainCamera);
    commands.spawn_bundle(UiCameraBundle::default());
}
