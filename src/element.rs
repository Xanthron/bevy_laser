use bevy::{ecs::schedule::ShouldRun, prelude::*, render2::render_phase::TrackedRenderPass};
use rand::prelude::*;

use crate::{
    collider2d::{self, Collider},
    game_state::*,
    window::{get_3d_from_cord, COLUMNS, HEIGHT, ROWS, SIZE, SIZE_MULTIPLIER, WIDTH},
};

//TODO Import Const

pub struct ElementPlugin;
impl Plugin for ElementPlugin {
    fn build(&self, app: &mut App) {
        //Spawn
        app.add_startup_system(startup_spawn_world_collider)
            .add_system_set(
                SystemSet::new()
                    .label(GenerateElementLabel::Main)
                    .with_run_criteria(IS_GENERATE_OBSTACLE_STATE)
                    .with_system(
                        generate_next_elements_system
                            .system()
                            .label(GenerateElementLabel::Generate),
                    ),
            )
            .add_system_set(
                SystemSet::new()
                    .label(SpawnElementLabel::Main)
                    .with_run_criteria(IS_SPAWN_OBSTACLE_STATE)
                    .with_system(spawn_block_system.system().label(SpawnElementLabel::Spawn))
                    .with_system(
                        spawn_powerup_laser_system
                            .system()
                            .label(SpawnElementLabel::Spawn),
                    )
                    .with_system(
                        init_move_system
                            .system()
                            .label(SpawnElementLabel::InitMove)
                            .after(SpawnElementLabel::Spawn),
                    ),
            )
            .add_system_set(
                SystemSet::new()
                    .label(MoveElementLabel::Main)
                    .with_run_criteria(IS_MOVE_OBSTACLE_STATE)
                    .with_system(move_system.system().label(MoveElementLabel::Move)),
            );
    }
}

pub struct Element;

#[derive(Component)]
struct AnimationMoveDown {
    destination: Vec3,
}
// pub struct AnimationLoseLive;
// pub struct AnimationSpawn;
// pub struct AnimationDestroy;

#[derive(Component)]
pub struct Block;
// #[derive(Component)]
// pub struct Triangle;
//pub struct Live;
#[derive(Component)]
pub struct Live(pub i32);

#[derive(Component)]
pub struct PowerupAddLaser;

#[derive(Component)]
pub struct Bounce;

fn startup_spawn_world_collider(mut commands: Commands) {
    commands
        .spawn()
        .insert(Collider::new(vec![
            Vec2::new(WIDTH / 2.0, -HEIGHT / 2.0),
            Vec2::new(WIDTH / 2.0, HEIGHT / 2.0),
            Vec2::new(-WIDTH / 2.0, HEIGHT / 2.0),
            Vec2::new(-WIDTH / 2.0, -HEIGHT / 2.0),
        ]))
        .insert(Transform::default());
}

//region [rgba(256,256,0,0.2)] Generate
#[derive(Clone, Hash, Debug, PartialEq, Eq, SystemLabel)]
pub enum GenerateElementLabel {
    Main,
    Generate,
}

pub fn generate_next_elements_system(mut commands: Commands, mut game_state: ResMut<GameStateRes>) {
    //TODO Move to spawn options
    let block_probability = 0.3;
    let triangle_probability = 0.1;
    let bounce_probability = 0.1;

    let powerup_laser_position = rand::thread_rng().gen_range(1u8..(COLUMNS as u8));
    info!("Powerup Laser pos {}", powerup_laser_position);
    for element in 0..(COLUMNS as u8) {
        let mut o_entity = None;
        if element == powerup_laser_position {
            let mut entity = commands.spawn();
            entity.insert(PowerupAddLaser);
            o_entity = Some(entity);
        } else {
            let rng = rand::thread_rng().gen();
            if block_probability <= rng {
                let mut entity = commands.spawn();
                entity.insert(Block);
                o_entity = Some(entity);
            // } else if block_probability + triangle_probability <= rng {
            //     commands.spawn().insert(Triangle);
            } else if block_probability + triangle_probability + bounce_probability <= rng {
                let mut entity = commands.spawn();
                entity.insert(Bounce);
                o_entity = Some(entity);
            }
        }
        if let Some(mut entity) = o_entity {
            let pos = Vec3::from(get_3d_from_cord(element as f32, 0.0, 0.0));
            entity.insert(AnimationMoveDown { destination: pos });
        }
    }

    game_state.change(GameState::SpawnObstacle);
}
//endregion

//region [rgba(256,150,0,0.15)] Spawn
#[derive(Clone, Hash, Debug, PartialEq, Eq, SystemLabel)]
pub enum SpawnElementLabel {
    Main,
    Spawn,
    InitMove,
}

fn spawn_block_system(
    mut commands: Commands,
    query: Query<(Entity, &AnimationMoveDown), (With<Block>, Without<Transform>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, animation_move_down) in query.iter() {
        commands
            .entity(entity)
            .insert_bundle(SpriteBundle {
                sprite: Sprite::new((SIZE * 0.95, SIZE * 0.95).into()),
                material: materials.add(Color::rgb(0.0, 0.0, 1.0).into()),
                transform: Transform::from_translation(animation_move_down.destination),
                ..Default::default()
            })
            .insert(collider2d::Block::new_collider(Vec2::new(SIZE, SIZE)))
            .with_children(|parent| {
                parent.spawn_bundle(SpriteBundle {
                    sprite: Sprite::new((SIZE * 0.85, SIZE * 0.85).into()),
                    material: materials.add(Color::rgb(0.0, 0.0, 0.0).into()),
                    transform: Transform::from_translation((0.0, 0.0, 1.0).into()),
                    ..Default::default()
                });
            });
    }
}
fn spawn_powerup_laser_system(
    mut commands: Commands,
    query: Query<(Entity, &AnimationMoveDown), (With<PowerupAddLaser>, Without<Transform>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, animation_move_down) in query.iter() {
        commands.entity(entity).insert_bundle(SpriteBundle {
            sprite: Sprite::new((SIZE / 2.0, SIZE / 2.0).into()),
            material: materials.add(Color::rgb(0.0, 1.0, 0.0).into()),
            transform: Transform::from_translation(animation_move_down.destination),
            ..Default::default()
        });
    }
}

fn init_move_system(
    mut game_state: ResMut<GameStateRes>,
    mut query: Query<&mut AnimationMoveDown>,
) {
    for mut animation_move_down in query.iter_mut() {
        animation_move_down.destination =
            animation_move_down.destination + Vec3::new(0.0, -SIZE, 0.0);
    }
    game_state.change(GameState::MoveObstacle);
}
//endregion

//region [rgba(0,256,0,0.1)] Movement
#[derive(Clone, Hash, Debug, PartialEq, Eq, SystemLabel)]
pub enum MoveElementLabel {
    Main,
    Move,
}

fn move_system(
    mut game_state: ResMut<GameStateRes>,
    mut query: Query<(&mut Transform, &AnimationMoveDown)>,
    time: Res<Time>,
) {
    let mut in_movement = false;
    let max_length = SIZE_MULTIPLIER / 2.0 * time.delta_seconds() * 15.0;
    let max_length_squared = max_length * max_length;

    for (mut transform, animation_move_down) in query.iter_mut() {
        let vec = animation_move_down.destination - transform.translation;
        if vec.length_squared() <= max_length_squared {
            transform.translation = animation_move_down.destination;
        } else {
            in_movement = true;
            transform.translation += vec.normalize() * max_length;
        }
    }

    if !in_movement {
        game_state.change(GameState::AimingLaser);
    }
}
//endregion
