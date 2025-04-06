use core::f32;
use std::cmp::{max, min};
use std::collections::HashMap;
use std::mem::swap;
use glam::Vec3;

use crate::game::{Camera, Object};
use crate::transform::{transform_mesh, transform_world_to_screen};
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

pub fn bresenham(mut x0: i32, mut y0: i32, mut x1: i32, mut y1: i32) -> Vec<(i32, i32)> {
    let mut line = vec![];
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
            line.push((x, y));
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
            line.push((x, y));
            if d > 0 {
                x += xi;
                d += 2*(dx-dy);
            } else {
                d += 2*dx;
            }
        }
    }
    line
}

pub fn draw_line(frame: &mut [u8], x0: i32, y0: i32, x1: i32, y1: i32, color: u32) {
    if x0 < 0 || x0 >= WIDTH as i32 || x1 < 0 || x1 >= WIDTH as i32 || y0 < 0 || y0 >= HEIGHT as i32 || y1 < 0 || y1 >= HEIGHT as i32 {
        return;
    }
    let line = bresenham(x0, y0, x1, y1);
    for p in line {
        draw_pixel(frame, p.0, p.1, color);
    }
}

pub fn draw_triangle_fill(frame: &mut [u8], x0: i32, y0: i32, x1: i32, y1: i32, x2: i32, y2: i32, fill: u32) {
    if x0 < 0 || x0 >= WIDTH as i32 || x1 < 0 || x1 >= WIDTH as i32 || x2 < 0 || x2 >= WIDTH as i32 || y0 < 0 || y0 >= HEIGHT as i32 || y1 < 0 || y1 >= HEIGHT as i32 || y2 < 0 || y2 >= HEIGHT as i32 {
        return;
    }
    let mut lines = vec![];
    lines.append(&mut bresenham(x0, y0, x1, y1));
    lines.append(&mut bresenham(x1, y1, x2, y2));
    lines.append(&mut bresenham(x2, y2, x0, y0));
    
    let mut map_y = HashMap::<i32, Vec<i32>>::new();
    for p in lines {
        if map_y.contains_key(&p.1) {
            map_y.get_mut(&p.1).unwrap().push(p.0);
        } else {
            map_y.insert(p.1, vec![p.0]);
        }
    }
    for (y, v) in map_y {
        for x in *v.iter().min().unwrap()..=*v.iter().max().unwrap() {
            draw_pixel(frame, x, y, fill);
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

pub fn draw_point_3d(frame: &mut [u8], v: Vec3, camera: &Camera, color: u32) {
    let p = transform_world_to_screen(v, camera);
    if p.z > 1.0 {
        return;
    }
    draw_pixel(frame, p.x as i32, p.y as i32, color);
}

pub fn draw_line_3d(frame: &mut [u8], v0: Vec3, v1: Vec3, camera: &Camera, color: u32) {
    let p0 = transform_world_to_screen(v0, camera);
    let p1 = transform_world_to_screen(v1, camera);
    if p0.z > 1.0 || p1.z > 1.0 {
        return;
    }
    draw_line(frame, p0.x as i32, p0.y as i32, p1.x as i32, p1.y as i32, color);
}

pub fn draw_triangle_fill_3d(frame: &mut [u8], v0: Vec3, v1: Vec3, v2: Vec3, camera: &Camera, fill: u32) {
    let p0 = transform_world_to_screen(v0, camera);
    let p1 = transform_world_to_screen(v1, camera);
    let p2 = transform_world_to_screen(v2, camera);
    if p0.z > 1.0 || p1.z > 1.0 || p2.z > 1.0 {
        return;
    }
    draw_triangle_fill(frame, p0.x as i32, p0.y as i32, p1.x as i32, p1.y as i32, p2.x as i32, p2.y as i32, fill);
}

pub fn draw_polygon_3d(frame: &mut [u8], polygon: &Vec<Vec3>, camera: &Camera, color: u32, fill: u32) {
    if polygon.len() == 1 {
        draw_point_3d(frame, polygon[0], camera, color);
    } else if polygon.len() == 2 {
        draw_line_3d(frame, polygon[0], polygon[1], camera, color);
    } else if !polygon.is_empty() {
        if fill != 0x00000000 {
            for i in 2..polygon.len() {
                let v0 = polygon[0];
                let v1 = polygon[i-1];
                let v2 = polygon[i];
                draw_triangle_fill_3d(frame, v0, v1, v2, camera, fill);
            }
        }
        for i in 0..polygon.len() {
            draw_line_3d(frame, polygon[i], polygon[(i+1) % polygon.len()], camera, color);
        }
    }
}

pub fn draw_mesh_3d(frame: &mut [u8], mesh: &Vec<Vec<Vec3>>, camera: &Camera, color: u32, fill: u32) {
    for polygon in mesh {
        draw_polygon_3d(frame, polygon, camera, color, fill);
    }
}

pub fn draw_object(frame: &mut [u8], object: &Object, camera: &Camera) {
    draw_mesh_3d(frame, &transform_mesh(&object.mesh, object.model), camera, object.color, object.fill);
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