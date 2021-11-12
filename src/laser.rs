use crate::{
    collider2d::Collider,
    player::{Cannon, Player},
};
use bevy::{ecs::query, prelude::*};

pub const AIMING_LASERS: usize = 3;

pub struct LaserDataRes {}

#[derive(Component)]
pub struct Laser {
    origin: Vec2,
    destination: Vec2,
}

#[derive(Component)]
struct Aiming;
#[derive(Component)]
struct Shooting;

fn aiming_system(
    query_cannon: Query<(&GlobalTransform, &Transform), With<Cannon>>,
    query_collider: Query<(&Collider, &Transform)>,
    mut query_laser: Query<(&mut Laser, &mut Sprite, &mut Visible, &Transform), With<Aiming>>,
) {
    let (global_transform_cannon, transform_cannon) = query_cannon.single();
    let mut pos = Vec2::from(global_transform_cannon.translation + transform_cannon.translation);
    let mut dir = Vec2::from(transform_cannon.rotation.mul_vec3(Vec3::new(0.0, 1.0, 0.0)));

    let mut search_collide = true;

    for (mut laser, mut _sprite, mut visible, mut transform) in query_laser.iter_mut() {
        if search_collide == true {
            search_collide = false;
            laser.origin = pos;
            let mut min_distance_square = f32::MAX;
            for (collider, collider_transform) in query_collider.iter() {
                if let Some((hit, angle)) = collider.ray_collide(collider_transform, pos, dir) {
                    let distance_squared = (hit - pos).length_squared();
                    if distance_squared < min_distance_square {
                        min_distance_square = distance_squared;
                        laser.destination = hit;
                        //TODO direction
                        search_collide = true;
                    }
                }
            }
            pos = laser.destination;
        }
        if search_collide == false {
        } else {
            visible.is_visible = true;
        }
    }
}
