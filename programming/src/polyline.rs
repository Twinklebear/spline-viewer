//! Manages displaying and toggling interaction modes with
//! a specific Bezier curve in the scene.
#![allow(dead_code)]

use std::f32;

use glium::{Surface, VertexBuffer, Program, DrawParameters};
use glium::backend::Facade;
use glium::index::{NoIndices, PrimitiveType};
use imgui::Ui;

use point::Point;

pub struct Polyline {
    points_vbo: VertexBuffer<Point>,
    draw_lines: bool,
    draw_points: bool,
    color: [f32; 3],
}

impl Polyline {
    pub fn new<F: Facade>(points: Vec<Point>, display: &F) -> Polyline {
        let points_vbo  = VertexBuffer::immutable(display, &points[..]).unwrap();
        Polyline { points_vbo: points_vbo,
                   draw_lines: true,
                   draw_points: true,
                   color: [0.8, 0.8, 0.8],
        }
    }
    pub fn render<S: Surface>(&self, target: &mut S, program: &Program, draw_params: &DrawParameters,
                  proj_view: &[[f32; 4]; 4]) {
        let uniforms = uniform! {
            proj_view: *proj_view,
            pcolor: self.color,
        };
        // Draw the control polygon
        if self.draw_lines {
            target.draw(&self.points_vbo, &NoIndices(PrimitiveType::LineStrip),
                        &program, &uniforms, &draw_params).unwrap();
        }
        // Draw the control points
        if self.draw_points {
            target.draw(&self.points_vbo, &NoIndices(PrimitiveType::Points),
                        &program, &uniforms, &draw_params).unwrap();
        }
    }
    pub fn draw_ui(&mut self, ui: &Ui) {
        ui.text(im_str!("Number of Points: {}", self.points_vbo.len()));
        ui.checkbox(im_str!("Draw Polyline"), &mut self.draw_lines);
        ui.checkbox(im_str!("Draw Points"), &mut self.draw_points);
        ui.color_edit3(im_str!("Color"), &mut self.color).build();
    }
}

