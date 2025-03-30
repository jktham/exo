use glam::Quat;
use glam::{Mat4, Vec3};
use rand::Rng;
use rand_distr::StandardNormal;

use crate::graphics::*;
use crate::sprites::*;

pub struct Game {
    pub ship: Ship,
    pub camera: Camera,
    pub stars: Vec<Object>,
}

pub struct Ship {
    pub position: Vec3,
    pub velocity: Vec3,
    pub acceleration: Vec3,
    pub rotation: Quat,
    pub angular_velocity: Quat,
    pub angular_acceleration: Quat,
    pub thrusters: Thrusters,
	pub object: Object,
}

#[derive(Default)]
pub struct Thrusters {
    pub left: f32,
    pub right: f32,
    pub up: f32,
    pub down: f32,
    pub front: f32,
    pub back: f32,
    pub yaw_left: f32,
    pub yaw_right: f32,
    pub pitch_up: f32,
    pub pitch_down: f32,
    pub roll_ccw: f32,
    pub roll_cw: f32,
}

pub struct Camera {
    pub position: Vec3,
    pub rotation: Quat,
    pub fov: f32,
    pub model: Mat4,
    pub view: Mat4,
}

pub struct Object {
	pub mesh: Vec<Vec<Vec3>>,
	pub model: Mat4,
	pub color: u32,
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

        Self {
            ship: Ship {
                position: Vec3::new(0.0, 0.0, -1.0),
                velocity: Vec3::ZERO,
                acceleration: Vec3::ZERO,
                rotation: Quat::IDENTITY,
                angular_velocity: Quat::IDENTITY,
                angular_acceleration: Quat::IDENTITY,
                thrusters: Default::default(),
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
                position: Vec3::new(0.0, 0.0, 0.0),
                rotation: Quat::IDENTITY,
                fov: 90.0,
                model: Mat4::IDENTITY,
                view: Mat4::IDENTITY,
            },
            stars,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.ship.angular_acceleration = Quat::from_euler(
            glam::EulerRot::XYZ,
            (self.ship.thrusters.pitch_up - self.ship.thrusters.pitch_down) * dt*dt, // todo: apply delta properly
            (self.ship.thrusters.yaw_left - self.ship.thrusters.yaw_right) * dt*dt,
            (self.ship.thrusters.roll_ccw - self.ship.thrusters.roll_cw) * dt*dt,
        );
        self.ship.angular_velocity *= self.ship.angular_acceleration;
        self.ship.rotation *= self.ship.angular_velocity;

        self.ship.acceleration = self.ship.rotation * Vec3::new(
            self.ship.thrusters.right - self.ship.thrusters.left,
            self.ship.thrusters.up - self.ship.thrusters.down,
            self.ship.thrusters.back - self.ship.thrusters.front,
        );
        self.ship.velocity += self.ship.acceleration * dt;
        self.ship.position += self.ship.velocity * dt;

		self.ship.object.model = Mat4::from_rotation_translation(self.ship.rotation, self.ship.position);

        let position_offset = Vec3::new(0.0, 2.0, 4.0);
        let rotation_offset = Vec3::new(0.0, 1.0, 0.0);
        let trailing_factor = 0.9;
        self.camera.position = self.camera.position * trailing_factor + (self.ship.position + self.ship.rotation * position_offset) * (1.0 - trailing_factor);
        self.camera.rotation = Quat::look_at_rh(self.camera.position, self.ship.position + self.ship.rotation * rotation_offset, self.ship.rotation * Vec3::new(0.0, 1.0, 0.0)).inverse();
        self.camera.model = Mat4::from_rotation_translation(self.camera.rotation, self.camera.position);
        self.camera.view = self.camera.model.inverse();
    }

    pub fn draw(&self, frame: &mut [u8]) {
        clear(frame, 0x000000ff);

        for star in &self.stars {
            draw_object(frame, star, &self.camera);
        }
		draw_object(frame, &self.ship.object, &self.camera);

        draw_line_3d(frame, self.ship.position, self.ship.position + Vec3::new(1.0, 0.0, 0.0), Mat4::IDENTITY, Mat4::IDENTITY, &self.camera, 0xff0000ff);
        draw_line_3d(frame, self.ship.position, self.ship.position + Vec3::new(0.0, 1.0, 0.0), Mat4::IDENTITY, Mat4::IDENTITY, &self.camera, 0x00ff00ff);
        draw_line_3d(frame, self.ship.position, self.ship.position + Vec3::new(0.0, 0.0, 1.0), Mat4::IDENTITY, Mat4::IDENTITY, &self.camera, 0x0000ffff);

        draw_rectangle_fill(frame, 0, 0, 6, 6, if self.ship.thrusters.left > 0.0 {0xffffffff} else {0x000000ff});
        draw_text(frame, 1, 1, "A", &FONT_5PX, 6, 1, if self.ship.thrusters.left > 0.0 {0x000000ff} else {0xffffffff});
        draw_rectangle_fill(frame, 7, 7, 13, 13, if self.ship.thrusters.front > 0.0 {0xffffffff} else {0x000000ff});
        draw_text(frame, 8, 8, "W", &FONT_5PX, 6, 1, if self.ship.thrusters.front > 0.0 {0x000000ff} else {0xffffffff});
        draw_rectangle_fill(frame, 7, 0, 13, 6, if self.ship.thrusters.back > 0.0 {0xffffffff} else {0x000000ff});
        draw_text(frame, 8, 1, "S", &FONT_5PX, 6, 1, if self.ship.thrusters.back > 0.0 {0x000000ff} else {0xffffffff});
        draw_rectangle_fill(frame, 14, 0, 20, 6, if self.ship.thrusters.right > 0.0 {0xffffffff} else {0x000000ff});
        draw_text(frame, 15, 1, "D", &FONT_5PX, 6, 1, if self.ship.thrusters.right > 0.0 {0x000000ff} else {0xffffffff});
        draw_rectangle_fill(frame, 21, 7, 27, 13, if self.ship.thrusters.up > 0.0 {0xffffffff} else {0x000000ff});
        draw_text(frame, 22, 8, "R", &FONT_5PX, 6, 1, if self.ship.thrusters.up > 0.0 {0x000000ff} else {0xffffffff});
        draw_rectangle_fill(frame, 21, 0, 27, 6, if self.ship.thrusters.down > 0.0 {0xffffffff} else {0x000000ff});
        draw_text(frame, 22, 1, "F", &FONT_5PX, 6, 1, if self.ship.thrusters.down > 0.0 {0x000000ff} else {0xffffffff});
        draw_rectangle_fill(frame, 28, 0, 34, 6, if self.ship.velocity.length() == 0.0 && self.ship.angular_velocity.to_axis_angle().1 == 0.0 {0xffffffff} else {0x000000ff});
        draw_text(frame, 29, 1, "_", &FONT_5PX, 6, 1, if self.ship.velocity.length() == 0.0 && self.ship.angular_velocity.to_axis_angle().1 == 0.0 {0x000000ff} else {0xffffffff});
        draw_rectangle_fill(frame, 35, 0, 41, 6, if self.ship.thrusters.yaw_left > 0.0 {0xffffffff} else {0x000000ff});
        draw_text(frame, 36, 1, "J", &FONT_5PX, 6, 1, if self.ship.thrusters.yaw_left > 0.0 {0x000000ff} else {0xffffffff});
        draw_rectangle_fill(frame, 42, 7, 48, 13, if self.ship.thrusters.pitch_down > 0.0 {0xffffffff} else {0x000000ff});
        draw_text(frame, 43, 8, "I", &FONT_5PX, 6, 1, if self.ship.thrusters.pitch_down > 0.0 {0x000000ff} else {0xffffffff});
        draw_rectangle_fill(frame, 42, 0, 48, 6, if self.ship.thrusters.pitch_up > 0.0 {0xffffffff} else {0x000000ff});
        draw_text(frame, 43, 1, "K", &FONT_5PX, 6, 1, if self.ship.thrusters.pitch_up > 0.0 {0x000000ff} else {0xffffffff});
        draw_rectangle_fill(frame, 49, 0, 55, 6, if self.ship.thrusters.yaw_right > 0.0 {0xffffffff} else {0x000000ff});
        draw_text(frame, 50, 1, "L", &FONT_5PX, 6, 1, if self.ship.thrusters.yaw_right > 0.0 {0x000000ff} else {0xffffffff});
        draw_rectangle_fill(frame, 35, 7, 41, 13, if self.ship.thrusters.roll_ccw > 0.0 {0xffffffff} else {0x000000ff});
        draw_text(frame, 36, 8, "U", &FONT_5PX, 6, 1, if self.ship.thrusters.roll_ccw > 0.0 {0x000000ff} else {0xffffffff});
        draw_rectangle_fill(frame, 49, 7, 55, 13, if self.ship.thrusters.roll_cw > 0.0 {0xffffffff} else {0x000000ff});
        draw_text(frame, 50, 8, "O", &FONT_5PX, 6, 1, if self.ship.thrusters.roll_cw > 0.0 {0x000000ff} else {0xffffffff});
    }
}