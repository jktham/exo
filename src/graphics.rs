use core::f32;
use std::cmp::{max, min};
use std::collections::{HashMap, HashSet};
use std::hash::RandomState;
use std::mem::swap;
use glam::Vec3;

use crate::game::{Camera, Object};
use crate::transform::{behind_camera, out_of_bounds, transform_mesh, transform_world_to_screen};
use crate::{WIDTH, HEIGHT};

pub fn clear(frame: &mut [u8], depth: &mut [f32], color: u32) {
    for x in 0..WIDTH as i32 {
        for y in 0..HEIGHT as i32 {
            let i = (((HEIGHT as i32 - 1 - y) * WIDTH as i32 + x) * 4) as usize;
            depth[i/4] = 10000.0;
            frame[i] = (color >> 24) as u8;
            frame[i+1] = (color >> 16) as u8;
            frame[i+2] = (color >> 8) as u8;
            frame[i+3] = (color) as u8;
        }
    }
}

pub fn draw_pixel(frame: &mut [u8], depth: &mut [f32], p: Vec3, color: u32) {
    if out_of_bounds(p, 0) { return; };
    let (x, y) = (p.x as i32, p.y as i32);
    let i = (((HEIGHT as i32 - 1 - y) * WIDTH as i32 + x) * 4) as usize;
    if p.z <= depth[i/4] {
        // color = float_to_color(f32::clamp((p.z - 0.975) * 40.0, 0.0, 1.0), 0.0, 0.0, 1.0);
        depth[i/4] = p.z;
        frame[i] = (color >> 24) as u8;
        frame[i+1] = (color >> 16) as u8;
        frame[i+2] = (color >> 8) as u8;
        frame[i+3] = (color) as u8;
    }
}

pub fn bresenham(p0: Vec3, p1: Vec3) -> Vec<Vec3> {
    if out_of_bounds(p0, 200) || out_of_bounds(p1, 200) { return vec![]; };
    let (mut x0, mut y0, mut z0) = (p0.x as i32, p0.y as i32, p0.z);
    let (mut x1, mut y1, mut z1) = (p1.x as i32, p1.y as i32, p1.z);
    let mut line = vec![];

    if i32::abs(y1 - y0) < i32::abs(x1 - x0) {
        if x0 > x1 {
            swap(&mut x0, &mut x1);
            swap(&mut y0, &mut y1);
            swap(&mut z0, &mut z1);
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
            line.push(Vec3::new(x as f32, y as f32, 0.0));
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
            swap(&mut z0, &mut z1);
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
            line.push(Vec3::new(x as f32, y as f32, 0.0));
            if d > 0 {
                x += xi;
                d += 2*(dx-dy);
            } else {
                d += 2*dx;
            }
        }
    }
    for i in 0..line.len() {
        let di = i as f32 / (line.len()-1) as f32;
        let z = z0 * (1.0 - di) + z1 * di;
        line[i].z = z;
    }
    line
}

pub fn map_lines(lines: &Vec<Vec3>) -> HashMap::<i32, Vec<&Vec3>> {
    let mut map_y = HashMap::<i32, Vec<&Vec3>>::new();
    for p in lines {
        if map_y.contains_key(&(p.y as i32)) {
            map_y.get_mut(&(p.y as i32)).unwrap().push(p);
        } else {
            map_y.insert(p.y as i32, vec![p]);
        }
    };
    for (_y, v) in &mut map_y {
        v.sort_by(|a, b| (a.x as i32).cmp(&(b.x as i32)));
    }
    map_y
}

pub fn draw_line(frame: &mut [u8], depth: &mut [f32], p0: Vec3, p1: Vec3, color: u32) {
    if out_of_bounds(p0, 200) || out_of_bounds(p1, 200) { return; };

    let line = bresenham(p0, p1);
    for p in line {
        draw_pixel(frame, depth, p, color);
    }
}

pub fn draw_triangle_fill_outline(frame: &mut [u8], depth: &mut [f32], p0: Vec3, p1: Vec3, p2: Vec3, outline_lines: &Vec<Vec3>, color: u32, fill: u32) {
    if out_of_bounds(p0, 200) || out_of_bounds(p1, 200) || out_of_bounds(p2, 200) { return; };
    
    let mut lines = vec![];
    lines.append(&mut bresenham(p0, p1));
    lines.append(&mut bresenham(p1, p2));
    lines.append(&mut bresenham(p2, p0));

    let map_y = map_lines(&lines);
    let outline_map_y = map_lines(outline_lines);

    for (y, v) in map_y {
        let min = v.first().unwrap_or(&&Vec3::ZERO);
        let max = v.last().unwrap_or(&&Vec3::ZERO);
        let outline_x: HashSet<i32, RandomState> = HashSet::from_iter(outline_map_y.get(&y).unwrap_or(&Vec::new()).iter().map(|p| p.x as i32).collect::<Vec<i32>>());

        for x in min.x as i32 ..= max.x as i32 {
            let dx = (x as f32 - min.x) / f32::max(max.x - min.x, 1.0);
            let z = min.z * (1.0 - dx) + max.z * dx;
            // println!("({x}, {dx}, {z})");
            if outline_x.contains(&x) {
                draw_pixel(frame, depth, Vec3::new(x as f32, y as f32, z), color);
            } else {
                draw_pixel(frame, depth, Vec3::new(x as f32, y as f32, z), fill);
            }
        }
    }
}

pub fn draw_rectangle(frame: &mut [u8], depth: &mut [f32], p0: Vec3, p1: Vec3, color: u32) {
    let (mut x0, mut y0) = (p0.x as i32, p0.y as i32);
    let (mut x1, mut y1) = (p1.x as i32, p1.y as i32);
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
                draw_pixel(frame, depth, Vec3::new(x as f32, y as f32, p0.z), color);
            }
        }
    }
}

pub fn draw_rectangle_fill(frame: &mut [u8], depth: &mut [f32], p0: Vec3, p1: Vec3, color: u32) {
    let (mut x0, mut y0) = (p0.x as i32, p0.y as i32);
    let (mut x1, mut y1) = (p1.x as i32, p1.y as i32);
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
            draw_pixel(frame, depth, Vec3::new(x as f32, y as f32, p0.z), color);
        }
    }
}

pub fn draw_sprite(frame: &mut [u8], depth: &mut [f32], p: Vec3, sprite: &[&[u8]], scale: i32, color: u32) {
    if scale < 0 { // todo
        return;
    }
    for i in 0..sprite.len() {
        for j in 0..sprite[i].len() {
            if sprite[sprite.len() - 1 - i][j] == 1 {
                for di in 0..scale {
                    for dj in 0..scale {
                        draw_pixel(frame, depth, Vec3::new((p.x as i32 + scale * j as i32 + dj) as f32, (p.y as i32 + scale * i as i32 + di) as f32, p.z), color);
                    }
                }
            }
        }
    }
}

pub fn draw_text(frame: &mut [u8], depth: &mut [f32], p: Vec3, text: &str, font: &[&[&[u8]]], offset: i32, scale: i32, color: u32) {
    let mut dx = 0;
    let mut dy = 0;
    for c in text.as_bytes() {
        if (*c as usize) < font.len() {
            draw_sprite(frame, depth, Vec3::new(p.x + dx as f32, p.y + dy as f32, p.z), &font[*c as usize], scale, color);
            dx += offset * scale;
            if *c == 10 { // LF
                dx = 0;
                dy -= offset * scale;
            }
        }
    }
}

pub fn draw_point_3d(frame: &mut [u8], depth: &mut [f32], v: Vec3, camera: &Camera, color: u32) {
    let p = transform_world_to_screen(v, camera);
    if behind_camera(p) { return; };
    draw_pixel(frame, depth, p, color);
}

pub fn draw_line_3d(frame: &mut [u8], depth: &mut [f32], v0: Vec3, v1: Vec3, camera: &Camera, color: u32) {
    let p0 = transform_world_to_screen(v0, camera);
    let p1 = transform_world_to_screen(v1, camera);
    if behind_camera(p0) || behind_camera(p1) { return; };
    draw_line(frame, depth, p0, p1, color);
}

pub fn draw_triangle_fill_outline_3d(frame: &mut [u8], depth: &mut [f32], v0: Vec3, v1: Vec3, v2: Vec3, outline_lines: &Vec<Vec3>, camera: &Camera, color: u32, fill: u32) {
    let p0 = transform_world_to_screen(v0, camera);
    let p1 = transform_world_to_screen(v1, camera);
    let p2 = transform_world_to_screen(v2, camera);
    if behind_camera(p0) || behind_camera(p1) || behind_camera(p2) { return; };
    draw_triangle_fill_outline(frame, depth, p0, p1, p2, &outline_lines, color, fill);
}

pub fn draw_polygon_3d(frame: &mut [u8], depth: &mut [f32], polygon: &Vec<Vec3>, camera: &Camera, color: u32, fill: u32) {
    if polygon.len() == 1 {
        draw_point_3d(frame, depth, polygon[0], camera, color);
    } else if polygon.len() == 2 {
        draw_line_3d(frame, depth, polygon[0], polygon[1], camera, color);
    } else if !polygon.is_empty() {
        if fill != 0x00000000 {
            let outline_points: Vec<Vec3> = polygon.iter().map(|v| transform_world_to_screen(*v, camera)).collect();
            let mut outline_lines = vec![];
            for i in 0..outline_points.len() {
                outline_lines.append(&mut bresenham(outline_points[i], outline_points[(i+1) % outline_points.len()]));
            }
            for i in 2..polygon.len() {
                let v0 = polygon[0];
                let v1 = polygon[i-1];
                let v2 = polygon[i];
                draw_triangle_fill_outline_3d(frame, depth, v0, v1, v2, &outline_lines, camera, color, fill);
            }
        } else {
            for i in 0..polygon.len() {
                draw_line_3d(frame, depth, polygon[i], polygon[(i+1) % polygon.len()], camera, color);
            }
        }
    }
}

pub fn draw_mesh_3d(frame: &mut [u8], depth: &mut [f32], mesh: &Vec<Vec<Vec3>>, camera: &Camera, color: u32, fill: u32) {
    for polygon in mesh {
        draw_polygon_3d(frame, depth, polygon, camera, color, fill);
    }
}

pub fn draw_object(frame: &mut [u8], depth: &mut [f32], object: &Object, camera: &Camera) {
    draw_mesh_3d(frame, depth, &transform_mesh(&object.mesh, object.model), camera, object.color, object.fill);
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