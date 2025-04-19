use core::f64;
use glam::{DVec3, DVec4, DMat4};
use crate::{game::Camera, HEIGHT, WIDTH};

pub const NEAR: f64 = 0.01;
pub const FAR: f64 = 1000000.0;

pub fn transform_vertex(vertex: DVec3, model: DMat4) -> DVec3 {
	model.transform_point3(vertex)
}

pub fn transform_polygon(polygon: &Vec<DVec3>, model: DMat4) -> Vec<DVec3> {
	polygon.iter().map(|v| transform_vertex(*v, model)).collect()
}

pub fn transform_mesh(mesh: &Vec<Vec<DVec3>>, model: DMat4) -> Vec<Vec<DVec3>> {
	mesh.iter().map(|p| transform_polygon(p, model)).collect()
}

pub fn transform_world_to_screen(vertex: DVec3, camera: &Camera) -> DVec3 {
    let w = WIDTH as f64;
    let h = HEIGHT as f64;
    let n = NEAR;
    let f = FAR;
    let phi = camera.fov / 180.0 * f64::consts::PI;
    let r = f64::tan(phi/2.0) * n;
    let t = r * h/w;

    let projection = DMat4::from_cols_array(&[
        n/r, 0.0, 0.0, 0.0,
        0.0, n/t, 0.0, 0.0,
        0.0, 0.0, -(f+n)/(f-n), -2.0*f*n/(f-n),
        0.0, 0.0, -1.0, 0.0,
    ]).transpose();

    let world = DVec4::new(vertex.x, vertex.y, vertex.z, 1.0);
    let eye = camera.view * world;
    let clip = projection * eye;
    let ndc = DVec3::new(clip.x/clip.w, clip.y/clip.w, clip.z/clip.w);
    let screen = DVec3::new(
        w/2.0 * ndc.x + w/2.0, 
        h/2.0 * ndc.y + h/2.0, 
        (f-n)/2.0 * ndc.z + (f+n)/2.0
    );

    screen
}

pub fn out_of_bounds(p: DVec3, tolerance: i32) -> bool {
    (p.x as i32) < 0 - tolerance || 
    (p.x as i32) >= WIDTH as i32 + tolerance || 
    (p.y as i32) < 0 - tolerance || 
    (p.y as i32) >= HEIGHT as i32 + tolerance ||
    // p.z < NEAR ||
    p.z < 0.0 ||
    p.z > FAR
}
