use std::f64::consts::PI;
use std::rc::Rc;

use enum_map::{enum_map, Enum, EnumMap};
use glam::DQuat;
use glam::{DMat4, DVec3};
use rand::Rng;
use rand_distr::StandardNormal;

use crate::transform::FAR;
use crate::{graphics::*, HEIGHT, WIDTH};
use crate::sprites::*;
use crate::meshes::*;

pub struct Game {
    pub ship: Ship,
    pub camera: Camera,
    pub stars: Vec<Object>,
    pub dust: Vec<Object>,
    pub particles: Vec<Particle>,
    pub asteroids: Vec<Asteroid>,
}

pub struct Ship {
    pub position: DVec3,
    pub velocity: DVec3,
    pub acceleration: DVec3,
    pub rotation: DQuat,
    pub angular_velocity: DQuat,
    pub angular_acceleration: DQuat,
    pub thrust: EnumMap<Thrust, f64>,
    pub boost: f64,
    pub boost_cooldown: f64,
    pub jumping: bool,
    pub charging_jump: bool,
    pub jump_charge: f64,
    pub brake: bool,
    pub hull: Object,
    pub thrusters: EnumMap<Thrust, Object>,
    pub stats: ShipStats,
}

pub struct ShipStats {
    pub thrust: f64,
    pub angular_thrust: f64,
    pub boost_strength: f64,
    pub boost_duration: f64,
    pub boost_cooldown: f64,
    pub jump_speed: f64,
    pub jump_charge: f64,
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
    pub position: DVec3,
    pub rotation: DQuat,
    pub fov: f64,
    pub model: DMat4,
    pub view: DMat4,
}

#[derive(Clone)]
pub struct Object {
    pub mesh: Rc<Vec<Vec<DVec3>>>,
    pub model: DMat4,
    pub color: u32,
    pub fill: u32,
}

pub struct Particle {
    pub object: Object,
    pub lifetime: f64,
}

pub struct Asteroid {
    pub object: Object,
    pub rotation_axis: DVec3,
    pub rotation_speed: f64,
}

impl Game {
    pub fn new() -> Self {
        Self {
            ship: Ship {
                position: DVec3::ZERO,
                velocity: DVec3::ZERO,
                acceleration: DVec3::ZERO,
                rotation: DQuat::IDENTITY,
                angular_velocity: DQuat::IDENTITY,
                angular_acceleration: DQuat::IDENTITY,
                thrust: enum_map! {_ => 0.0},
                boost: 0.0,
                boost_cooldown: 0.0,
                brake: false,
                jumping: false,
                charging_jump: false,
                jump_charge: -1.0,
                hull: Object {
                    mesh: Rc::new(hull_mesh()),
                    model: DMat4::IDENTITY,
                    color: 0xffffffff,
                    fill: 0x000000ff,
                },
                thrusters: create_thrusters(),
                stats: ShipStats {
                    thrust: 40.0,
                    angular_thrust: 5.0,
                    boost_strength: 800.0,
                    boost_duration: 0.5,
                    boost_cooldown: 1.0,
                    jump_speed: 18000.0,
                    jump_charge: 3.0,
                },
            },
            camera: Camera {
                position: DVec3::ZERO,
                rotation: DQuat::IDENTITY,
                fov: 90.0,
                model: DMat4::IDENTITY,
                view: DMat4::IDENTITY,
            },
            stars: generate_stars(),
            dust: generate_dust(),
            particles: Vec::new(),
            asteroids: generate_asteroids(),
        }
    }

    pub fn update(&mut self, dt: f64) {
        update_ship_movement(&mut self.ship, dt);
        update_camera_position(&mut self.camera, &self.ship);

        for star in &mut self.stars {
            star.model = DMat4::from_translation(self.camera.position);
        }
        update_dust(&mut self.dust, self.camera.position, false);

        for particle in &mut self.particles {
            particle.lifetime -= dt;
            if particle.lifetime < 1.0 {
                let (r, g, b, a) = color_to_float(particle.object.color);
                let brightness = f64::max(0.0, particle.lifetime);
                particle.object.color = float_to_color((brightness*r, brightness*g, brightness*b, a));
            }
        }
        self.particles.retain(|p| p.lifetime > 0.0);

        add_exhaust_particles(&mut self.particles, &self.ship, dt);

        for (_thrust, thruster) in &mut self.ship.thrusters {
            thruster.model = self.ship.hull.model;
        }

        for asteroid in &mut self.asteroids {
            asteroid.object.model *= DMat4::from_axis_angle(asteroid.rotation_axis, asteroid.rotation_speed * dt);
        }
    }

    pub fn draw(&self, frame: &mut [u8], depth: &mut [f64], dt: f64) {
        if self.ship.jumping {
            clear_depth(depth);
            clear_fade(frame, 0.95);
        } else {
            clear(frame, depth, 0x000000ff);
        }

        for star in &self.stars {
            draw_object(frame, depth, star, &self.camera);
        }
        for dust in &self.dust {
            let trail = -self.ship.velocity * 0.005;
            if trail.length() > 0.1 {
                draw_line_3d(frame, depth, dust.model.transform_point3(DVec3::ZERO), dust.model.transform_point3(DVec3::ZERO) + trail, &self.camera, dust.color);
            }
            draw_object(frame, depth, &dust, &self.camera);
        }
        for particle in &self.particles {
            draw_object(frame, depth, &particle.object, &self.camera);
        }
        for asteroid in &self.asteroids {
            draw_object(frame, depth, &asteroid.object, &self.camera);
        }

        draw_object(frame, depth, &self.ship.hull, &self.camera);
        for (thrust, thruster) in &self.ship.thrusters {
            if self.ship.thrust[thrust] > 0.01 {
                draw_object(frame, depth, thruster, &self.camera);
            }
        }

        draw_line_3d(frame, depth, self.ship.position, self.ship.position + DVec3::new(1.0, 0.0, 0.0), &self.camera, 0xff0000ff);
        draw_line_3d(frame, depth, self.ship.position, self.ship.position + DVec3::new(0.0, 1.0, 0.0), &self.camera, 0x00ff00ff);
        draw_line_3d(frame, depth, self.ship.position, self.ship.position + DVec3::new(0.0, 0.0, 1.0), &self.camera, 0x0000ffff);

        self.draw_hud(frame, depth, dt);
    }

    pub fn draw_hud(&self, frame: &mut [u8], depth: &mut [f64], dt: f64) {
        for (thrust, t) in self.ship.thrust {
            let (x0, y0, x1, y1, key) = match thrust {
                Thrust::Left => (0, 7, 6, 13, "A"),
                Thrust::Right => (14, 7, 20, 13, "D"),
                Thrust::Up => (21, 14, 27, 20, "R"),
                Thrust::Down => (21, 7, 27, 13, "F"),
                Thrust::Front => (7, 14, 13, 20, "W"),
                Thrust::Back => (7, 7, 13, 13, "S"),
                Thrust::YawLeft => (35, 7, 41, 13, "J"),
                Thrust::YawRight => (49, 7, 55, 13, "L"),
                Thrust::PitchUp => (42, 7, 48, 13, "K"),
                Thrust::PitchDown => (42, 14, 48, 20, "I"),
                Thrust::RollCCW => (35, 14, 41, 20, "U"),
                Thrust::RollCW => (49, 14, 55, 20, "O"),
            };
            let bg: u32 = if t > 0.01 {0xffffffff} else {0x00000000};
            let fg: u32 = if t > 0.01 {0x00000000} else {0xffffffff};
            draw_rectangle_fill(frame, depth, DVec3::new(x0 as f64, y0 as f64, 0.0), DVec3::new(x1 as f64, y1 as f64, 0.0), bg);
            draw_text(frame, depth, DVec3::new((x0 + 1) as f64, (y0 + 1) as f64, 0.0), key, &FONT_5PX, 6, 1, fg);
        }
        draw_rectangle_fill(frame, depth, DVec3::new(21.0, 0.0, 0.0), DVec3::new(55.0, 6.0, 0.0), if self.ship.brake {0xffffffff} else {0x000000ff});
        draw_text(frame, depth, DVec3::new(22.0, 1.0, 0.0), "SPACE", &FONT_5PX, 7, 1, if self.ship.brake {0x000000ff} else {0xffffffff});

        draw_rectangle_fill(frame, depth, DVec3::new(0.0, 21.0, 0.0), DVec3::new(20.0, 27.0, 0.0), if self.ship.boost > 0.0 {0xffffffff} else {0x000000ff});
        draw_text(frame, depth, DVec3::new(1.0, 22.0, 0.0), "TAB", &FONT_5PX, 7, 1, if self.ship.boost > 0.0 {0x000000ff} else {0xffffffff});

        draw_rectangle_fill(frame, depth, DVec3::new(0.0, 0.0, 0.0), DVec3::new(20.0, 6.0, 0.0), if self.ship.jumping || self.ship.charging_jump {0xffffffff} else {0x000000ff});
        draw_text(frame, depth, DVec3::new(1.0, 1.0, 0.0), "ALT", &FONT_5PX, 7, 1, if self.ship.jumping|| self.ship.charging_jump {0x000000ff} else {0xffffffff});
    
        draw_text(frame, depth, DVec3::new(1.0, (HEIGHT - 6) as f64, 0.0), &(f64::round(dt * 1000.0) / 1000.0).to_string(), &FONT_5PX, 6, 1, 0xffffffff);

        let velocity = format!("{:.3} m/s  ", f64::round(self.ship.velocity.length() * 1000.0) / 1000.0);
        let acceleration = format!("{:.3} m/s^2", f64::round(self.ship.acceleration.length() * 1000.0) / 1000.0);
        draw_text(frame, depth, DVec3::new(WIDTH as f64 - (velocity.len() * 6) as f64, 8.0, 0.0), &velocity, &FONT_5PX, 6, 1, 0xffffffff);
        draw_text(frame, depth, DVec3::new(WIDTH as f64 - (acceleration.len() * 6) as f64, 1.0, 0.0), &acceleration, &FONT_5PX, 6, 1, 0xffffffff);

        let boost_cooldown = format!("{:.2}", f64::round(self.ship.boost_cooldown * 100.0) / 100.0);
        draw_rectangle_fill(frame, depth, DVec3::new(28.0, 21.0, 0.0), DVec3::new(55.0, 27.0, 0.0), if self.ship.boost_cooldown > 0.0 {0xffffffff} else {0x000000ff});
        draw_text(frame, depth, DVec3::new(57.0 as f64 - (boost_cooldown.len() * 7) as f64, 22.0, 0.0), &boost_cooldown, &FONT_5PX, 7, 1, if self.ship.boost_cooldown > 0.0 {0x000000ff} else {0xffffffff});

        if self.ship.charging_jump {
            let jump_charge = if self.ship.jump_charge <= 1.0 {
                format!(">>> {:.2} <<<", f64::round(self.ship.jump_charge * 100.0) / 100.0)
            } else if self.ship.jump_charge <= 2.0 {
                format!(">>  {:.2}  <<", f64::round(self.ship.jump_charge * 100.0) / 100.0)
            } else if self.ship.jump_charge <= 3.0 {
                format!(">   {:.2}   <", f64::round(self.ship.jump_charge * 100.0) / 100.0)
            } else {
                format!("    {:.2}    ", f64::round(self.ship.jump_charge * 100.0) / 100.0)
            };
            draw_text(frame, depth, DVec3::new(WIDTH as f64 / 2.0 + 48.0 + 96.0 - (jump_charge.len() * 6*4) as f64, HEIGHT as f64 - 48.0, 0.0), &jump_charge, &FONT_5PX, 6, 4, 0xffffffff);
        }
    }
}

pub fn update_ship_movement(ship: &mut Ship, dt: f64) {
    if ship.charging_jump {
        ship.brake = true;
        ship.jump_charge = f64::max(0.0, ship.jump_charge - dt);
        ship.thrust[Thrust::Front] = 2.0 * ship.stats.thrust;
        if ship.jump_charge == 0.0 {
            ship.charging_jump = false;
            ship.jumping = true;
            start_jump(ship, dt);
        }
    }

    if ship.brake {
        let angular_brake_thrust = DVec3::from(ship.angular_velocity.inverse().to_euler(glam::EulerRot::XYZ)) * 200.0;
        if ship.thrust[Thrust::PitchUp] == 0.0 && ship.thrust[Thrust::PitchDown] == 0.0 {ship.thrust[Thrust::PitchUp] = f64::clamp(angular_brake_thrust.x, 0.0, ship.stats.angular_thrust);}
        if ship.thrust[Thrust::PitchUp] == 0.0 && ship.thrust[Thrust::PitchDown] == 0.0 {ship.thrust[Thrust::PitchDown] = f64::clamp(-angular_brake_thrust.x, 0.0, ship.stats.angular_thrust);}
        if ship.thrust[Thrust::YawLeft] == 0.0 && ship.thrust[Thrust::YawRight] == 0.0 {ship.thrust[Thrust::YawLeft] = f64::clamp(angular_brake_thrust.y, 0.0, ship.stats.angular_thrust);}
        if ship.thrust[Thrust::YawLeft] == 0.0 && ship.thrust[Thrust::YawRight] == 0.0 {ship.thrust[Thrust::YawRight] = f64::clamp(-angular_brake_thrust.y, 0.0, ship.stats.angular_thrust);}
        if ship.thrust[Thrust::RollCCW] == 0.0 && ship.thrust[Thrust::RollCW] == 0.0 {ship.thrust[Thrust::RollCCW] = f64::clamp(angular_brake_thrust.z, 0.0, ship.stats.angular_thrust);}
        if ship.thrust[Thrust::RollCCW] == 0.0 && ship.thrust[Thrust::RollCW] == 0.0 {ship.thrust[Thrust::RollCW] = f64::clamp(-angular_brake_thrust.z, 0.0, ship.stats.angular_thrust);}

        let brake_thrust = ship.rotation.inverse() * ship.velocity * 10.0;
        if ship.thrust[Thrust::Right] == 0.0 && ship.thrust[Thrust::Left] == 0.0 {ship.thrust[Thrust::Right] = f64::clamp(-brake_thrust.x, 0.0, ship.stats.thrust);}
        if ship.thrust[Thrust::Right] == 0.0 && ship.thrust[Thrust::Left] == 0.0 {ship.thrust[Thrust::Left] = f64::clamp(brake_thrust.x, 0.0, ship.stats.thrust);}
        if ship.thrust[Thrust::Up] == 0.0 && ship.thrust[Thrust::Down] == 0.0 {ship.thrust[Thrust::Up] = f64::clamp(-brake_thrust.y, 0.0, ship.stats.thrust);}
        if ship.thrust[Thrust::Up] == 0.0 && ship.thrust[Thrust::Down] == 0.0 {ship.thrust[Thrust::Down] = f64::clamp(brake_thrust.y, 0.0, ship.stats.thrust);}
        if ship.thrust[Thrust::Back] == 0.0 && ship.thrust[Thrust::Front] == 0.0 {ship.thrust[Thrust::Back] = f64::clamp(-brake_thrust.z, 0.0, ship.stats.thrust);}
        if ship.thrust[Thrust::Back] == 0.0 && ship.thrust[Thrust::Front] == 0.0 {ship.thrust[Thrust::Front] = f64::clamp(brake_thrust.z, 0.0, 2.0 * ship.stats.thrust);}
    }

    if ship.jumping {
        ship.thrust = enum_map! {
            _ => 0.0,
        };
    }

    ship.angular_acceleration = DQuat::from_euler(
        glam::EulerRot::XYZ,
        (ship.thrust[Thrust::PitchUp] - ship.thrust[Thrust::PitchDown]) * dt*dt, // todo: apply delta properly
        (ship.thrust[Thrust::YawLeft] - ship.thrust[Thrust::YawRight]) * dt*dt,
        (ship.thrust[Thrust::RollCCW] - ship.thrust[Thrust::RollCW]) * dt*dt,
    );
    ship.angular_velocity *= ship.angular_acceleration;
    ship.rotation *= ship.angular_velocity;

    ship.thrust[Thrust::Front] += ship.boost;
    ship.boost = f64::max(0.0, ship.boost - ship.stats.boost_strength / ship.stats.boost_duration * dt);
    ship.boost_cooldown = f64::max(0.0, ship.boost_cooldown - dt);
    
    ship.acceleration = ship.rotation * DVec3::new(
        ship.thrust[Thrust::Right] - ship.thrust[Thrust::Left],
        ship.thrust[Thrust::Up] - ship.thrust[Thrust::Down],
        ship.thrust[Thrust::Back] - ship.thrust[Thrust::Front],
    );
    ship.velocity += ship.acceleration * dt;
    ship.position += ship.velocity * dt;

    ship.hull.model = DMat4::from_rotation_translation(ship.rotation, ship.position);
}

pub fn update_camera_position(camera: &mut Camera, ship: &Ship) {
    let position_offset = DVec3::new(0.0, 4.0, 10.0);
    let rotation_offset = DVec3::new(0.0, 0.0, 0.0);
    let trailing_factor = 0.85;

    camera.position = camera.position * trailing_factor + (ship.position + ship.rotation * position_offset) * (1.0 - trailing_factor);
    camera.rotation = DQuat::look_at_rh(camera.position, ship.position + ship.rotation * rotation_offset, ship.rotation * DVec3::new(0.0, 1.0, 0.0)).inverse();
    camera.model = DMat4::from_rotation_translation(camera.rotation, camera.position);
    camera.view = camera.model.inverse();
}

pub fn start_jump(ship: &mut Ship, dt: f64) {
    ship.thrust = enum_map! {
        _ => 0.0,
    };
    ship.angular_velocity = DQuat::from_euler(glam::EulerRot::XYZ, 0.0, 0.0, 1.0 * dt);
    ship.velocity = ship.rotation * DVec3::new(0.0, 0.0, -ship.stats.jump_speed);
}

pub fn end_jump(ship: &mut Ship) {
    ship.angular_velocity = DQuat::from_euler(glam::EulerRot::XYZ, 0.0, 0.0, 0.0);
    ship.velocity = ship.rotation * DVec3::new(0.0, 0.0, -100.0);
}

pub fn generate_stars() -> Vec<Object> {
    let count = 1000;

    let mut stars = Vec::new();
    for _ in 0..count {
        let pos = DVec3::new(
            rand::rng().sample::<f64, StandardNormal>(StandardNormal), 
            rand::rng().sample::<f64, StandardNormal>(StandardNormal), 
            rand::rng().sample::<f64, StandardNormal>(StandardNormal),
        ).normalize() * FAR / 2.0;
        let b = (rand::rng().random::<f64>() * 255.0) as u32 & 0xff;
        let col = (b << 24) | (b << 16) | (b << 8) | 0xff;
        stars.push(Object {
            mesh: Rc::new(vec![vec![pos]]),
            model: DMat4::IDENTITY,
            color: col,
            fill: 0x00000000,
        });
    }
    stars
}

pub fn update_dust(dust: &mut Vec<Object>, center: DVec3, first: bool) {
    let count: usize = 200;
    let (min_dist, max_dist): (f64, f64) = (90.0, 100.0);

    dust.retain(|d| (d.model.transform_point3(DVec3::ZERO) - center).length() <= max_dist);
    while dust.len() < count {
        let offset = DVec3::new(
            rand::rng().sample::<f64, StandardNormal>(StandardNormal), 
            rand::rng().sample::<f64, StandardNormal>(StandardNormal), 
            rand::rng().sample::<f64, StandardNormal>(StandardNormal),
        ).normalize() * rand::rng().random_range(if first {0.0} else {min_dist.powf(3.0)}..=max_dist.powf(3.0)).powf(1.0/3.0);
        dust.push(Object {
            mesh: Rc::new(vec![vec![DVec3::ZERO]]),
            model: DMat4::from_translation(center + offset),
            color: 0xffffffff,
            fill: 0x00000000,
        });
    }
    for d in dust {
        let brightness = f64::max(0.0, 1.0 - (d.model.transform_point3(DVec3::ZERO) - center).length() / min_dist);
        d.color = float_to_color((brightness, brightness, brightness, 1.0));
    }
}

pub fn generate_dust() -> Vec<Object> {
    let mut dust = Vec::new();
    update_dust(&mut dust, DVec3::ZERO, true);
    dust
}

pub fn create_thrusters() -> EnumMap<Thrust, Object> {
    let color = 0xff00ffff;

    let thrusters = enum_map! {
        Thrust::Front => Object {
            mesh: Rc::new(front_thruster_mesh()),
            model: DMat4::IDENTITY,
            color: color,
            fill: 0x000000ff,
        },
        _ => Object {
            mesh: Rc::new(vec![]),
            model: DMat4::IDENTITY,
            color: color,
            fill: 0x00000000,
        },
    };
    thrusters
}

pub fn add_exhaust_particles(particles: &mut Vec<Particle>, ship: &Ship, dt: f64) {
    let acceleration_factor = f64::clamp(ship.acceleration.length() / 100.0, 0.0, 1.0);
    let velocity_factor = f64::clamp(ship.velocity.length() / 200.0, 0.1, 1.0);
    let particle_strength = if !ship.jumping {(acceleration_factor * 2.0 + velocity_factor) / 3.0} else {1.0};

    let thruster_positions = vec![
        DVec3::new(-2.3, 0.0, 3.0),
        DVec3::new(2.3, 0.0, 3.0)
    ];

    for _ in 0..5 {
        let particle_offset = DVec3::new(
            rand::rng().sample::<f64, StandardNormal>(StandardNormal), 
            rand::rng().sample::<f64, StandardNormal>(StandardNormal), 
            rand::rng().sample::<f64, StandardNormal>(StandardNormal),
        ).normalize() * 0.5;

        for pos in &thruster_positions {
            if rand::random::<f64>() < particle_strength {
                let translation = ship.position + particle_offset + ship.rotation * *pos - ship.velocity * dt * rand::random::<f64>();
                particles.push(Particle {
                    object: Object {
                        mesh: Rc::new(vec![vec![DVec3::ZERO]]),
                        model: DMat4::from_translation(translation),
                        color: 0xff00ffff,
                        fill: 0x00000000,
                    },
                    lifetime: 10.0, 
                });
            }
        }
    }
}

pub fn generate_asteroids() -> Vec<Asteroid> {
    let count = 20000;
    let (min_dist, max_dist): (f64, f64) = (60000.0, 120000.0);
    let (min_scale, max_scale): (f64, f64) = (1.0, 100.0);
    let ring_plane_rotation = DMat4::from_axis_angle(DVec3::new(rand::random::<f64>(), rand::random::<f64>(), rand::random::<f64>()).normalize(), rand::random::<f64>() * PI);
    let mesh = Rc::new(parse_obj(ASTEROID_OBJ));
    let center = ring_plane_rotation.transform_point3(DVec3::new(0.0, 0.0, 100000.0));

    let mut asteroids = Vec::with_capacity(count);
    let planet_scale = 20000.0;
    asteroids.push(Asteroid {
        object: Object {
            mesh: mesh.clone(),
            model: DMat4::from_translation(center) * DMat4::from_scale(DVec3::ONE * planet_scale),
            color: 0xffffffff,
            fill: 0x000000ff,
        },
        rotation_axis: ring_plane_rotation.transform_point3(DVec3::new(0.0, 1.0, 0.0)),
        rotation_speed: 0.1,
    });
    for _ in 0..count {
        let offset = ring_plane_rotation.transform_point3((DVec3::new(
            rand::rng().sample::<f64, StandardNormal>(StandardNormal), 
            rand::rng().sample::<f64, StandardNormal>(StandardNormal), 
            rand::rng().sample::<f64, StandardNormal>(StandardNormal),
        ) * DVec3::new(1.0, 0.01, 1.0)).normalize() * rand::rng().random_range(min_dist.powf(2.0)..max_dist.powf(2.0)).powf(1.0/2.0));
        let scale = rand::random_range(min_scale..max_scale);

        asteroids.push(Asteroid {
            object: Object {
                mesh: mesh.clone(),
                model: DMat4::from_translation(center + offset) * DMat4::from_scale(DVec3::ONE * scale),
                color: 0xffffffff,
                fill: 0x000000ff,
            },
            rotation_axis: DVec3::new(rand::random_range(-1.0..1.0), rand::random_range(-1.0..1.0), rand::random_range(-1.0..1.0)).normalize(),
            rotation_speed: rand::random_range(-1.0..1.0),
        });
    }
    asteroids
}
