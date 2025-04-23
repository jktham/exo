use core::f64;
use std::cmp::{max, min};
use std::collections::{HashMap, HashSet};
use fxhash::FxBuildHasher;
use std::mem::swap;
use glam::DVec3;

use crate::game::{Camera, Object};
use crate::transform::{out_of_bounds, transform_mesh, transform_world_to_screen, FAR};
use crate::{WIDTH, HEIGHT};

pub fn clear(frame: &mut [u8], depth: &mut [f64], color: u32) {
    for x in 0..WIDTH as i32 {
        for y in 0..HEIGHT as i32 {
            let i = (((HEIGHT as i32 - 1 - y) * WIDTH as i32 + x) * 4) as usize;
            depth[i/4] = FAR;
            frame[i] = (color >> 24) as u8;
            frame[i+1] = (color >> 16) as u8;
            frame[i+2] = (color >> 8) as u8;
            frame[i+3] = (color) as u8;
        }
    }
}

pub fn clear_depth(depth: &mut [f64]) {
    for x in 0..WIDTH as i32 {
        for y in 0..HEIGHT as i32 {
            let i = (((HEIGHT as i32 - 1 - y) * WIDTH as i32 + x) * 4) as usize;
            depth[i/4] = FAR;
        }
    }
}

pub fn clear_fade(frame: &mut [u8], f: f64) {
    for x in 0..WIDTH as i32 {
        for y in 0..HEIGHT as i32 {
            let i = (((HEIGHT as i32 - 1 - y) * WIDTH as i32 + x) * 4) as usize;

            let mut color = 0x00000000;
            color |= (frame[i] as u32) << 24;
            color |= (frame[i+1] as u32) << 16;
            color |= (frame[i+2] as u32) << 8;
            color |= frame[i+3] as u32;

            let c = color_to_float(color);
            color = float_to_color((c.0 * f, c.1 * f, c.2 * f, c.3));

            frame[i] = (color >> 24) as u8;
            frame[i+1] = (color >> 16) as u8;
            frame[i+2] = (color >> 8) as u8;
            frame[i+3] = (color) as u8;
        }
    }
}

pub fn draw_pixel(frame: &mut [u8], depth: &mut [f64], p: DVec3, color: u32) {
    if out_of_bounds(p, 0) { return; };
    let (x, y) = (p.x as i32, p.y as i32);
    let i = (((HEIGHT as i32 - 1 - y) * WIDTH as i32 + x) * 4) as usize;
    if p.z <= depth[i/4] {
        // color = float_to_color(f64::clamp((p.z - 0.975) * 40.0, 0.0, 1.0), 0.0, 0.0, 1.0);
        depth[i/4] = p.z;
        frame[i] = (color >> 24) as u8;
        frame[i+1] = (color >> 16) as u8;
        frame[i+2] = (color >> 8) as u8;
        frame[i+3] = (color) as u8;
    }
}

pub fn bresenham(p0: DVec3, p1: DVec3) -> Vec<DVec3> {
    if out_of_bounds(p0, 0) && out_of_bounds(p1, 0) { return vec![]; };
    let (mut x0, mut y0, mut z0) = (p0.x as i32, p0.y as i32, p0.z);
    let (mut x1, mut y1, mut z1) = (p1.x as i32, p1.y as i32, p1.z);

    let length = max(i32::abs(y1 - y0), i32::abs(x1 - x0)) + 1;
    if length > (WIDTH * 4) as i32 { return vec![]; };
    let mut line = Vec::with_capacity(length as usize);

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
            line.push(DVec3::new(x as f64, y as f64, 0.0));
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
            line.push(DVec3::new(x as f64, y as f64, 0.0));
            if d > 0 {
                x += xi;
                d += 2*(dx-dy);
            } else {
                d += 2*dx;
            }
        }
    }
    for i in 0..line.len() {
        let di = i as f64 / (line.len()-1) as f64;
        let z = z0 * (1.0 - di) + z1 * di;
        line[i].z = z;
    }
    line
}

pub fn map_lines(lines: &Vec<DVec3>) -> HashMap::<i32, (DVec3, DVec3), FxBuildHasher> {
    let mut map_y = HashMap::<i32, (DVec3, DVec3), FxBuildHasher>::default();
    for p in lines {
        if map_y.contains_key(&(p.y as i32)) {
            let e = map_y.get_mut(&(p.y as i32)).unwrap();
            if p.x < e.0.x {
                e.0 = *p;
            }
            if p.x > e.1.x {
                e.1 = *p;
            }
        } else {
            map_y.insert(p.y as i32, (*p, *p));
        }
    };
    map_y
}

pub fn map_outline(outline_lines: &Vec<DVec3>) -> HashMap::<i32, HashSet<i32, FxBuildHasher>, FxBuildHasher> {
    let mut outline_map_y = HashMap::<i32, HashSet<i32, FxBuildHasher>, FxBuildHasher>::default();
    for p in outline_lines {
        if outline_map_y.contains_key(&(p.y as i32)) {
            outline_map_y.get_mut(&(p.y as i32)).unwrap().insert(p.x as i32);
        } else {
            outline_map_y.insert(p.y as i32, HashSet::<i32, FxBuildHasher>::from_iter([p.x as i32]));
        }
    };
    outline_map_y
}

pub fn draw_line(frame: &mut [u8], depth: &mut [f64], p0: DVec3, p1: DVec3, color: u32) {
    if out_of_bounds(p0, 0) && out_of_bounds(p1, 0) { return; };

    let line = bresenham(p0, p1);
    for p in line {
        draw_pixel(frame, depth, p, color);
    }
}

pub fn draw_triangle_fill_outline(frame: &mut [u8], depth: &mut [f64], p0: DVec3, p1: DVec3, p2: DVec3, outline_lines: &Vec<DVec3>, color: u32, fill: u32) {
    if out_of_bounds(p0, 0) && out_of_bounds(p1, 0) && out_of_bounds(p2, 0) { return; };
    
    let mut lines = vec![];
    lines.append(&mut bresenham(p0, p1));
    lines.append(&mut bresenham(p1, p2));
    lines.append(&mut bresenham(p2, p0));

    let map_y = map_lines(&lines);
    let outline_map_y = map_outline(outline_lines);

    for (y, v) in map_y {
        let min = v.0;
        let max = v.1;
        let empty = HashSet::<i32, FxBuildHasher>::default();
        let outline_x = outline_map_y.get(&y).unwrap_or(&empty);

        for x in min.x as i32 ..= max.x as i32 {
            let dx = (x as f64 - min.x) / f64::max(max.x - min.x, 1.0);
            let z = min.z * (1.0 - dx) + max.z * dx;
            // println!("({x}, {dx}, {z})");
            if outline_x.contains(&x) {
                draw_pixel(frame, depth, DVec3::new(x as f64, y as f64, z), color);
            } else {
                draw_pixel(frame, depth, DVec3::new(x as f64, y as f64, z), fill);
            }
        }
    }
}

pub fn draw_rectangle(frame: &mut [u8], depth: &mut [f64], p0: DVec3, p1: DVec3, color: u32) {
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
                draw_pixel(frame, depth, DVec3::new(x as f64, y as f64, p0.z), color);
            }
        }
    }
}

pub fn draw_rectangle_fill(frame: &mut [u8], depth: &mut [f64], p0: DVec3, p1: DVec3, color: u32) {
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
            draw_pixel(frame, depth, DVec3::new(x as f64, y as f64, p0.z), color);
        }
    }
}

pub fn draw_sprite(frame: &mut [u8], depth: &mut [f64], p: DVec3, sprite: &[&[u8]], scale: i32, color: u32) {
    if scale < 0 { // todo
        return;
    }
    for i in 0..sprite.len() {
        for j in 0..sprite[i].len() {
            if sprite[sprite.len() - 1 - i][j] == 1 {
                for di in 0..scale {
                    for dj in 0..scale {
                        draw_pixel(frame, depth, DVec3::new((p.x as i32 + scale * j as i32 + dj) as f64, (p.y as i32 + scale * i as i32 + di) as f64, p.z), color);
                    }
                }
            }
        }
    }
}

pub fn draw_text(frame: &mut [u8], depth: &mut [f64], p: DVec3, text: &str, font: &[&[&[u8]]], offset: i32, scale: i32, color: u32) {
    let mut dx = 0;
    let mut dy = 0;
    for c in text.as_bytes() {
        if (*c as usize) < font.len() {
            draw_sprite(frame, depth, DVec3::new(p.x + dx as f64, p.y + dy as f64, p.z), &font[*c as usize], scale, color);
            dx += offset * scale;
            if *c == 10 { // LF
                dx = 0;
                dy -= offset * scale;
            }
        }
    }
}

pub fn draw_point_3d(frame: &mut [u8], depth: &mut [f64], v: DVec3, camera: &Camera, color: u32) {
    let p = transform_world_to_screen(v, camera);
    draw_pixel(frame, depth, p, color);
}

pub fn draw_line_3d(frame: &mut [u8], depth: &mut [f64], v0: DVec3, v1: DVec3, camera: &Camera, color: u32) {
    let p0 = transform_world_to_screen(v0, camera);
    let p1 = transform_world_to_screen(v1, camera);
    draw_line(frame, depth, p0, p1, color);
}

pub fn draw_triangle_fill_outline_3d(frame: &mut [u8], depth: &mut [f64], v0: DVec3, v1: DVec3, v2: DVec3, outline_lines: &Vec<DVec3>, camera: &Camera, color: u32, fill: u32) {
    let p0 = transform_world_to_screen(v0, camera);
    let p1 = transform_world_to_screen(v1, camera);
    let p2 = transform_world_to_screen(v2, camera);
    draw_triangle_fill_outline(frame, depth, p0, p1, p2, &outline_lines, color, fill);
}

pub fn draw_polygon_3d(frame: &mut [u8], depth: &mut [f64], polygon: &Vec<DVec3>, camera: &Camera, color: u32, fill: u32) {
    if polygon.len() == 1 {
        draw_point_3d(frame, depth, polygon[0], camera, color);

    } else if polygon.len() == 2 {
        draw_line_3d(frame, depth, polygon[0], polygon[1], camera, color);

    } else if polygon.len() >= 3 {
        let normal = (polygon[1] - polygon[0]).cross(polygon[2] - polygon[0]).normalize() * 10.0;
        if normal.dot(camera.position - polygon[0]) >= 0.0 {
            if fill != 0x00000000 {
                let outline_points: Vec<DVec3> = polygon.iter().map(|v| transform_world_to_screen(*v, camera)).collect();
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
}

pub fn draw_mesh_3d(frame: &mut [u8], depth: &mut [f64], mesh: &Vec<Vec<DVec3>>, camera: &Camera, color: u32, fill: u32) {
    for polygon in mesh {
        draw_polygon_3d(frame, depth, polygon, camera, color, fill);
    }
}

// todo: check bounding box
pub fn draw_object(frame: &mut [u8], depth: &mut [f64], object: &Object, camera: &Camera) {
    let (scale, _, _) = object.model.to_scale_rotation_translation(); // todo: dedicated lod properties
    let distance = (object.model.transform_point3(DVec3::ZERO) - camera.model.transform_point3(DVec3::ZERO)).length();
    if distance > 10000.0 && scale.x < 1000.0 {
        draw_point_3d(frame, depth, object.model.transform_point3(DVec3::ZERO), camera, object.color);
    } else if distance > 4000.0 && scale.x < 1000.0 {
        draw_mesh_3d(frame, depth, &transform_mesh(&object.mesh, object.model), camera, object.color, 0x00000000);
    } else {
        draw_mesh_3d(frame, depth, &transform_mesh(&object.mesh, object.model), camera, object.color, object.fill);
    }
}

pub fn color_to_float(color: u32) -> (f64, f64, f64, f64) {
    let r = ((color >> 24) as u8) as f64 / 255.0;
    let g = ((color >> 16) as u8) as f64 / 255.0;
    let b = ((color >> 8) as u8) as f64 / 255.0;
    let a = ((color) as u8) as f64 / 255.0;
    (r, g, b, a)
}

pub fn float_to_color((r, g, b, a): (f64, f64, f64, f64)) -> u32 {
    let mut color = 0x00000000;
    color |= ((r * 255.0) as u32) << 24;
    color |= ((g * 255.0) as u32) << 16;
    color |= ((b * 255.0) as u32) << 8;
    color |= (a * 255.0) as u32;
    color
}