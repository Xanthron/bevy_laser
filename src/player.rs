use std::f32::consts::PI;

use bevy::{ecs::schedule::ShouldRun, math::Vec3Swizzles, prelude::*};

use crate::{
    click::{self, Clicked},
    game_state::*,
    window::{get_3d_from_cord, COLUMNS, HEIGHT, ROWS, SIZE, SIZE_MULTIPLIER, WIDTH},
};

pub const MAX_ANGLE: f32 = PI / 2.5;
//pub const MAX_STEPS: f32 = (6.0) / 2.0;

#[derive(Component)]
pub struct Player;
#[derive(Component)]
pub struct PlayerMoveAnimation {
    destination: Vec3,
    at_destination: bool,
}

#[derive(Component)]
pub struct PossiblePositions;
#[derive(Component)]
pub struct PossiblePositionsClicked;
#[derive(Component)]
pub struct PossiblePositionsAnimation {
    _data: bool,
}

#[derive(Component)]
pub struct Cannon;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(startup_system)
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(IS_AIMING_LASER_STATE)
                    .with_system(cannon_mouse_rotation_system)
                    //.with_system(select_possible_position_system)
                    .with_system(animate_selected_possible_position_system)
                    .with_system(set_player_move_position_system)
                    .with_system(player_start_shoot_system),
            )
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(IS_MOVE_PLAYER_STATE)
                    .with_system(player_movement_system),
            );
    }
}

pub fn startup_system(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    println!("Spawn Player!");

    //Player Position Pattern
    for i in 1..(COLUMNS as u16 - 1) {
        let i = i as f32;
        commands
            .spawn_bundle(SpriteBundle {
                material: materials.add(Color::rgb(0.2, 0.2, 0.2).into()),
                transform: Transform {
                    translation: get_3d_from_cord(i, ROWS - 1.0, 0.0).into(),
                    ..Default::default()
                },
                sprite: Sprite::new((SIZE * 0.9, SIZE * 0.9).into()),
                ..Default::default()
            })
            .insert(PossiblePositions)
            .insert(click::Clickable {
                size: Vec2::new(SIZE, SIZE),
                active: true,
            });
    }

    //Player
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(Color::rgb(1.0, 0.0, 0.0).into()),
            transform: Transform {
                translation: get_3d_from_cord(COLUMNS / 2.0 - 0.5, ROWS - 2.0, 2.0).into(),
                ..Default::default()
            },
            sprite: Sprite::new((SIZE, SIZE).into()),
            ..Default::default()
        })
        .insert(Player)
        .insert(PlayerMoveAnimation {
            destination: Vec3::ZERO,
            at_destination: true,
        })
        .with_children(|parent| {
            parent
                .spawn()
                .insert(Transform {
                    rotation: Quat::from_rotation_z(PI / 4.0),
                    ..Default::default()
                })
                .insert(GlobalTransform::default())
                .insert(Cannon)
                .with_children(|parent| {
                    parent.spawn_bundle(SpriteBundle {
                        material: materials.add(Color::rgb(0.0, 0.0, 1.0).into()),
                        transform: Transform::from_xyz(0.0, SIZE, -1.0),
                        sprite: Sprite::new((SIZE / 2.0, SIZE).into()),
                        ..Default::default()
                    });
                });
        });
}

pub fn cannon_mouse_rotation_system(
    mut query: Query<(&Cannon, &GlobalTransform, &mut Transform)>,
    mut cursor_moved_events: EventReader<CursorMoved>,
) {
    if let Some(cursor_move) = cursor_moved_events.iter().next() {
        let (_, global_transform, mut transform) = query.single_mut();

        let cursor_pos: Vec2 = cursor_move.position;
        let cannon_pos: Vec2 =
            global_transform.translation.xy() + Vec2::new(WIDTH / 2.0, HEIGHT / 2.0);
        let vec = cursor_pos - cannon_pos;
        let mut angle = Vec2::new(0.0, 1.0).angle_between(vec);

        if !angle.is_nan() {
            //angle = ((angle / MAX_ANGLE * MAX_STEPS).floor() - 0.5) * (MAX_ANGLE / MAX_STEPS);
            angle = angle.clamp(-MAX_ANGLE, MAX_ANGLE);

            transform.rotation = Quat::from_rotation_z(angle);
        }
    }
    //FIXME Does not rotate, when mouse is under cannon. Solution: Abs value
}

pub fn animate_selected_possible_position_system(
    mut materials: ResMut<Assets<ColorMaterial>>,
    query: Query<
        (
            Entity,
            //&PossiblePositionsAnimation,
            Option<&click::Selected>,
            Option<&click::Hovered>,
            &Handle<ColorMaterial>,
        ),
        With<PossiblePositions>,
    >,
) {
    for (_entity, selected, hovered, material) in query.iter() {
        if selected.is_some() {
            if hovered.is_some() {
                let _ = materials.set(material, Color::rgb(0.4, 0.2, 0.2).into());
            } else {
                let _ = materials.set(material, Color::rgb(0.2, 0.4, 0.2).into());
            }
        } else {
            if hovered.is_some() {
                let _ = materials.set(material, Color::rgb(0.2, 0.2, 0.4).into());
            } else {
                let _ = materials.set(material, Color::rgb(0.2, 0.2, 0.2).into());
            }
        }
        //TODO Add animation
    }
}

pub fn player_movement_system(
    mut game_state: ResMut<GameStateRes>,
    mut query: Query<(&mut Transform, &mut PlayerMoveAnimation)>,
    delta_time: Res<Time>,
) {
    let max_length = SIZE_MULTIPLIER / 2.0 * delta_time.delta_seconds() * 50.0;
    let max_length_squared = max_length * max_length;

    let (mut transform, mut player_move_animation) = query.single_mut();
    if !player_move_animation.at_destination {
        let vec = player_move_animation.destination - transform.translation;
        if vec.length_squared() <= max_length_squared {
            transform.translation = player_move_animation.destination;
            player_move_animation.at_destination = true;
        } else {
            transform.translation += vec.normalize() * max_length;
        }
    } else {
        game_state.change(GameState::AimingLaser);
    }
}

pub fn set_player_move_position_system(
    mut game_state: ResMut<GameStateRes>,
    mut commands: Commands,
    query_selectable: Query<(Entity, &Transform), (With<PossiblePositions>, With<Clicked>)>,
    mut query_player: Query<&mut PlayerMoveAnimation, With<Player>>,
) {
    if let Ok((entity, transform)) = query_selectable.get_single() {
        let mut player_move_animation = query_player.single_mut();
        player_move_animation.at_destination = false;
        player_move_animation.destination =
            (transform.translation.x, (-ROWS / 2.0 + 1.5) * SIZE, 2.0).into();

        commands.entity(entity).remove::<Clicked>();

        game_state.change(GameState::MovePlayer);
    }
}

pub fn player_start_shoot_system(
    mut game_state: ResMut<GameStateRes>,
    mouse_buttons: Res<Input<MouseButton>>,
    windows: Res<Windows>,
) {
    if let Some(vec2) = windows
        .get_primary()
        .expect("no primary window")
        .cursor_position()
    {
        if mouse_buttons.just_released(MouseButton::Left)
            && vec2.x >= 0.0
            && vec2.x <= WIDTH
            && vec2.y >= SIZE * 3.0
            && vec2.y <= HEIGHT
        {
            game_state.change(GameState::FireLaser);
        }
    }
}
