/// Manages displaying and toggling interaction modes with
/// a specific BSpline surface in the scene.

use std::f32;

use glium::{Surface, VertexBuffer, Program, DrawParameters};
use glium::backend::Facade;
use glium::index::{NoIndices, PrimitiveType};
use imgui::Ui;

use bspline_surf::BSplineSurf;
use point::Point;

pub struct DisplaySurf<'a, F: 'a + Facade> {
    display: &'a F,
    surf: BSplineSurf<Point>,
    isolines_u_vbos: Vec<VertexBuffer<Point>>,
    isolines_v_vbos: Vec<VertexBuffer<Point>>,
    control_points_vbo: VertexBuffer<Point>,
    draw_surf: bool,
    draw_control_mesh: bool,
    draw_control_points: bool,
    curve_color: [f32; 3],
    control_color: [f32; 3],
}

impl<'a, F: 'a + Facade> DisplaySurf<'a, F> {
    pub fn new(surf: BSplineSurf<Point>, display: &'a F) -> DisplaySurf<'a, F> {
        let isoline_step_size = 0.1;
        let step_size = 0.01;

        let t_range_u = surf.knot_domain_u();
        let t_range_v = surf.knot_domain_v();
        let isoline_steps_u = ((t_range_u.1 - t_range_u.0) / isoline_step_size) as usize;
        let isoline_steps_v = ((t_range_v.1 - t_range_v.0) / isoline_step_size) as usize;

        let mut isolines_u_vbos = Vec::with_capacity(isoline_steps_v);
        let mut isolines_v_vbos = Vec::with_capacity(isoline_steps_u);

        // Compute isolines along u
        for vs in 0..isoline_steps_v + 1 {
            let v = isoline_step_size * vs as f32 + t_range_v.0;
            let curve = surf.isoline_u(v);
            let t_range = curve.knot_domain();
            let steps = ((t_range.1 - t_range.0) / step_size) as usize;
            let mut points = Vec::with_capacity(steps);
            for s in 0..steps + 1 {
                let t = step_size * s as f32 + t_range.0;
                points.push(curve.point(t));
            }
            isolines_u_vbos.push(VertexBuffer::new(display, &points[..]).unwrap());
        }
        // Compute isolines along v
        for us in 0..isoline_steps_u + 1 {
            let u = isoline_step_size * us as f32 + t_range_u.0;
            let curve = surf.isoline_v(u);
            let t_range = curve.knot_domain();
            let steps = ((t_range.1 - t_range.0) / step_size) as usize;
            let mut points = Vec::with_capacity(steps);
            for s in 0..steps + 1 {
                let t = step_size * s as f32 + t_range.0;
                points.push(curve.point(t));
            }
            isolines_v_vbos.push(VertexBuffer::new(display, &points[..]).unwrap());
        }

        let mut control_points = Vec::new();
        for r in &surf.control_mesh[..] {
            for p in &r[..] {
                control_points.push(*p);
            }
        }
        let control_points_vbo = VertexBuffer::new(display, &control_points[..]).unwrap();

        DisplaySurf { display: display,
                      surf: surf,
                      isolines_u_vbos: isolines_u_vbos,
                      isolines_v_vbos: isolines_v_vbos,
                      control_points_vbo: control_points_vbo,
                      draw_surf: true,
                      draw_control_mesh: true,
                      draw_control_points: true,
                      curve_color: [0.8, 0.8, 0.1],
                      control_color: [0.8, 0.8, 0.8],
        }
    }
    pub fn render<S: Surface>(&self, target: &mut S, program: &Program, draw_params: &DrawParameters,
                  proj_view: &[[f32; 4]; 4], selected: bool, attenuation: f32) {
        let (curve_color, control_color) =
            if selected {
                (self.curve_color, self.control_color)
            } else {
                ([attenuation * self.curve_color[0], attenuation * self.curve_color[1],
                  attenuation * self.curve_color[2]],
                 [attenuation * self.control_color[0], attenuation * self.control_color[1],
                  attenuation * self.control_color[2]])
            };
        let uniforms = uniform! {
            proj_view: *proj_view,
            pcolor: curve_color,
        };
        // Draw the curve
        if self.draw_surf {
            for iso in self.isolines_u_vbos.iter().chain(self.isolines_v_vbos.iter()) {
                target.draw(iso, &NoIndices(PrimitiveType::LineStrip),
                            &program, &uniforms, &draw_params).unwrap();
            }
        }
        let uniforms = uniform! {
            proj_view: *proj_view,
            pcolor: control_color,
        };
        /*
        // Draw the control mesh
        if self.draw_control_mesh {
        target.draw(&self.control_points_vbo, &NoIndices(PrimitiveType::LineStrip),
        &program, &uniforms, &draw_params).unwrap();
        }
        */
        if self.draw_control_points {
            // Draw the control points
            target.draw(&self.control_points_vbo, &NoIndices(PrimitiveType::Points),
                        &program, &uniforms, &draw_params).unwrap();
        }
    }
    pub fn draw_ui(&mut self, ui: &Ui) {
        ui.text(im_str!("3D Surface"));
        //ui.text(im_str!("Number of Control Points: {}", self.curve.control_points.len()));
        ui.checkbox(im_str!("Draw Surface"), &mut self.draw_surf);
        ui.checkbox(im_str!("Draw Control Mesh"), &mut self.draw_control_mesh);
        ui.checkbox(im_str!("Draw Control Points"), &mut self.draw_control_points);
        /*
        let mut curve_degree = self.curve.degree() as i32;
        if ui.slider_int(im_str!("Curve Degree"), &mut curve_degree, 1,
            self.curve.max_possible_degree() as i32).build()
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
            self.control_points_vbo = VertexBuffer::new(self.display, &self.curve.control_points[..]).unwrap();
            let mut points = Vec::with_capacity(steps);
            // Just draw the first one for now
            for s in 0..steps + 1 {
                let t = step_size * s as f32 + t_range.0;
                points.push(self.curve.point(t));
            }
            self.curve_points_vbo = VertexBuffer::new(self.display, &points[..]).unwrap();
        }
        */
        ui.color_edit3(im_str!("Curve Color"), &mut self.curve_color).build();
        ui.color_edit3(im_str!("Control Color"), &mut self.control_color).build();
    }
}


