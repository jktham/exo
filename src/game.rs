use glam::Vec3;

use crate::graphics::*;

pub struct Camera {
    pub position: Vec3,
    pub direction: Vec3,
    pub fov: f32,
}

pub struct Ship {
    pub position: Vec3,
    pub velocity: Vec3,
    pub thrust: Vec3,
}

pub struct Game {
    pub ship: Ship,
    pub camera: Camera,
}

impl Game {
    pub fn new() -> Self {
        Self {
            ship: Ship {
                position: Vec3::new(0.0, 0.0, 0.0),
                velocity: Vec3::new(0.0, 0.0, 0.0),
                thrust: Vec3::new(0.0, 0.0, 0.0),
            },
            camera: Camera {
                position: Vec3::new(0.0, 0.0, 1.0),
                direction: Vec3::new(0.0, 0.0, -1.0),
                fov: 90.0,
            },
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.ship.velocity += self.ship.thrust * dt;
        self.ship.position += self.ship.velocity * dt;

		// self.camera.direction = Vec3::normalize(self.ship.position - self.camera.position);
    }

    pub fn draw(&self, frame: &mut [u8]) {
        let p0 = transform(self.ship.position, &self.camera);
        let p1 = transform(self.ship.position + Vec3::new(1.0, -1.0, 0.0), &self.camera);

        clear(frame, 0x000000ff);
        draw_rectangle(frame, p0.x as i32, p0.y as i32, p1.x as i32, p1.y as i32, 0xff0000ff);
        draw_line(frame, p0.x as i32, p0.y as i32, p1.x as i32, p1.y as i32, 0x0000ffff);
        draw_pixel(frame, p0.x as i32, p0.y as i32, 0xffffffff);
    }
}