use crate::{
    collider2d::Collider,
    element::Live,
    game_state::*,
    player::Cannon,
    window::{HEIGHT, WIDTH},
};
use bevy::{
    core::FixedTimestep,
    ecs::schedule::ShouldRun,
    math::{Mat2, Vec3Swizzles},
    prelude::*,
};

pub const AIMING_LASERS: usize = 300;

pub const MAX_LASER_LENGTH: f32 = WIDTH * WIDTH + HEIGHT * HEIGHT; //FIXME take the square root but that is not supported for const.
pub const HIT_OFFSET: f32 = 0.1;
pub const LASER_WIDTH: f32 = 4.0;

pub struct LaserPlugin;

impl Plugin for LaserPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(FireLaserRes {
            amount: 100,
            shot: 0,
        })
        .add_startup_system(aiming_startup)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(IS_AIMING_LASER_STATE)
                .label(LaserLabel::Main)
                .with_system(aiming_system.system().label(LaserLabel::CalculateLaser))
                .with_system(
                    drawing_system
                        .system()
                        .label(LaserLabel::DrawLaser)
                        .after(LaserLabel::CalculateLaser),
                ),
        )
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(1f64 / 24f64).chain(run_if_fire_laser_chain))
                .label(FireLaserLabel::Main)
                .with_system(hide_aiming_laser_system)
                .with_system(
                    instantiate_fire_laser_system
                        .system()
                        .label(FireLaserLabel::Instantiate),
                )
                .with_system(
                    shoot_fire_laser_system
                        .system()
                        .label(FireLaserLabel::Shoot)
                        .after(FireLaserLabel::Instantiate),
                )
                .with_system(
                    drawing_system
                        .system()
                        .label(FireLaserLabel::Drawing)
                        .after(FireLaserLabel::Shoot),
                )
                .with_system(
                    hide_fire_laser_system
                        .system()
                        .label(FireLaserLabel::Hiding)
                        .after(FireLaserLabel::Drawing),
                )
                .with_system(
                    detect_end_fire_laser_system
                        .system()
                        .after(FireLaserLabel::Hiding)
                        .label(FireLaserLabel::DetectEnd),
                ),
        )
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(IS_MOVE_PLAYER_STATE)
                .with_system(hide_aiming_laser_system),
        );
    }
}

#[derive(Clone, Hash, Debug, PartialEq, Eq, SystemLabel)]
pub enum LaserLabel {
    Main,
    CalculateLaser,
    DrawLaser,
}

pub struct FireLaserRes {
    pub amount: u32,
    pub shot: u32,
}

#[derive(Component)]
pub struct Laser {
    origin: Vec2,
    destination: Vec2,
    is_visible: bool,
}
impl Default for Laser {
    fn default() -> Self {
        Self {
            origin: Default::default(),
            destination: Default::default(),
            is_visible: false,
        }
    }
}

#[derive(Component)]
struct Aiming;
#[derive(Component)]
struct Fire;
#[derive(Component)]
struct Shooting {
    dir: Vec2,
}

fn aiming_startup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    for _ in 0..AIMING_LASERS {
        commands
            .spawn_bundle(SpriteBundle {
                material: materials.add(Color::rgb(1.0, 0.5, 0.0).into()),
                visible: Visible {
                    is_visible: false,
                    is_transparent: false,
                },
                ..Default::default()
            })
            .insert(Laser::default())
            .insert(Aiming);
    }
}

fn aiming_system(
    query_cannon: Query<(&GlobalTransform, &Transform), With<Cannon>>,
    query_collider: Query<(&Collider, &Transform)>,
    mut query_laser: Query<&mut Laser, With<Aiming>>,
) {
    let (global_transform_cannon, transform_cannon) = query_cannon.single();
    let mut pos = global_transform_cannon.translation.xy() + transform_cannon.translation.xy();
    let quat = transform_cannon.rotation.to_axis_angle();
    let a = -1.0 * quat.0.z * quat.1;
    let mut dir = Vec2::new(a.sin(), a.cos()).normalize();

    let mut normal = Vec2::ZERO;

    let mut search_collide = true;

    for mut laser in query_laser.iter_mut() {
        if search_collide == true {
            search_collide = false;
            laser.origin = pos;
            let mut min_distance_square = f32::MAX;

            //find closest collide and sets lasers start and end position;
            for (collider, collider_transform) in query_collider.iter() {
                if let Some((hit, n)) = collider.ray_collide(collider_transform, pos, dir) {
                    let distance_squared = (hit - pos).length_squared();
                    if distance_squared < min_distance_square {
                        min_distance_square = distance_squared;
                        laser.destination = hit;
                        normal = n;
                        search_collide = true;
                    }
                }
            }

            if search_collide == false {
                laser.is_visible = true;
                laser.destination = dir * MAX_LASER_LENGTH;
            } else {
                let a = normal.angle_between(dir);
                let angle = 2.0 * a.abs() - std::f32::consts::PI;
                pos = laser.destination - dir * HIT_OFFSET;
                dir = Mat2::from_angle(-a.signum() * angle) * dir;
                laser.is_visible = true;
            }
        } else {
            laser.is_visible = false;
        }
    }
}

fn drawing_system(mut query: Query<(&Laser, &mut Transform, &mut Sprite, &mut Visible)>) {
    for (laser, mut transform, mut sprite, mut visible) in query.iter_mut() {
        if laser.is_visible {
            let pos = (laser.origin + laser.destination) / 2.0;
            transform.translation = Vec3::new(pos.x, pos.y, 0.0);
            let dir = laser.destination - laser.origin;
            let angle = Vec2::Y.angle_between(dir);
            transform.rotation = Quat::from_rotation_z(angle);

            sprite.size = Vec2::new(LASER_WIDTH, dir.length());

            visible.is_visible = true;
        } else {
            visible.is_visible = false;
        }
    }
}

#[derive(Clone, Hash, Debug, PartialEq, Eq, SystemLabel)]
enum FireLaserLabel {
    Main,
    Instantiate,
    Shoot,
    Drawing,
    Hiding,
    DetectEnd,
}

fn run_if_fire_laser_chain(
    In(should_run): In<ShouldRun>,
    game_state: Res<GameStateRes>,
) -> ShouldRun {
    match IS_FIRE_LASER_STATE(game_state) {
        ShouldRun::Yes => should_run,
        _ => ShouldRun::No,
    }
}

fn instantiate_fire_laser_system(
    mut commands: Commands,
    mut fire_laser: ResMut<FireLaserRes>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    query_cannon: Query<(&GlobalTransform, &Transform), With<Cannon>>,
    mut query_laser: Query<(Entity, &mut Laser), (With<Fire>, Without<Shooting>)>,
) {
    let (global_transform_cannon, transform_cannon) = query_cannon.single();
    let pos = global_transform_cannon.translation.xy() + transform_cannon.translation.xy();
    let quat = transform_cannon.rotation.to_axis_angle();
    let a = -1.0 * quat.0.z * quat.1;
    let dir = Vec2::new(a.sin(), a.cos()).normalize();
    if fire_laser.amount > fire_laser.shot {
        if let Some((entity, mut laser)) = query_laser.iter_mut().next() {
            laser.origin = pos;
            laser.destination = pos;
            laser.is_visible = true;
            commands.entity(entity).insert(Shooting { dir });
        } else {
            commands
                .spawn_bundle(SpriteBundle {
                    material: materials.add(
                        if fire_laser.shot % 2 == 0 {
                            Color::rgb(0.9, 0.1, 0.0)
                        } else {
                            Color::rgb(0.8, 0.2, 1.0)
                        }
                        .into(),
                    ),
                    visible: Visible {
                        is_visible: false,
                        is_transparent: false,
                    },
                    ..Default::default()
                })
                .insert(Laser {
                    origin: pos,
                    destination: pos,
                    is_visible: true,
                })
                .insert(Shooting { dir });
        }
        fire_laser.shot += 1;
    }
}

fn shoot_fire_laser_system(
    mut commands: Commands,
    mut query_collider: Query<(&Collider, &Transform, Option<&mut Live>)>,
    mut query_laser: Query<(Entity, &mut Laser, &mut Shooting)>,
) {
    for (entity, mut laser, mut shooting) in query_laser.iter_mut() {
        let pos = laser.destination;
        let dir = shooting.dir;
        let mut collision = None;
        let mut min_distance = f32::MAX;

        for (collider, transform, o_live) in query_collider.iter_mut() {
            if let Some((hit, n)) = collider.ray_collide(transform, pos, dir) {
                let distance = pos.distance_squared(hit);

                if distance < min_distance {
                    min_distance = distance;
                    collision = Some((hit, n, o_live));
                }
            }
        }

        if let Some((hit, n, o_live)) = collision {
            let a = n.angle_between(dir);
            let angle = 2.0 * a.abs() - std::f32::consts::PI;
            laser.origin = laser.destination;
            laser.destination = hit - dir * HIT_OFFSET;
            shooting.dir = Mat2::from_angle(-a.signum() * angle) * dir;

            if let Some(mut live) = o_live {
                live.0 -= 1;
            }
        } else {
            laser.origin = laser.destination;
            laser.destination += dir * MAX_LASER_LENGTH;
            commands.entity(entity).remove::<Shooting>();
        }
    }
}

fn hide_fire_laser_system(mut query: Query<&mut Laser, (With<Aiming>, Without<Shooting>)>) {
    for mut laser in query.iter_mut() {
        laser.is_visible = false;
    }
}

fn detect_end_fire_laser_system(
    mut fire_laser: ResMut<FireLaserRes>,
    mut game_state: ResMut<GameStateRes>,
    query_laser: Query<(), (With<Laser>, With<Shooting>)>,
) {
    if fire_laser.amount == fire_laser.shot && query_laser.iter().count() == 0 {
        info!("Shot: {}", fire_laser.shot);
        fire_laser.shot = 0;
        game_state.change(GameState::GenerateObstacle);
    }
}

fn hide_aiming_laser_system(
    game_state: Res<GameStateRes>,
    mut query: Query<(&mut Visible, &mut Laser), (With<Sprite>, With<Aiming>)>,
) {
    if game_state.is_changed() {
        for (mut visible, mut laser) in query.iter_mut() {
            visible.is_visible = false;
            laser.is_visible = false;
        }
        info!("Hide Lasers!")
    }
}
