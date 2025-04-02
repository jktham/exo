#![allow(dead_code)]

use glam::Vec3;

pub fn hull_mesh() -> Vec<Vec<Vec3>> {
	vec![
		vec![
			Vec3::new(-4.0, 0.0, 2.0), // stern L
			Vec3::new(-2.0, 1.0, 2.0), // stern TL
			Vec3::new(2.0, 1.0, 2.0), // stern TR
			Vec3::new(4.0, 0.0, 2.0), // stern R
			Vec3::new(2.0, -1.0, 2.0), // stern BR
			Vec3::new(-2.0, -1.0, 2.0), // stern BL
		],
		vec![
			Vec3::new(-3.0, 0.0, 0.0), // mid L
			Vec3::new(0.0, 1.0, 0.0), // mid T
			Vec3::new(3.0, 0.0, 0.0), // mid R
			Vec3::new(0.0, -1.0, 0.0), // mid B
		],
		vec![
			Vec3::new(-2.0, 0.5, 2.0), // thruster LTR
			Vec3::new(-2.0, -0.5, 2.0), // thruster LBR
			Vec3::new(-3.0, 0.0, 2.0), // thruster LL
		],
		vec![
			Vec3::new(2.0, 0.5, 2.0), // thruster RTL
			Vec3::new(2.0, -0.5, 2.0), // thruster RBL
			Vec3::new(3.0, 0.0, 2.0), // thruster RR
		],
		vec![
			Vec3::new(-2.0, 1.0, 2.0), // stern TL
			Vec3::new(2.0, 1.0, 2.0), // stern TR
			Vec3::new(0.0, 1.0, 0.0), // mid T
		],
		vec![
			Vec3::new(2.0, -1.0, 2.0), // stern BR
			Vec3::new(-2.0, -1.0, 2.0), // stern BL
			Vec3::new(0.0, -1.0, 0.0), // mid B
		],
		vec![
			Vec3::new(0.0, 1.0, 0.0), // mid T
			Vec3::new(3.0, 0.0, 0.0), // mid R
			Vec3::new(2.0, 1.0, 2.0), // stern TR
		],
		vec![
			Vec3::new(3.0, 0.0, 0.0), // mid R
			Vec3::new(0.0, -1.0, 0.0), // mid B
			Vec3::new(2.0, -1.0, 2.0), // stern BR
		],
		vec![
			Vec3::new(-3.0, 0.0, 0.0), // mid L
			Vec3::new(0.0, 1.0, 0.0), // mid T
			Vec3::new(-2.0, 1.0, 2.0), // stern TL
		],
		vec![
			Vec3::new(-3.0, 0.0, 0.0), // mid L
			Vec3::new(0.0, -1.0, 0.0), // mid B
			Vec3::new(-2.0, -1.0, 2.0), // stern BL
		],
		vec![
			Vec3::new(-4.0, 0.0, 2.0), // stern L
			Vec3::new(-2.0, 1.0, 2.0), // stern TL
			Vec3::new(-3.0, 0.0, 0.0), // mid L
		],
		vec![
			Vec3::new(2.0, 1.0, 2.0), // stern TR
			Vec3::new(4.0, 0.0, 2.0), // stern R
			Vec3::new(3.0, 0.0, 0.0), // mid R
		],
		vec![
			Vec3::new(4.0, 0.0, 2.0), // stern R
			Vec3::new(2.0, -1.0, 2.0), // stern BR
			Vec3::new(3.0, 0.0, 0.0), // mid R
		],
		vec![
			Vec3::new(-4.0, 0.0, 2.0), // stern L
			Vec3::new(-2.0, -1.0, 2.0), // stern BL
			Vec3::new(-3.0, 0.0, 0.0), // mid L
		],
		vec![
			Vec3::new(-3.0, 0.0, 0.0), // mid L
			Vec3::new(0.0, 1.0, 0.0), // mid T
			Vec3::new(0.0, 0.0, -2.0), // bow C
		],
		vec![
			Vec3::new(0.0, 1.0, 0.0), // mid T
			Vec3::new(3.0, 0.0, 0.0), // mid R
			Vec3::new(0.0, 0.0, -2.0), // bow C
		],
		vec![
			Vec3::new(3.0, 0.0, 0.0), // mid R
			Vec3::new(0.0, -1.0, 0.0), // mid B
			Vec3::new(0.0, 0.0, -2.0), // bow C
		],
		vec![
			Vec3::new(-3.0, 0.0, 0.0), // mid L
			Vec3::new(0.0, -1.0, 0.0), // mid B
			Vec3::new(0.0, 0.0, -2.0), // bow C
		],
	]
}

pub fn front_thruster_mesh() -> Vec<Vec<Vec3>> {
	vec![
		vec![
			Vec3::new(-2.0, 0.5, 2.0), // thruster LTR
			Vec3::new(-2.0, -0.5, 2.0), // thruster LBR
			Vec3::new(-3.0, 0.0, 2.0), // thruster LL
		],
		vec![
			Vec3::new(2.0, 0.5, 2.0), // thruster RTL
			Vec3::new(2.0, -0.5, 2.0), // thruster RBL
			Vec3::new(3.0, 0.0, 2.0), // thruster RR
		],
	]
}