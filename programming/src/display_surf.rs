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
    // Plain isolines along the curve
    isolines_u_vbos: Vec<VertexBuffer<Point>>,
    isolines_v_vbos: Vec<VertexBuffer<Point>>,
    // Isolines along the greville abscissa
    greville_u_vbos: Vec<VertexBuffer<Point>>,
    greville_v_vbos: Vec<VertexBuffer<Point>>,
    // Isolines at each knot value
    knot_u_vbos: Vec<VertexBuffer<Point>>,
    knot_v_vbos: Vec<VertexBuffer<Point>>,
    control_points_vbo: VertexBuffer<Point>,
    draw_surf: bool,
    draw_greville: bool,
    draw_knots: bool,
    draw_control_points: bool,
    curve_color: [f32; 3],
    greville_color: [f32; 3],
    knot_color: [f32; 3],
    control_color: [f32; 3],
}

impl<'a, F: 'a + Facade> DisplaySurf<'a, F> {
    pub fn new(surf: BSplineSurf<Point>, display: &'a F) -> DisplaySurf<'a, F> {
        let isoline_step_size = 0.1;
        let step_size = 0.01;

        let t_range_u = surf.knot_domain_u();
        let t_range_v = surf.knot_domain_v();

        let isoline_start_steps_u = ((t_range_u.1 - t_range_u.0) / isoline_step_size) as usize;
        let isoline_start_steps_v = ((t_range_v.1 - t_range_v.0) / isoline_step_size) as usize;
        let steps_u = ((t_range_u.1 - t_range_u.0) / step_size) as usize;
        let steps_v = ((t_range_v.1 - t_range_v.0) / step_size) as usize;

        let abscissa_u = surf.greville_abscissa_u();
        let abscissa_v = surf.greville_abscissa_v();

        // We need in addition to the regular line sample steps to also sample where
        // an isoline is along the other axis, or greville point, or knot value so that
        // when the lines cross they both have that crossing point
        // Every knot value along u that we're going to have an isoline on
        let mut t_vals_u: Vec<_> = (0..isoline_start_steps_u + 1).map(|us| isoline_step_size * us as f32 + t_range_u.0)
            .chain(abscissa_u.iter().map(|x| *x))
            .chain(surf.knots_u.iter()
                   .filter_map(|x| if *x >= t_range_u.0 && *x <= t_range_u.1 { Some(*x) } else { None }))
            .collect();
        t_vals_u.sort_by(|a, b| a.partial_cmp(b).unwrap());
        t_vals_u.dedup();

        // Every knot value along v that we're going to have an isoline on
        let mut t_vals_v: Vec<_> = (0..isoline_start_steps_v + 1).map(|vs| isoline_step_size * vs as f32 + t_range_v.0)
            .chain(abscissa_v.iter().map(|x| *x))
            .chain(surf.knots_v.iter()
                   .filter_map(|x| if *x >= t_range_v.0 && *x <= t_range_v.1 { Some(*x) } else { None }))
            .collect();
        t_vals_v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        t_vals_v.dedup();

        // t values for an isoline along u
        let mut isoline_u_t_vals: Vec<_> = (0..steps_u + 1).map(|u| step_size * u as f32 + t_range_u.0)
            .chain(t_vals_u.iter().map(|x| *x)).collect();
        isoline_u_t_vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
        isoline_u_t_vals.dedup();

        // t values for an isoline along v
        let mut isoline_v_t_vals: Vec<_> = (0..steps_v + 1).map(|u| step_size * u as f32 + t_range_v.0)
            .chain(t_vals_v.iter().map(|x| *x)).collect();
        isoline_v_t_vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
        isoline_v_t_vals.dedup();

        let mut greville_u_vbos = Vec::with_capacity(abscissa_u.len());
        let mut greville_v_vbos = Vec::with_capacity(abscissa_v.len());
        // For each Greville abscissa on u draw an isoline along v
        for u in &abscissa_u[..] {
            let curve = surf.isoline_v(*u);
            let mut points = Vec::with_capacity(steps_v);
            for t in &isoline_v_t_vals[..] {
                points.push(curve.point(*t));
            }
            greville_u_vbos.push(VertexBuffer::new(display, &points[..]).unwrap());
        }
        // For each Greville abscissa on v draw an isoline along u
        for v in &abscissa_v[..] {
            let curve = surf.isoline_u(*v);
            let mut points = Vec::with_capacity(steps_u);
            for t in &isoline_u_t_vals[..] {
                points.push(curve.point(*t));
            }
            greville_v_vbos.push(VertexBuffer::new(display, &points[..]).unwrap());
        }

        let mut knot_u_vbos = Vec::with_capacity(surf.knots_u.len());
        let mut knot_v_vbos = Vec::with_capacity(surf.knots_v.len());
        // For each knot on u draw an isoline along v
        for u in surf.knot_domain_u_iter() {
            let curve = surf.isoline_v(*u);
            let mut points = Vec::with_capacity(steps_v);
            for t in &isoline_v_t_vals[..] {
                points.push(curve.point(*t));
            }
            knot_u_vbos.push(VertexBuffer::new(display, &points[..]).unwrap());
        }
        // For each knot on v draw an isoline along u
        for v in surf.knot_domain_v_iter() {
            let curve = surf.isoline_u(*v);
            let mut points = Vec::with_capacity(steps_u);
            for t in &isoline_u_t_vals[..] {
                points.push(curve.point(*t));
            }
            knot_v_vbos.push(VertexBuffer::new(display, &points[..]).unwrap());
        }

        let mut isolines_u_vbos = Vec::with_capacity(isoline_start_steps_v);
        let mut isolines_v_vbos = Vec::with_capacity(isoline_start_steps_u);
        // Compute isolines along u
        for vs in 0..isoline_start_steps_v + 1 {
            let v = isoline_step_size * vs as f32 + t_range_v.0;
            if !abscissa_v.iter().chain(surf.knots_v.iter()).any(|x| *x == v) {
                let curve = surf.isoline_u(v);
                let mut points = Vec::with_capacity(steps_u);
                for t in &isoline_u_t_vals[..] {
                    points.push(curve.point(*t));
                }
                isolines_u_vbos.push(VertexBuffer::new(display, &points[..]).unwrap());
            }
        }
        // Compute isolines along v
        for us in 0..isoline_start_steps_u + 1 {
            let u = isoline_step_size * us as f32 + t_range_u.0;
            if !abscissa_u.iter().chain(surf.knots_u.iter()).any(|x| *x == u) {
                let curve = surf.isoline_v(u);
                let mut points = Vec::with_capacity(steps_v);
                for t in &isoline_v_t_vals[..] {
                    points.push(curve.point(*t));
                }
                isolines_v_vbos.push(VertexBuffer::new(display, &points[..]).unwrap());
            }
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
                      greville_u_vbos: greville_u_vbos,
                      greville_v_vbos: greville_v_vbos,
                      knot_u_vbos: knot_u_vbos,
                      knot_v_vbos: knot_v_vbos,
                      control_points_vbo: control_points_vbo,
                      draw_surf: true,
                      draw_greville: true,
                      draw_knots: true,
                      draw_control_points: true,
                      curve_color: [0.8, 0.8, 0.1],
                      greville_color: [0.1, 0.8, 0.8],
                      knot_color: [0.8, 0.1, 0.8],
                      control_color: [0.8, 0.8, 0.8],
        }
    }
    pub fn render<S: Surface>(&self, target: &mut S, program: &Program, draw_params: &DrawParameters,
                  proj_view: &[[f32; 4]; 4], selected: bool, attenuation: f32) {
        let (curve_color, control_color, greville_color, knot_color) =
            if selected {
                (self.curve_color, self.control_color, self.greville_color, self.knot_color)
            } else {
                ([attenuation * self.curve_color[0], attenuation * self.curve_color[1],
                  attenuation * self.curve_color[2]],

                 [attenuation * self.control_color[0], attenuation * self.control_color[1],
                  attenuation * self.control_color[2]],

                 [attenuation * self.greville_color[0], attenuation * self.greville_color[1],
                  attenuation * self.greville_color[2]],

                 [attenuation * self.knot_color[0], attenuation * self.knot_color[1],
                  attenuation * self.knot_color[2]])
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
            pcolor: greville_color,
        };
        if self.draw_greville {
            for iso in self.greville_u_vbos.iter().chain(self.greville_v_vbos.iter()) {
                target.draw(iso, &NoIndices(PrimitiveType::LineStrip),
                            &program, &uniforms, &draw_params).unwrap();
            }
        }
        let uniforms = uniform! {
            proj_view: *proj_view,
            pcolor: knot_color,
        };
        if self.draw_knots {
            for iso in self.knot_u_vbos.iter().chain(self.knot_v_vbos.iter()) {
                target.draw(iso, &NoIndices(PrimitiveType::LineStrip),
                            &program, &uniforms, &draw_params).unwrap();
            }
        }
        let uniforms = uniform! {
            proj_view: *proj_view,
            pcolor: control_color,
        };
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
        ui.checkbox(im_str!("Draw Greville Isolines"), &mut self.draw_greville);
        ui.checkbox(im_str!("Draw Knot Isolines"), &mut self.draw_knots);
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
        ui.color_edit3(im_str!("Greville Color"), &mut self.greville_color).build();
        ui.color_edit3(im_str!("Knot Color"), &mut self.knot_color).build();
        ui.color_edit3(im_str!("Control Color"), &mut self.control_color).build();
    }
}


