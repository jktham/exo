use core::f32;
use glam::{Vec3, Vec4, Mat4};
use crate::{game::Camera, HEIGHT, WIDTH};

pub fn transform_vertex(vertex: Vec3, model: Mat4) -> Vec3 {
	model.transform_point3(vertex)
}

pub fn transform_polygon(polygon: &Vec<Vec3>, model: Mat4) -> Vec<Vec3> {
	polygon.iter().map(|v| transform_vertex(*v, model)).collect()
}

pub fn transform_mesh(mesh: &Vec<Vec<Vec3>>, model: Mat4) -> Vec<Vec<Vec3>> {
	mesh.iter().map(|p| transform_polygon(p, model)).collect()
}

pub fn transform_world_to_screen(vertex: Vec3, camera: &Camera) -> Vec3 {
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

    let world = Vec4::new(vertex.x, vertex.y, vertex.z, 1.0);
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
