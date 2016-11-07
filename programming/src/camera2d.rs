use cgmath::{Vector3, vec3, Matrix4};

/// Camera for 2D scenes which can zoom in an out and pan around
pub struct Camera2d {
    pub position: Vector3<f32>,
    pub zoom: f32
}

impl Camera2d {
    pub fn new() -> Camera2d {
        Camera2d { position: vec3(0.0, 0.0, 2.0), zoom: 1.0 }
    }
    pub fn translate(&mut self, x: f32, y: f32) {
        self.position += vec3(x, y, 0.0) / self.zoom;
    }
    pub fn zoom(&mut self, z: f32) {
        self.zoom += z;
        // Only allow up to 10x zoom out
        if self.zoom < 0.1 {
            self.zoom = 0.1;
        }
    }
    pub fn get_mat4(&self) -> Matrix4<f32> {
        Matrix4::from_nonuniform_scale(self.zoom, self.zoom, 1.0)
            * Matrix4::from_translation(self.position)
    }
}

