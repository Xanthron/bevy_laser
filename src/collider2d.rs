use bevy::{math::Vec3Swizzles, prelude::*};

#[derive(Component)]
pub struct Collider(Vec<Vec2>);
impl Collider {
    pub fn ray_collide(
        &self,
        transform: &Transform,
        pos: Vec2,
        direction: Vec2,
    ) -> Option<(Vec2, Vec2)> {
        let mut vec_iter = self.0.iter();
        let translation = transform.translation.xy();
        let first = *vec_iter.next().unwrap() + translation;
        let mut last = first;

        let mut min_distance_squared = f32::MAX;

        let mut ret = None;

        for vec in vec_iter {
            let vec = *vec + translation;
            if let Some(hit) = line_line_intersection(pos, pos + direction, last, vec) {
                ray_hit(
                    pos,
                    direction,
                    hit,
                    last,
                    vec,
                    &mut min_distance_squared,
                    &mut ret,
                );
            }
            last = vec;
        }
        if let Some(hit) = line_line_intersection(pos, pos + direction, last, first) {
            ray_hit(
                pos,
                direction,
                hit,
                last,
                first,
                &mut min_distance_squared,
                &mut ret,
            );
        }

        ret
    }
}

pub struct Block {
    _size: Vec2,
}
impl Block {
    pub fn new(size: Vec2) -> Self {
        Self { _size: size }
    }

    pub fn new_collider(size: Vec2) -> Collider {
        let (width, height) = size.into();
        let mut vec = Vec::with_capacity(4);
        vec.push(Vec2::new(-width / 2.0, -height / 2.0));
        vec.push(Vec2::new(width / 2.0, -height / 2.0));
        vec.push(Vec2::new(width / 2.0, height / 2.0));
        vec.push(Vec2::new(-width / 2.0, height / 2.0));
        Collider(vec)
    }
}
pub struct Triangle {
    _size: Vec2,
}
impl Triangle {
    pub fn new(size: Vec2) -> Self {
        Self { _size: size }
    }

    pub fn new_collider(size: Vec2) -> Collider {
        let (width, height) = size.into();
        let mut vec = Vec::with_capacity(3);
        vec.push(Vec2::new(-width / 2.0, -height / 2.0));
        vec.push(Vec2::new(width / 2.0, -height / 2.0));
        vec.push(Vec2::new(width / 2.0, height / 2.0));
        Collider(vec)
    }
}

pub fn line_line_intersection(a1: Vec2, a2: Vec2, b1: Vec2, b2: Vec2) -> Option<Vec2> {
    //line1
    let ax = a1.x - a2.x;
    let ay = a2.y - a1.y;

    //line2
    let bx = b1.x - b2.x;
    let by = b2.y - b1.y;

    let det = ay * bx - by * ax;

    if det == 0.0 {
        None
    } else {
        let ca = ay * a1.x + ax * a1.y;
        let cb = by * b1.x + bx * b1.y;

        let vec = Vec2::new((bx * ca - ax * cb) / det, (ay * cb - by * ca) / det);

        Some(vec)
    }
}

fn line_direction(dir1: Vec2, dir2: Vec2) -> f32 {
    f32::max(dir2.x / dir1.x, dir2.y / dir1.y)
}

fn ray_hit(
    pos: Vec2,
    dir: Vec2,
    hit: Vec2,
    a1: Vec2,
    a2: Vec2,
    min_distance_squared: &mut f32,
    value: &mut Option<(Vec2, Vec2)>,
) {
    if line_direction(dir, hit - pos) >= 0.0 {
        let a = a2 - a1;
        let line_dir = line_direction(a2 - a1, hit - a1);
        info!("{}", line_dir);
        if line_dir >= 0.0 && line_dir <= 1.0 {
            let distance_squared = pos.distance_squared(hit);
            if distance_squared < *min_distance_squared {
                *min_distance_squared = distance_squared;

                *value = Some((hit, Vec2::new(a.y, -a.x).normalize()));
            }
        }
    }
}

#[test]
fn test_line_line_intersection() {
    assert_eq!(
        line_line_intersection(
            Vec2::new(0.0, 0.0),
            Vec2::new(2.0, 2.0),
            Vec2::new(0.0, 1.0),
            Vec2::new(2.0, 1.0),
        ),
        Some(Vec2::new(1.0, 1.0))
    );
}

#[test]
fn test_collider() {
    let collider = Block::new_collider((2.0, 2.0).into());

    assert_eq!(
        Some((Vec2::new(1.0, 1.5), Vec2::new(-1.0, 0.0))),
        collider.ray_collide(
            &Transform::from_xyz(2.0, 2.0, 0.0),
            Vec2::new(0.0, 1.0),
            Vec2::new(1.0, 0.5),
        ),
    );
}
