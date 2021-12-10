use crate::{
    collider2d::Collider,
    player::{Cannon, Player},
    window::{HEIGHT, WIDTH},
    GameStateRes,
};
use bevy::{
    ecs::{query, schedule::ShouldRun, system::Command},
    prelude::*,
};

pub const AIMING_LASERS: usize = 3;

pub const MAX_LASER_LENGTH: f32 = WIDTH * WIDTH + HEIGHT * HEIGHT; //FIXME take the square root but that is not supported vor const.
pub const LASER_WIDTH: f32 = 4.0;

pub struct LaserPlugin;

impl Plugin for LaserPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(aiming_startup).add_system_set(
            SystemSet::new()
                .with_run_criteria(is_aiming_state)
                .label(LaserLabel::Main)
                .with_system(aiming_system.system().label(LaserLabel::CalculateLaser))
                .with_system(
                    drawing_system
                        .system()
                        .label(LaserLabel::DrawLaser)
                        .after(LaserLabel::CalculateLaser),
                ),
        );
    }
}

#[derive(Clone, Hash, Debug, PartialEq, Eq, SystemLabel)]
pub enum LaserLabel {
    Main,
    CalculateLaser,
    DrawLaser,
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
struct Shooting;

fn is_aiming_state(game_state: Res<GameStateRes>) -> ShouldRun {
    if game_state.eq(&GameStateRes::PlayerMovement) {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
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
    let mut pos = Vec2::from(global_transform_cannon.translation + transform_cannon.translation);
    let a = transform_cannon.rotation.to_axis_angle().1;
    let a_cos = a.cos();
    let a_sin = a.sin();
    let mut dir = Vec2::new(a_cos * 1.0, a_sin * 1.0);
    info!("{},{}", pos, dir);

    let mut normal = None;

    let mut search_collide = true;

    for mut laser in query_laser.iter_mut() {
        if search_collide == true {
            search_collide = false;
            laser.origin = pos;
            let mut min_distance_square = f32::MAX;

            //find closest collide and sets Lasers start and end position;
            for (collider, collider_transform) in query_collider.iter() {
                if let Some((hit, n)) = collider.ray_collide(collider_transform, pos, dir) {
                    info!("JO");
                    let distance_squared = (hit - pos).length_squared();
                    if distance_squared < min_distance_square {
                        min_distance_square = distance_squared;
                        laser.destination = hit;
                        normal = Some(n);
                        search_collide = true;
                    }
                }
            }
        }

        if search_collide == false {
            laser.is_visible = false;
        } else {
            let n = normal.unwrap();
            //TODO Move to other point
            let a = n.angle_between(dir);

            let a_cos = a.cos();
            let a_sin = a.sin();

            dir = Vec2::new(a_cos * dir.x - a_sin * dir.y, a_sin * dir.x + a_cos * dir.y);
            pos = laser.destination;
            laser.is_visible = true;
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
            transform.rotation = Quat::from_rotation_y(angle);

            sprite.size = Vec2::new(LASER_WIDTH, dir.length());

            visible.is_visible = true;
        } else {
            visible.is_visible = false;
        }
    }
}
