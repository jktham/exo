use glam::{Mat4, Vec3};
use rand::Rng;
use rand_distr::StandardNormal;

use crate::graphics::*;

pub struct Object {
	pub mesh: Vec<Vec<Vec3>>,
	pub model: Mat4,
	pub color: u32,
}

pub struct Camera {
    pub position: Vec3,
    pub direction: Vec3,
    pub fov: f32,
}

pub struct Ship {
    pub position: Vec3,
    pub velocity: Vec3,
    pub thrust: Vec3,
	pub object: Object,
}

pub struct Game {
    pub ship: Ship,
    pub camera: Camera,
    pub stars: Vec<Object>,
}

impl Game {
    pub fn new() -> Self {
        let mut stars = Vec::new();
        for _ in 0..1000 {
            let pos = Vec3::new(
                rand::rng().sample::<f32, StandardNormal>(StandardNormal), 
                rand::rng().sample::<f32, StandardNormal>(StandardNormal), 
                rand::rng().sample::<f32, StandardNormal>(StandardNormal),
            ).normalize() * 1000.0;
            let b = (rand::rng().random::<f32>() * 255.0) as u32 & 0xff;
            let col = (b << 24) | (b << 16) | (b << 8) | 0xff;
            stars.push(Object {
                mesh: Vec::from([Vec::from([pos])]),
                model: Mat4::IDENTITY,
                color: col,
            });
        };

        return Self {
            ship: Ship {
                position: Vec3::new(0.0, 0.0, 0.0),
                velocity: Vec3::new(0.0, 0.0, 0.0),
                thrust: Vec3::new(0.0, 0.0, 0.0),
				object: Object {
					mesh: Vec::from([
                        Vec::from([
                            Vec3::new(0.0, 0.0, -1.0),
                            Vec3::new(1.0, 0.5, 0.0),
                            Vec3::new(-1.0, 0.5, 0.0),
                        ]),
                        Vec::from([
                            Vec3::new(0.0, 0.0, -1.0),
                            Vec3::new(1.0, -0.5, 0.0),
                            Vec3::new(-1.0, -0.5, 0.0),
                        ]),
                        Vec::from([
                            Vec3::new(1.0, 0.5, 0.0),
                            Vec3::new(-1.0, 0.5, 0.0),
                            Vec3::new(-1.0, -0.5, 0.0),
                            Vec3::new(1.0, -0.5, 0.0),
                        ]),
					]),
					model: Mat4::IDENTITY,
					color: 0xffffffff,
				}
            },
            camera: Camera {
                position: Vec3::new(0.0, 0.0, 1.0),
                direction: Vec3::new(0.0, 0.0, -1.0),
                fov: 90.0,
            },
            stars: stars,
        };
    }

    pub fn update(&mut self, dt: f32) {
        self.ship.velocity += self.ship.thrust * dt;
        self.ship.position += self.ship.velocity * dt;

		self.ship.object.model = Mat4::from_translation(self.ship.position);

		self.camera.direction = Vec3::normalize(self.ship.position - self.camera.position);
    }

    pub fn draw(&self, frame: &mut [u8]) {
        clear(frame, 0x000000ff);

        for star in &self.stars {
            draw_object(frame, star, &self.camera);
        }
		draw_object(frame, &self.ship.object, &self.camera);
		draw_point_3d(frame, self.ship.position + Vec3::new(0.0, 0.0, -0.5), Mat4::IDENTITY, &self.camera, 0xff00ffff);

        let p0 = transform(self.ship.position, Mat4::IDENTITY, &self.camera);
        let p1 = transform(self.ship.position + Vec3::new(1.0, -1.0, 0.0), Mat4::IDENTITY, &self.camera);
        draw_rectangle(frame, p0.x as i32, p0.y as i32, p1.x as i32, p1.y as i32, 0xff0000ff);
        draw_line(frame, p0.x as i32, p0.y as i32, p1.x as i32, p1.y as i32, 0x0000ffff);
        draw_pixel(frame, p0.x as i32, p0.y as i32, 0x00ff00ff);
		draw_rectangle_fill(frame, 0, 0, 7, 7, 0xffffffff);
    }
}