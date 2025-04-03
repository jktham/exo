use enum_map::{enum_map, Enum, EnumMap};
use glam::Quat;
use glam::{Mat4, Vec3};
use rand::Rng;
use rand_distr::StandardNormal;

use crate::{graphics::*, HEIGHT};
use crate::sprites::*;
use crate::meshes::*;

pub struct Game {
    pub ship: Ship,
    pub camera: Camera,
    pub stars: Vec<Object>,
    pub dust: Vec<Object>,
    pub particles: Vec<Particle>,
}

pub struct Ship {
    pub position: Vec3,
    pub velocity: Vec3,
    pub acceleration: Vec3,
    pub rotation: Quat,
    pub angular_velocity: Quat,
    pub angular_acceleration: Quat,
    pub thrust: EnumMap<Thrust, f32>,
    pub hull: Object,
    pub thrusters: EnumMap<Thrust, Object>,
}

#[derive(Enum)]
pub enum Thrust {
    Left,
    Right,
    Up,
    Down,
    Front,
    Back,
    YawLeft,
    YawRight,
    PitchUp,
    PitchDown,
    RollCCW,
    RollCW,
}

pub struct Camera {
    pub position: Vec3,
    pub rotation: Quat,
    pub fov: f32,
    pub model: Mat4,
    pub view: Mat4,
}

#[derive(Clone)]
pub struct Object {
    pub mesh: Vec<Vec<Vec3>>,
    pub model: Mat4,
    pub color: u32,
}

pub struct Particle {
    pub object: Object,
    pub lifetime: f32,
}

impl Game {
    pub fn new() -> Self {
        Self {
            ship: Ship {
                position: Vec3::ZERO,
                velocity: Vec3::ZERO,
                acceleration: Vec3::ZERO,
                rotation: Quat::IDENTITY,
                angular_velocity: Quat::IDENTITY,
                angular_acceleration: Quat::IDENTITY,
                thrust: enum_map! {_ => 0.0},
                hull: Object {
                    mesh: hull_mesh(),
                    model: Mat4::IDENTITY,
                    color: 0xffffffff,
                },
                thrusters: create_thrusters(),
            },
            camera: Camera {
                position: Vec3::ZERO,
                rotation: Quat::IDENTITY,
                fov: 90.0,
                model: Mat4::IDENTITY,
                view: Mat4::IDENTITY,
            },
            stars: generate_stars(),
            dust: update_dust(Vec::new(), Vec3::ZERO, true),
            particles: Vec::new(),
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.ship.angular_acceleration = Quat::from_euler(
            glam::EulerRot::XYZ,
            (self.ship.thrust[Thrust::PitchUp] - self.ship.thrust[Thrust::PitchDown]) * dt*dt, // todo: apply delta properly
            (self.ship.thrust[Thrust::YawLeft] - self.ship.thrust[Thrust::YawRight]) * dt*dt,
            (self.ship.thrust[Thrust::RollCCW] - self.ship.thrust[Thrust::RollCW]) * dt*dt,
        );
        self.ship.angular_velocity *= self.ship.angular_acceleration;
        self.ship.rotation *= self.ship.angular_velocity;

        self.ship.acceleration = self.ship.rotation * Vec3::new(
            self.ship.thrust[Thrust::Right] - self.ship.thrust[Thrust::Left],
            self.ship.thrust[Thrust::Up] - self.ship.thrust[Thrust::Down],
            self.ship.thrust[Thrust::Back] - self.ship.thrust[Thrust::Front],
        );
        self.ship.velocity += self.ship.acceleration * dt;
        self.ship.position += self.ship.velocity * dt;

        self.ship.hull.model = Mat4::from_rotation_translation(self.ship.rotation, self.ship.position);

        let position_offset = Vec3::new(0.0, 4.0, 10.0);
        let rotation_offset = Vec3::new(0.0, 0.0, 0.0);
        let trailing_factor = 0.85;
        self.camera.position = self.camera.position * trailing_factor + (self.ship.position + self.ship.rotation * position_offset) * (1.0 - trailing_factor);
        self.camera.rotation = Quat::look_at_rh(self.camera.position, self.ship.position + self.ship.rotation * rotation_offset, self.ship.rotation * Vec3::new(0.0, 1.0, 0.0)).inverse();
        self.camera.model = Mat4::from_rotation_translation(self.camera.rotation, self.camera.position);
        self.camera.view = self.camera.model.inverse();

        for star in &mut self.stars {
            star.model = Mat4::from_translation(self.camera.position);
        }
        self.dust = update_dust(self.dust.clone(), self.camera.position, false);

        for particle in &mut self.particles {
            particle.lifetime -= dt;
            if particle.lifetime < 1.0 {
                let (r, g, b, a) = color_to_float(0xff00ffff);
                let brightness = f32::max(0.0, particle.lifetime);
                particle.object.color = float_to_color(brightness*r, brightness*g, brightness*b, a);
            }
        }
        self.particles.retain(|p| p.lifetime > 0.0);

        let particle_offset = Vec3::new(
            rand::rng().sample::<f32, StandardNormal>(StandardNormal), 
            rand::rng().sample::<f32, StandardNormal>(StandardNormal), 
            rand::rng().sample::<f32, StandardNormal>(StandardNormal),
        ).normalize() * 0.5;
        self.particles.push(Particle {
            object: Object {
                mesh: vec![vec![self.ship.position + particle_offset + self.ship.rotation * Vec3::new(-2.3, 0.0, 3.0)]],
                model: Mat4::IDENTITY,
                color: 0xff00ffff,
            },
            lifetime: 10.0, 
        });
        self.particles.push(Particle {
            object: Object {
                mesh: vec![vec![self.ship.position + particle_offset + self.ship.rotation * Vec3::new(2.3, 0.0, 3.0)]],
                model: Mat4::IDENTITY,
                color: 0xff00ffff,
            },
            lifetime: 10.0, 
        });

        for (_thrust, thruster) in &mut self.ship.thrusters {
            thruster.model = self.ship.hull.model;
        }
    }

    pub fn draw(&self, frame: &mut [u8], dt: f32) {
        clear(frame, 0x000000ff);

        for star in &self.stars {
            draw_object(frame, star, &self.camera);
        }
        for dust in &self.dust {
            draw_object(frame, dust, &self.camera);
        }
        for particle in &self.particles {
            draw_object(frame, &particle.object, &self.camera);
        }

        draw_object(frame, &self.ship.hull, &self.camera);
        for (thrust, thruster) in &self.ship.thrusters {
            if self.ship.thrust[thrust] > 0.0 {
                draw_object(frame, thruster, &self.camera);
            }
        }

        draw_line_3d(frame, self.ship.position, self.ship.position + Vec3::new(1.0, 0.0, 0.0), &self.camera, 0xff0000ff);
        draw_line_3d(frame, self.ship.position, self.ship.position + Vec3::new(0.0, 1.0, 0.0), &self.camera, 0x00ff00ff);
        draw_line_3d(frame, self.ship.position, self.ship.position + Vec3::new(0.0, 0.0, 1.0), &self.camera, 0x0000ffff);

        self.draw_hud(frame, dt);
    }

    pub fn draw_hud(&self, frame: &mut [u8], dt: f32) {
        for (thrust, t) in self.ship.thrust {
            let (x0, y0, x1, y1, key) = match thrust {
                Thrust::Left => (0, 0, 6, 6, "A"),
                Thrust::Right => (14, 0, 20, 6, "D"),
                Thrust::Up => (21, 7, 27, 13, "R"),
                Thrust::Down => (21, 0, 27, 6, "F"),
                Thrust::Front => (7, 7, 13, 13, "W"),
                Thrust::Back => (7, 0, 13, 6, "S"),
                Thrust::YawLeft => (35, 0, 41, 6, "J"),
                Thrust::YawRight => (49, 0, 55, 6, "L"),
                Thrust::PitchUp => (42, 0, 48, 6, "K"),
                Thrust::PitchDown => (42, 7, 48, 13, "I"),
                Thrust::RollCCW => (35, 7, 41, 13, "U"),
                Thrust::RollCW => (49, 7, 55, 13, "O"),
            };
            let bg: u32 = if t > 0.0 {0xffffffff} else {0x00000000};
            let fg: u32 = if t > 0.0 {0x00000000} else {0xffffffff};
            draw_rectangle_fill(frame, x0, y0, x1, y1, bg);
            draw_text(frame, x0 + 1, y0 + 1, key, &FONT_5PX, 6, 1, fg);
        }
        draw_rectangle_fill(frame, 28, 0, 34, 6, if self.ship.velocity.length() == 0.0 && self.ship.angular_velocity.to_axis_angle().1 == 0.0 {0xffffffff} else {0x000000ff});
        draw_text(frame, 29, 1, "_", &FONT_5PX, 6, 1, if self.ship.velocity.length() == 0.0 && self.ship.angular_velocity.to_axis_angle().1 == 0.0 {0x000000ff} else {0xffffffff});
    
        draw_text(frame, 1, (HEIGHT - 6) as i32, &(f32::round(dt * 1000.0) / 1000.0).to_string(), &FONT_5PX, 6, 1, 0xffffffff);
    }
}

pub fn generate_stars() -> Vec<Object> {
    const COUNT: usize = 1000;

    let mut stars = Vec::new();
    for _ in 0..COUNT {
        let pos = Vec3::new(
            rand::rng().sample::<f32, StandardNormal>(StandardNormal), 
            rand::rng().sample::<f32, StandardNormal>(StandardNormal), 
            rand::rng().sample::<f32, StandardNormal>(StandardNormal),
        ).normalize() * 1000.0;
        let b = (rand::rng().random::<f32>() * 255.0) as u32 & 0xff;
        let col = (b << 24) | (b << 16) | (b << 8) | 0xff;
        stars.push(Object {
            mesh: vec![vec![pos]],
            model: Mat4::IDENTITY,
            color: col,
        });
    }
    stars
}

pub fn update_dust(mut dust: Vec<Object>, center: Vec3, first: bool) -> Vec<Object> {
    const COUNT: usize = 200;
    const MIN_DIST: f32 = 70.0;
    const MAX_DIST: f32 = 80.0;

    dust = dust.iter().filter(|d| (d.mesh[0][0] - center).length() <= MAX_DIST).cloned().collect();
    while dust.len() < COUNT {
        let pos = Vec3::new(
            rand::rng().sample::<f32, StandardNormal>(StandardNormal), 
            rand::rng().sample::<f32, StandardNormal>(StandardNormal), 
            rand::rng().sample::<f32, StandardNormal>(StandardNormal),
        ).normalize() * rand::rng().random_range(if first {0.0} else {MIN_DIST.powf(3.0)}..=MAX_DIST.powf(3.0)).powf(1.0/3.0);
        dust.push(Object {
            mesh: vec![vec![center + pos]],
            model: Mat4::IDENTITY,
            color: 0xffffffff,
        });
    }
    for d in &mut dust {
        let brightness = f32::max(0.0, 1.0 - (d.mesh[0][0] - center).length() / MIN_DIST);
        d.color = float_to_color(brightness, brightness, brightness, 1.0);
    }

    dust
}

pub fn create_thrusters() -> EnumMap<Thrust, Object> {
    let color = 0xff00ffff;
    let thrusters = enum_map! {
        Thrust::Front => Object {
            mesh: front_thruster_mesh(),
            model: Mat4::IDENTITY,
            color: color,
        },
        _ => Object {
            mesh: vec![],
            model: Mat4::IDENTITY,
            color: color,
        },
    };
    thrusters
}