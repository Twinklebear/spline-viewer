extern crate cgmath;

use cgmath::prelude::*;
use cgmath::{Basis3, Matrix4, Quaternion, Rad, Vector2, Vector3};

use point::clamp;

pub struct ArcballCamera {
	look_at: Matrix4<f32>,
	translation: Matrix4<f32>,
	rotation: Quaternion<f32>,
	camera: Matrix4<f32>,
	inv_camera: Matrix4<f32>,
	motion_speed: f32,
	rotation_speed: f32,
	inv_screen: [f32; 2],
}

/// Assumes all input x and y coordinates are in normalized screen coordinates [-1, 1] in x and y
impl ArcballCamera {
	pub fn new(look_at: &Matrix4<f32>, motion_speed: f32, rotation_speed: f32, inv_screen: [f32; 2]) -> ArcballCamera {
		ArcballCamera {
			look_at: look_at.clone(),
			translation: Transform::one(),
			rotation: Quaternion::new(1.0, 0.0, 0.0, 0.0),
			camera: look_at.clone(),
			inv_camera: look_at.invert().unwrap(),
			motion_speed: motion_speed,
			rotation_speed: rotation_speed,
			inv_screen: inv_screen,
		}
	}
	pub fn get_mat4(&self) -> Matrix4<f32> {
		self.camera
	}
	pub fn rotate(&mut self, mouse_prev: Vector2<f32>, mouse_cur: Vector2<f32>, elapsed: f32) {
		let m_cur = Vector2::new(clamp(mouse_cur.x * 2.0 * self.inv_screen[0] - 1.0, -1.0, 1.0),
										clamp(1.0 - 2.0 * mouse_cur.y * self.inv_screen[1], -1.0, 1.0));
		let m_prev = Vector2::new(clamp(mouse_prev.x * 2.0 * self.inv_screen[0] - 1.0, -1.0, 1.0),
										 clamp(1.0 - 2.0 * mouse_prev.y * self.inv_screen[1], -1.0, 1.0));
		let mouse_cur_ball = ArcballCamera::screen_to_arcball(m_cur);
		let mouse_prev_ball = ArcballCamera::screen_to_arcball(m_prev);
		self.rotation = mouse_cur_ball * mouse_prev_ball * self.rotation;
		self.camera = self.translation * self.look_at * Matrix4::from(self.rotation);
		self.inv_camera = self.camera.invert().unwrap();
	}
	pub fn zoom(&mut self, amount: f32, elapsed: f32) {
		let motion = Vector3::new(0.0, 0.0, amount);
		self.translation = Matrix4::from_translation(motion * self.motion_speed * elapsed * 8.0) * self.translation;
		self.camera = self.translation * self.look_at * Matrix4::from(self.rotation);
		self.inv_camera = self.camera.invert().unwrap();
	}
	pub fn pan(&mut self, mouse_delta: Vector2<f32>, elapsed: f32) {
		let motion = Vector3::new(mouse_delta.x, mouse_delta.y, 0.0) * self.motion_speed * elapsed * 0.05;
		self.translation = Matrix4::from_translation(motion) * self.translation;
		self.camera = self.translation * self.look_at * Matrix4::from(self.rotation);
		self.inv_camera = self.camera.invert().unwrap();
	}
	pub fn update_screen(&mut self, width: f32, height: f32) {
		self.inv_screen[0] = 1.0 / width;
		self.inv_screen[1] = 1.0 / height;
	}
	fn screen_to_arcball(p: Vector2<f32>) -> Quaternion<f32> {
		let dist = cgmath::dot(p, p);
		// If we're on/in the sphere return the point on it
		if dist <= 1.0 {
			Quaternion::new(0.0, p.x, p.y, f32::sqrt(1.0 - dist))
		} else {
			let unit_p = p.normalize();
			Quaternion::new(0.0, unit_p.x, unit_p.y, 0.0)
		}
	}
}

