/// Manages displaying and toggling interaction modes with
/// a specific Bezier curve in the scene.

use glium::{Surface, VertexBuffer, Program, DrawParameters};
use glium::backend::Facade;
use glium::index::{NoIndices, PrimitiveType};
use imgui::Ui;

use bezier::Bezier;
use point::Point;

pub struct DisplayCurve {
    curve: Bezier<Point>,
    curve_points_vbo:  VertexBuffer<Point>,
    control_points_vbo: VertexBuffer<Point>,
    draw_curve: bool,
    draw_control_poly: bool,
    draw_control_points: bool,
}

impl DisplayCurve {
    pub fn new<F: Facade>(curve: Bezier<Point>, display: &F) -> DisplayCurve {
        let step_size = 0.01;
        let t_range = (0.0, 1.0);
        let steps = ((t_range.1 - t_range.0) / step_size) as usize;
        let control_points_vbo = VertexBuffer::new(display, &curve.control_points[..]).unwrap();
        let mut points = Vec::with_capacity(steps);
        // Just draw the first one for now
        for s in 0..steps + 1 {
            let t = step_size * s as f32 + t_range.0;
            points.push(curve.point(t));
        }
        let curve_points_vbo = VertexBuffer::new(display, &points[..]).unwrap();

        DisplayCurve { curve: curve,
                       curve_points_vbo: curve_points_vbo,
                       control_points_vbo: control_points_vbo,
                       draw_curve: true,
                       draw_control_poly: true,
                       draw_control_points: true,
        }
    }
    pub fn render<S: Surface>(&self, target: &mut S, program: &Program, draw_params: &DrawParameters,
                  proj_view: &[[f32; 4]; 4]) {
        if !self.curve.control_points.is_empty() {
            let uniforms = uniform! {
                proj_view: *proj_view,
                pcolor: [0.8f32, 0.8f32, 0.1f32],
            };
            // Draw the curve
            if self.draw_curve {
                target.draw(&self.curve_points_vbo, &NoIndices(PrimitiveType::LineStrip),
                            &program, &uniforms, &draw_params).unwrap();
            }
            let uniforms = uniform! {
                proj_view: *proj_view,
                pcolor: [0.8f32, 0.8f32, 0.8f32],
            };
            // Draw the control polygon
            if self.draw_control_poly {
                target.draw(&self.control_points_vbo, &NoIndices(PrimitiveType::LineStrip),
                            &program, &uniforms, &draw_params).unwrap();
            }
            if self.draw_control_points {
                // Draw the control points
                target.draw(&self.control_points_vbo, &NoIndices(PrimitiveType::Points),
                            &program, &uniforms, &draw_params).unwrap();
            }
        }
    }
    pub fn draw_ui(&mut self, ui: &Ui) {
        ui.text(im_str!("Number of Control Points: {}", self.curve.control_points.len()));
        ui.checkbox(im_str!("Draw Curve"), &mut self.draw_curve);
        ui.checkbox(im_str!("Draw Control Polygon"), &mut self.draw_control_poly);
        ui.checkbox(im_str!("Draw Control Points"), &mut self.draw_control_points);
    }
}

