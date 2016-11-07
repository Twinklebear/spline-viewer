extern crate cgmath;

use cgmath::prelude::*;
use cgmath::{Basis3, Matrix3, Matrix4, Rad, Vector2, Vector3};

/// TODO: Replace this garbage wanna-be arcball with a real arcball camera
/// copied from glt

// Trait names are fully qualified to make it clear where they come from
pub struct ArcballCamera<T: cgmath::BaseFloat> {
  p_mouse: Vector2<T>,
  pub target: Vector3<T>,
  rotation: Basis3<T>,
  distance: T,
  spin_speed: T,
  zoom_speed: T,
  pan_speed: T,
  rotating: bool,
  panning: bool,
}

/// Assumes all input x and y coordinates are in normalized screen coordinates [-1, 1] in x and y
impl<T: cgmath::BaseFloat> ArcballCamera<T> {
  pub fn new() -> ArcballCamera<T> {
    ArcballCamera {
      p_mouse: Vector2::zero(),
      target: Vector3::zero(),
      rotation: Basis3::one(),
      distance: T::zero(),
      spin_speed: T::one(),
      zoom_speed: T::one(),
      pan_speed: T::one(),
      rotating: false,
      panning: false,
    }
  }

  pub fn get_transform_mat(& self) -> Matrix4<T> {
    let cam_position = self.get_position();
    let position_transform = Matrix4::from_translation(cam_position);
    let rotation_transform = Matrix3::from(self.rotation.invert());
    // The normal order of operations (position * rotation * scale) is reversed here, because the matrix is inverted
    Matrix4::from(rotation_transform) * position_transform
  }

  pub fn get_position(& self) -> Vector3<T> {
    -(self.target + self.rotation.rotate_vector(Vector3::unit_z() * self.distance))
  }

  pub fn set_distance(&mut self, distance: T) -> &mut Self {
    self.distance = distance.max(T::zero());
    self
  }

  pub fn set_rotation(&mut self, rotation: Basis3<T>) -> &mut Self {
    self.rotation = rotation;
    self
  }

  pub fn set_target(&mut self, target: Vector3<T>) -> &mut Self {
    self.target = target;
    self
  }

  pub fn set_spin_speed(&mut self, speed: T) -> &mut Self {
    self.spin_speed = speed;
    self
  }

  pub fn set_zoom_speed(&mut self, speed: T) -> &mut Self {
    self.zoom_speed = speed;
    self
  }

  pub fn set_pan_speed(&mut self, speed: T) -> &mut Self {
    self.pan_speed = speed;
    self
  }

  pub fn rotate_start(&mut self, pos: Vector2<T>) {
    self.rotating = true;
    self.p_mouse = pos;
  }

  pub fn rotate_end(&mut self) {
    self.rotating = false;
  }

  pub fn pan_start(&mut self, pos: Vector2<T>) {
    self.panning = true;
    self.p_mouse = pos;
  }

  pub fn pan_end(&mut self) {
    self.panning = false;
  }

  pub fn get_vec_on_ball(input: Vector2<T>) -> Vector3<T> {
    let dist = input.magnitude();
    let point_z = if dist <= T::one() { (T::one() - dist).sqrt() } else { T::zero() };
    Vector3::new(input.x, input.y, point_z).normalize()
  }

  pub fn update(&mut self, cur_mouse: Vector2<T>) {
    if self.rotating {
      let prev_pt = ArcballCamera::get_vec_on_ball(self.p_mouse);
      let cur_pt = ArcballCamera::get_vec_on_ball(cur_mouse);
      let angle = Rad::acos(prev_pt.dot(cur_pt).min(T::one())) * self.spin_speed;
      // The order of the cross product here gets you the correct rotation direction
      let rot_vec = cur_pt.cross(prev_pt).normalize();
      let rotation = Basis3::from_axis_angle(rot_vec, angle);
      self.rotation = self.rotation * rotation;
      self.p_mouse = cur_mouse;
    } else if self.panning {
      // Note that the direction of target point movement is the reverse of the direction of mouse movement
      let mouse_vec = -(cur_mouse - self.p_mouse).normalize_to(self.pan_speed);
      let left_vec = self.rotation.rotate_vector(Vector3::unit_x() * mouse_vec.x);
      let up_vec = self.rotation.rotate_vector(Vector3::unit_y() * mouse_vec.y);
      self.target += left_vec + up_vec;
      self.p_mouse = cur_mouse;
    }
  }

  pub fn zoom(&mut self, d: T) {
    self.distance = (self.distance + d * self.zoom_speed).max(T::zero());
  }
}

