/// Manages displaying and toggling interaction modes with
/// a specific BSpline surface in the scene.

use std::f32;

use glium::{Surface, VertexBuffer, Program, DrawParameters};
use glium::backend::Facade;
use glium::index::{NoIndices, PrimitiveType};
use imgui::Ui;

use bspline::BSpline;
use point::Point;

pub struct DisplaySurfInterpolation<'a, F: 'a + Facade> {
    display: &'a F,
    //surf: DisplaySurf<'a, F>,
    // The input curves
    input_curves_vbo: Vec<VertexBuffer<Point>>,
    draw_input_curves: bool,
    curve_color: [f32; 3],
}

impl<'a, F: 'a + Facade> DisplaySurfInterpolation<'a, F> {
    pub fn new(curves: Vec<BSpline<Point>>, display: &'a F) -> DisplaySurfInterpolation<'a, F> {
        let mut control_points = Vec::new();
        let mut input_curves_vbo = Vec::with_capacity(curves.len());
        let step_size = 0.01;
        for c in curves.iter() {
            let t_range = c.knot_domain();
            let steps = ((t_range.1 - t_range.0) / step_size) as usize;
            let mut points = Vec::with_capacity(steps);
            // Just draw the first one for now
            for s in 0..steps + 1 {
                let t = step_size * s as f32 + t_range.0;
                points.push(c.point(t));
            }
            input_curves_vbo.push(VertexBuffer::new(display, &points[..]).unwrap());

            for pt in &c.control_points[..] {
                control_points.push(*pt);
            }
        }
        let control_points_vbo = VertexBuffer::new(display, &control_points[..]).unwrap();

        DisplaySurfInterpolation { display: display,
                      //surf: surf,
                      input_curves_vbo: input_curves_vbo,
                      draw_input_curves: true,
                      curve_color: [0.1, 0.8, 0.1],
        }
    }
    pub fn render<S: Surface>(&self, target: &mut S, program: &Program, draw_params: &DrawParameters,
                  proj_view: &[[f32; 4]; 4], selected: bool, attenuation: f32) {
        let curve_color =
            if selected {
                self.curve_color
            } else {
                [attenuation * self.curve_color[0], attenuation * self.curve_color[1],
                  attenuation * self.curve_color[2]]
            };
        let uniforms = uniform! {
            proj_view: *proj_view,
            pcolor: curve_color,
        };
        // Draw the curve
        if self.draw_input_curves {
            for iso in &self.input_curves_vbo[..] {
                target.draw(iso, &NoIndices(PrimitiveType::LineStrip),
                            &program, &uniforms, &draw_params).unwrap();
            }
        }
        //surf.render(target, program, draw_params, proj_view, selected, attenuation);
    }
    pub fn draw_ui(&mut self, ui: &Ui) {
        ui.text(im_str!("3D Surface"));
        ui.checkbox(im_str!("Draw Input Curves"), &mut self.draw_input_curves);
        ui.color_edit3(im_str!("Input Color"), &mut self.curve_color).build();
        //surf.draw_ui(ui);
    }
}



