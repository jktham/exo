use core::f32;
use std::cmp::{max, min};
use std::mem::swap;
use glam::{Mat4, Vec3, Vec4};

use crate::game::{Camera, Object};
use crate::{WIDTH, HEIGHT};

pub fn clear(frame: &mut [u8], color: u32) {
	for x in 0..WIDTH as i32 {
		for y in 0..HEIGHT as i32 {
			draw_pixel(frame, x, y, color);
		}
	}
}

pub fn draw_pixel(frame: &mut [u8], x: i32, y: i32, color: u32) {
	if x < 0 || x >= WIDTH as i32 || y < 0 || y >= HEIGHT as i32 {
		return;
	}
	let i = (((HEIGHT as i32 - 1 - y) * WIDTH as i32 + x) * 4) as usize;
	frame[i] = (color >> 24) as u8;
	frame[i+1] = (color >> 16) as u8;
	frame[i+2] = (color >> 8) as u8;
	frame[i+3] = (color) as u8;
}

pub fn draw_line(frame: &mut [u8], mut x0: i32, mut y0: i32, mut x1: i32, mut y1: i32, color: u32) {
	if x0 < 0 || x0 >= WIDTH as i32 || x1 < 0 || x1 >= WIDTH as i32 || y0 < 0 || y0 >= HEIGHT as i32 || y1 < 0 || y1 >= HEIGHT as i32 {
		return;
	}
	if i32::abs(y1 - y0) < i32::abs(x1 - x0) {
		if x0 > x1 {
			swap(&mut x0, &mut x1);
			swap(&mut y0, &mut y1);
		}
		let dx = x1 - x0;
		let mut dy = y1 - y0;
		let mut yi = 1;
		if dy < 0 {
			yi = -1;
			dy = -dy;
		}
		let mut d = 2*dy - dx;
		let mut y = y0;
	
		for x in x0..=x1 {
			draw_pixel(frame, x, y, color);
			if d > 0 {
				y += yi;
				d += 2*(dy-dx);
			} else {
				d += 2*dy;
			}
		}
	} else {
		if y0 > y1 {
			swap(&mut x0, &mut x1);
			swap(&mut y0, &mut y1);
		}
		let mut dx = x1 - x0;
		let dy = y1 - y0;
		let mut xi = 1;
		if dx < 0 {
			xi = -1;
			dx = -dx;
		}
		let mut d = 2*dx - dy;
		let mut x = x0;
	
		for y in y0..=y1 {
			draw_pixel(frame, x, y, color);
			if d > 0 {
				x += xi;
				d += 2*(dx-dy);
			} else {
				d += 2*dx;
			}
		}
	}
}

pub fn draw_rectangle(frame: &mut [u8], mut x0: i32, mut y0: i32, mut x1: i32, mut y1: i32, color: u32) {
	if x1 < x0 {
		swap(&mut x0, &mut x1);
	}
	if y1 < y0 {
		swap(&mut y0, &mut y1);
	}

	x0 = max(x0, -1);
	x1 = min(x1, WIDTH as i32);
	y0 = max(y0, -1);
	y1 = min(y1, HEIGHT as i32);
	for x in x0..=x1 {
		for y in y0..=y1 {
			if x == x0 || x == x1 || y == y0 || y == y1 {
				draw_pixel(frame, x, y, color);
			}
		}
	}
}

pub fn draw_rectangle_fill(frame: &mut [u8], mut x0: i32, mut y0: i32, mut x1: i32, mut y1: i32, color: u32) {
	if x1 < x0 {
		swap(&mut x0, &mut x1);
	}
	if y1 < y0 {
		swap(&mut y0, &mut y1);
	}

	x0 = max(x0, -1);
	x1 = min(x1, WIDTH as i32);
	y0 = max(y0, -1);
	y1 = min(y1, HEIGHT as i32);
	for x in x0..=x1 {
		for y in y0..=y1 {
			draw_pixel(frame, x, y, color);
		}
	}
}

pub fn draw_sprite(frame: &mut [u8], x: i32, y: i32, sprite: &[&[u8]], scale: i32, color: u32) {
	if scale < 0 { // todo
		return;
	}
	for i in 0..sprite.len() {
		for j in 0..sprite[i].len() {
			if sprite[sprite.len() - 1 - i][j] == 1 {
				for di in 0..scale {
					for dj in 0..scale {
						draw_pixel(frame, x + scale * j as i32 + dj, y + scale * i as i32 + di, color);
					}
				}
			}
		}
	}
}

pub fn draw_text(frame: &mut [u8], x: i32, y: i32, text: &str, font: &[&[&[u8]]], offset: i32, scale: i32, color: u32) {
	let mut dx = 0;
	let mut dy = 0;
	for c in text.as_bytes() {
		if (*c as usize) < font.len() {
			draw_sprite(frame, x+dx, y+dy, &font[*c as usize], scale, color);
			dx += offset * scale;
			if *c == 10 { // LF
				dx = 0;
				dy -= offset * scale;
			}
		}
	}
}

pub fn transform(vertex: Vec3, model: Mat4, camera: &Camera) -> Vec3 {
	let w = WIDTH as f32;
	let h = HEIGHT as f32;
	let n = 0.1;
	let f = 10000.0;
	let phi = camera.fov / 180.0 * f32::consts::PI;
	let r = f32::tan(phi/2.0) * n;
	let t = r * h/w;

	let projection = Mat4::from_cols_array(&[
		n/r, 0.0, 0.0, 0.0,
		0.0, n/t, 0.0, 0.0,
		0.0, 0.0, -(f+n)/(f-n), -2.0*f*n/(f-n),
		0.0, 0.0, -1.0, 0.0,
	]).transpose();

	let world = model * Vec4::new(vertex.x, vertex.y, vertex.z, 1.0);
	let eye = camera.view * world;
	let clip = projection * eye;
	let ndc = Vec3::new(clip.x/clip.w, clip.y/clip.w, clip.z/clip.w);
	let mut screen = Vec3::new(
		w/2.0 * ndc.x + w/2.0, 
		h/2.0 * ndc.y + h/2.0, 
		(f-n)/2.0 * ndc.z + (f+n)/2.0
	);
	screen.z /= f;

	screen
}

pub fn draw_point_3d(frame: &mut [u8], v: Vec3, model: Mat4, camera: &Camera, color: u32) {
	let p = transform(v, model, camera);
	if p.z > 1.0 {
		return;
	}
	draw_pixel(frame, p.x as i32, p.y as i32, color);
}

pub fn draw_line_3d(frame: &mut [u8], v0: Vec3, v1: Vec3, model0: Mat4, model1: Mat4, camera: &Camera, color: u32) {
	let p0 = transform(v0, model0, camera);
	let p1 = transform(v1, model1, camera);
	if p0.z > 1.0 || p1.z > 1.0 {
		return;
	}
	draw_line(frame, p0.x as i32, p0.y as i32, p1.x as i32, p1.y as i32, color);
}

pub fn draw_polygon_3d(frame: &mut [u8], polygon: &[Vec3], model: Mat4, camera: &Camera, color: u32) {
	if polygon.len() == 1 {
		draw_point_3d(frame, polygon[0], model, camera, color);
	} else if polygon.len() == 2 {
		draw_line_3d(frame, polygon[0], polygon[1], model, model, camera, color);
	} else if !polygon.is_empty() {
		for i in 0..polygon.len() {
			draw_line_3d(frame, polygon[i], polygon[(i+1) % polygon.len()], model, model, camera, color);
		}
	}
}

pub fn draw_object(frame: &mut [u8], object: &Object, camera: &Camera) {
	for polygon in &object.mesh {
		draw_polygon_3d(frame, polygon, object.model, camera, object.color);
	}
}

pub fn color_to_float(color: u32) -> (f32, f32, f32, f32) {
	let r = ((color >> 24) as u8) as f32 / 255.0;
	let g = ((color >> 16) as u8) as f32 / 255.0;
	let b = ((color >> 8) as u8) as f32 / 255.0;
	let a = ((color) as u8) as f32 / 255.0;
	(r, g, b, a)
}

pub fn float_to_color(r: f32, g: f32, b: f32, a: f32) -> u32 {
	let mut color = 0x00000000;
	color |= ((r * 255.0) as u32) << 24;
	color |= ((g * 255.0) as u32) << 16;
	color |= ((b * 255.0) as u32) << 8;
	color |= (a * 255.0) as u32;
	color
}