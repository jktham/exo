use std::cmp::{max, min};
use std::mem::swap;
use glam::{Mat4, Vec3, Vec4};

use crate::game::Camera;
use crate::{WIDTH, HEIGHT};

// world to screen
pub fn transform(pos: Vec3, camera: &Camera) -> Vec3 {
	let w = WIDTH as f32;
	let h = HEIGHT as f32;
	let n = 0.1;
	let f = 1000.0;
	let phi = camera.fov / 180.0 * 3.1415;
	let r = f32::tan(phi/2.0) * n;
	let t = r * h/w;

	let view = Mat4::look_at_lh(camera.position, camera.position + camera.direction, Vec3::new(0.0, 1.0, 0.0));
	let projection = Mat4::from_cols_array(&[
		n/r, 0.0, 0.0, 0.0,
		0.0, n/t, 0.0, 0.0,
		0.0, 0.0, -(f+n)/(f-n), -2.0*f*n/(f-n),
		0.0, 0.0, -1.0, 0.0,
	]).transpose();

	let world = Vec4::new(pos.x, pos.y, pos.z, 1.0);
	let eye = view * world;
	let clip = projection * eye;
	let ndc = Vec3::new(clip.x/clip.w, clip.y/clip.w, clip.z/clip.w);
	let mut screen = Vec3::new(w/2.0 * ndc.x + w/2.0, h/2.0 * ndc.y + h/2.0, (f-n)/2.0 * ndc.z + (f+n)/2.0);
	screen.z /= f;
	// println!("world: {}", world);
	// println!("eye: {}", eye);
	// println!("clip: {}", clip);
	// println!("ndc: {}", ndc);
	// println!("screen: {}", screen);

	return screen;
}

pub fn clear(frame: &mut [u8], col: u32) {
	for x in 0..WIDTH as i32 {
		for y in 0..HEIGHT as i32 {
			draw_pixel(frame, x, y, col);
		}
	}
}

pub fn draw_rectangle_fill(frame: &mut [u8], mut x0: i32, mut y0: i32, mut x1: i32, mut y1: i32, col: u32) {
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
			draw_pixel(frame, x, y, col);
		}
	}
}

pub fn draw_rectangle(frame: &mut [u8], mut x0: i32, mut y0: i32, mut x1: i32, mut y1: i32, col: u32) {
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
				draw_pixel(frame, x, y, col);
			}
		}
	}
}

pub fn draw_line(frame: &mut [u8], mut x0: i32, mut y0: i32, mut x1: i32, mut y1: i32, col: u32) {
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
			draw_pixel(frame, x, y, col);
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
			draw_pixel(frame, x, y, col);
			if d > 0 {
				x += xi;
				d += 2*(dx-dy);
			} else {
				d += 2*dx;
			}
		}
	}
}

pub fn draw_pixel(frame: &mut [u8], x: i32, y: i32, col: u32) {
	if x < 0 || x >= WIDTH as i32 || y < 0 || y >= HEIGHT as i32 {
		return;
	}
	let i = ((y * WIDTH as i32 + x) * 4) as usize;
	frame[i] = (col >> 24) as u8;
	frame[i+1] = (col >> 16) as u8;
	frame[i+2] = (col >> 8) as u8;
	frame[i+3] = (col) as u8;
}