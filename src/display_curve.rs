/// Manages displaying and toggling interaction modes with
/// a specific BSpline curve in the scene.
use std::f32;

use glium::backend::Facade;
use glium::index::{NoIndices, PrimitiveType};
use glium::{DrawParameters, Program, Surface, VertexBuffer};
use imgui::Ui;

use bspline::BSpline;
use point::Point;

pub struct DisplayCurve<'a, F: 'a + Facade> {
    display: &'a F,
    pub curve: BSpline<Point>,
    curve_points_vbo: VertexBuffer<Point>,
    control_points_vbo: VertexBuffer<Point>,
    break_points_vbo: VertexBuffer<Point>,
    draw_curve: bool,
    draw_control_poly: bool,
    draw_control_points: bool,
    draw_break_points: bool,
    moving_point: Option<usize>,
    curve_color: [f32; 3],
    control_color: [f32; 3],
    break_point_color: [f32; 3],
}

impl<'a, F: 'a + Facade> DisplayCurve<'a, F> {
    pub fn new(curve: BSpline<Point>, display: &'a F) -> DisplayCurve<'a, F> {
        let control_points_vbo;
        let curve_points_vbo;
        let break_points_vbo;
        if !curve.control_points.is_empty() {
            let step_size = 0.01;
            let t_range = curve.knot_domain();
            let steps = ((t_range.1 - t_range.0) / step_size) as usize;
            control_points_vbo = VertexBuffer::new(display, &curve.control_points[..]).unwrap();
            let mut points = Vec::with_capacity(steps);
            // Just draw the first one for now
            for s in 0..steps + 1 {
                let t = step_size * s as f32 + t_range.0;
                points.push(curve.point(t));
            }
            curve_points_vbo = VertexBuffer::new(display, &points[..]).unwrap();
            let break_points: Vec<_> = curve.knot_domain_iter().map(|b| curve.point(*b)).collect();
            break_points_vbo = VertexBuffer::new(display, &break_points[..]).unwrap();
        } else {
            control_points_vbo = VertexBuffer::empty(display, 10).unwrap();
            curve_points_vbo = VertexBuffer::empty(display, 10).unwrap();
            break_points_vbo = VertexBuffer::empty(display, 10).unwrap();
        }
        DisplayCurve {
            display: display,
            curve: curve,
            curve_points_vbo: curve_points_vbo,
            control_points_vbo: control_points_vbo,
            break_points_vbo: break_points_vbo,
            draw_curve: true,
            draw_control_poly: true,
            draw_control_points: true,
            draw_break_points: true,
            moving_point: None,
            curve_color: [0.8, 0.8, 0.1],
            control_color: [0.8, 0.8, 0.8],
            break_point_color: [0.1, 0.8, 0.8],
        }
    }
    pub fn handle_click(&mut self, pos: Point, shift_down: bool, zoom_factor: f32) {
        // If we're close to control point of the selected curve we're dragging it,
        // otherwise we're adding a new point
        let nearest = self
            .curve
            .control_points()
            .enumerate()
            .map(|(i, x)| (i, (*x - pos).length()))
            .fold(
                (0, f32::MAX),
                |acc, (i, d)| if d < acc.1 { (i, d) } else { acc },
            );
        let point_size = 12.0 / (100.0 * zoom_factor);
        if shift_down {
            self.moving_point = None;
            if nearest.1 < point_size {
                self.curve.remove_point(nearest.0);
            }
        } else if let Some(p) = self.moving_point {
            self.curve.control_points[p] = pos;
        } else if nearest.1 < point_size {
            self.moving_point = Some(nearest.0);
            self.curve.control_points[nearest.0] = pos;
        } else {
            self.moving_point = Some(self.curve.insert_point(pos));
        }
        if !self.curve.control_points.is_empty() {
            let step_size = 0.01;
            let t_range = self.curve.knot_domain();
            let steps = ((t_range.1 - t_range.0) / step_size) as usize;
            self.control_points_vbo =
                VertexBuffer::new(self.display, &self.curve.control_points[..]).unwrap();
            let mut points = Vec::with_capacity(steps);
            // Just draw the first one for now
            for s in 0..steps + 1 {
                let t = step_size * s as f32 + t_range.0;
                points.push(self.curve.point(t));
            }
            self.curve_points_vbo = VertexBuffer::new(self.display, &points[..]).unwrap();
            let break_points: Vec<_> = self
                .curve
                .knot_domain_iter()
                .map(|b| self.curve.point(*b))
                .collect();
            self.break_points_vbo = VertexBuffer::new(self.display, &break_points[..]).unwrap();
        }
    }
    /// Release any held point that was being dragged
    pub fn release_point(&mut self) {
        self.moving_point = None;
    }
    pub fn render<S: Surface>(
        &self,
        target: &mut S,
        program: &Program,
        draw_params: &DrawParameters,
        proj_view: &[[f32; 4]; 4],
        selected: bool,
        attenuation: f32,
    ) {
        let (curve_color, control_color, break_color) = if selected {
            (self.curve_color, self.control_color, self.break_point_color)
        } else {
            (
                [
                    attenuation * self.curve_color[0],
                    attenuation * self.curve_color[1],
                    attenuation * self.curve_color[2],
                ],
                [
                    attenuation * self.control_color[0],
                    attenuation * self.control_color[1],
                    attenuation * self.control_color[2],
                ],
                [
                    attenuation * self.break_point_color[0],
                    attenuation * self.break_point_color[1],
                    attenuation * self.break_point_color[2],
                ],
            )
        };
        if !self.curve.control_points.is_empty() {
            let uniforms = uniform! {
                proj_view: *proj_view,
                pcolor: curve_color,
            };
            // Draw the curve
            if self.draw_curve {
                target
                    .draw(
                        &self.curve_points_vbo,
                        &NoIndices(PrimitiveType::LineStrip),
                        &program,
                        &uniforms,
                        &draw_params,
                    )
                    .unwrap();
            }
            let uniforms = uniform! {
                proj_view: *proj_view,
                pcolor: control_color,
            };
            // Draw the control polygon
            if self.draw_control_poly {
                target
                    .draw(
                        &self.control_points_vbo,
                        &NoIndices(PrimitiveType::LineStrip),
                        &program,
                        &uniforms,
                        &draw_params,
                    )
                    .unwrap();
            }
            if self.draw_control_points {
                // Draw the control points
                target
                    .draw(
                        &self.control_points_vbo,
                        &NoIndices(PrimitiveType::Points),
                        &program,
                        &uniforms,
                        &draw_params,
                    )
                    .unwrap();
            }
            if self.draw_break_points {
                let uniforms = uniform! {
                    proj_view: *proj_view,
                    pcolor: break_color,
                };
                // Draw the control points
                target
                    .draw(
                        &self.break_points_vbo,
                        &NoIndices(PrimitiveType::Points),
                        &program,
                        &uniforms,
                        &draw_params,
                    )
                    .unwrap();
            }
        }
    }
    pub fn draw_ui(&mut self, ui: &Ui) {
        ui.text(im_str!("2D Curve"));
        ui.text(im_str!(
            "Number of Control Points: {}",
            self.curve.control_points.len()
        ));
        ui.checkbox(im_str!("Draw Curve"), &mut self.draw_curve);
        ui.checkbox(im_str!("Draw Control Polygon"), &mut self.draw_control_poly);
        ui.checkbox(
            im_str!("Draw Control Points"),
            &mut self.draw_control_points,
        );
        ui.checkbox(im_str!("Draw Break Points"), &mut self.draw_break_points);
        let mut curve_changed = false;
        // I use the open curve term b/c Elaine will be interacting with it and she
        // calls clamped curves open.
        let mut curve_clamped = self.curve.is_clamped();
        if ui.checkbox(im_str!("Open Curve"), &mut curve_clamped) {
            self.curve.set_clamped(curve_clamped);
            curve_changed = true;
        }
        let mut curve_degree = self.curve.degree() as i32;
        if ui
            .slider_int(
                im_str!("Curve Degree"),
                &mut curve_degree,
                1,
                self.curve.max_possible_degree() as i32,
            )
            .build()
        {
            if self.curve.max_possible_degree() != 0 {
                self.curve.set_degree(curve_degree as usize);
                curve_changed = true;
            }
        }
        if curve_changed && !self.curve.control_points.is_empty() {
            let step_size = 0.01;
            let t_range = self.curve.knot_domain();
            let steps = ((t_range.1 - t_range.0) / step_size) as usize;
            self.control_points_vbo =
                VertexBuffer::new(self.display, &self.curve.control_points[..]).unwrap();
            let mut points = Vec::with_capacity(steps);
            // Just draw the first one for now
            for s in 0..steps + 1 {
                let t = step_size * s as f32 + t_range.0;
                points.push(self.curve.point(t));
            }
            self.curve_points_vbo = VertexBuffer::new(self.display, &points[..]).unwrap();
            let break_points: Vec<_> = self
                .curve
                .knot_domain_iter()
                .map(|b| self.curve.point(*b))
                .collect();
            self.break_points_vbo = VertexBuffer::new(self.display, &break_points[..]).unwrap();
        }
        ui.color_edit3(im_str!("Curve Color"), &mut self.curve_color)
            .build();
        ui.color_edit3(im_str!("Control Color"), &mut self.control_color)
            .build();
        ui.color_edit3(im_str!("Break Point Color"), &mut self.break_point_color)
            .build();
    }
}
