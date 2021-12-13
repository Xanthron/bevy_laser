use bevy::{
    math::{Mat2, Vec3Swizzles},
    prelude::*,
};

#[derive(Component)]
pub struct Collider(Vec<Vec2>);

impl Collider {
    pub fn new(vec: Vec<Vec2>) -> Collider {
        Collider(vec)
    }

    pub fn ray_collide(&self, transform: &Transform, pos: Vec2, dir: Vec2) -> Option<(Vec2, Vec2)> {
        let translation = transform.translation.xy();
        let mut iter = self.0.iter();
        let mut c0 = *iter.next().unwrap() + translation;

        let mut ret: Option<(Vec2, Vec2)> = None;

        while let Some(&c1) = iter.next() {
            let c1 = c1 + translation;
            let dir_c = c1 - c0;
            let mat = Mat2::from_cols(dir_c, -dir).inverse();
            if !mat.is_nan() {
                let t = mat * (pos - c0);
                if t.x >= 0.0 && t.x <= 1.0 && t.y > 0.0 {
                    let hit = c0 + dir_c * t.x;

                    let angle = dir.angle_between(dir_c);
                    let n =
                        (Mat2::from_angle(angle + angle.signum() * std::f32::consts::FRAC_PI_2)
                            * dir)
                            .normalize();

                    if let Some(r) = ret {
                        if (hit - pos).length_squared() < (r.0 - pos).length_squared() {
                            ret = Some((hit, n));
                        }
                    } else {
                        ret = Some((hit, n));
                    }
                }
            }
            c0 = c1;
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
        let mut vec = Vec::with_capacity(5);
        vec.push(Vec2::new(-width / 2.0, -height / 2.0));
        vec.push(Vec2::new(width / 2.0, -height / 2.0));
        vec.push(Vec2::new(width / 2.0, height / 2.0));
        vec.push(Vec2::new(-width / 2.0, height / 2.0));
        vec.push(Vec2::new(-width / 2.0, -height / 2.0));
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

#[test]
fn test_collider() {
    //TODO Assert
    {
        let collider = Collider::new(vec![Vec2::new(0.0, 0.0), Vec2::new(0.0, 7.0)]);
        let result = collider.ray_collide(
            &Transform::default(),
            Vec2::new(-3.5, 0.0),
            Vec2::new(1.0, 1.0).normalize(),
        );
        println!("{:?}", result);
    }
    {
        let collider = Collider::new(vec![Vec2::new(0.0, 0.0), Vec2::new(0.0, 7.0)]);
        let result = collider.ray_collide(
            &Transform::default(),
            Vec2::new(3.5, 0.0),
            Vec2::new(-1.0, 1.0).normalize(),
        );
        println!("{:?}", result);
    }

    {
        let collider = Block::new_collider((2.0, 2.0).into());
        let result = collider.ray_collide(
            &Transform::from_xyz(2.0, 2.0, 0.0),
            Vec2::new(0.0, 1.0),
            Vec2::new(1.0, 0.5),
        );
        println!("{:?}", result);
    }
}
