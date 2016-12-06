/// Manages displaying and toggling interaction modes with
/// a specific BSpline surface in the scene.

use std::f32;
use std::iter::FromIterator;
use std::ops::Index;

use glium::{Surface, VertexBuffer, Program, DrawParameters};
use glium::backend::Facade;
use glium::index::{NoIndices, PrimitiveType};
use imgui::Ui;
use rulinalg::matrix::{Matrix, BaseMatrix};
use rulinalg::vector::Vector;

use bspline::BSpline;
use bspline_surf::BSplineSurf;
use bspline_basis::BSplineBasis;
use display_surf::DisplaySurf;
use point::Point;

pub struct DisplaySurfInterpolation<'a, F: 'a + Facade> {
    display: &'a F,
    curves: Vec<BSpline<Point>>,
    surf: DisplaySurf<'a, F>,
    // The input curves
    input_curves_vbo: Vec<VertexBuffer<Point>>,
    // The input control points
    input_points_vbo: VertexBuffer<Point>,
    draw_input_curves: bool,
    draw_input_points: bool,
    curve_color: [f32; 3],
}

impl<'a, F: 'a + Facade> DisplaySurfInterpolation<'a, F> {
    pub fn new(curves: Vec<BSpline<Point>>, display: &'a F) -> DisplaySurfInterpolation<'a, F> {
        let mut control_points = Vec::new();
        let mut input_curves_vbo = Vec::with_capacity(curves.len());
        let step_size = 0.01;
        for (i, c) in curves.iter().enumerate() {
            let t_range = c.knot_domain();
            let steps = ((t_range.1 - t_range.0) / step_size) as usize;
            let mut points = Vec::with_capacity(steps);
            // Just draw the first one for now
            for s in 0..steps + 1 {
                let t = step_size * s as f32 + t_range.0;
                points.push(c.point(t));
            }
            println!("--------");
            input_curves_vbo.push(VertexBuffer::new(display, &points[..]).unwrap());

            for pt in &c.control_points[..] {
                control_points.push(*pt);
            }
        }
        let control_points_vbo = VertexBuffer::new(display, &control_points[..]).unwrap();
        let surf = compute_nodal_interpolation(&curves[..], 1);

        DisplaySurfInterpolation { display: display,
                      curves: curves,
                      surf: DisplaySurf::new(surf, display),
                      input_curves_vbo: input_curves_vbo,
                      input_points_vbo: control_points_vbo,
                      draw_input_curves: true,
                      draw_input_points: true,
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
        if self.draw_input_points {
            target.draw(&self.input_points_vbo, &NoIndices(PrimitiveType::Points),
                        &program, &uniforms, &draw_params).unwrap();
        }
        self.surf.render(target, program, draw_params, proj_view, selected, attenuation);
    }
    pub fn draw_ui(&mut self, ui: &Ui) {
        ui.text(im_str!("3D Surface Interpolation"));
        ui.checkbox(im_str!("Draw Input Curves"), &mut self.draw_input_curves);
        ui.checkbox(im_str!("Draw Input Control Points"), &mut self.draw_input_points);
        ui.color_edit3(im_str!("Input Color"), &mut self.curve_color).build();
        self.surf.draw_ui(ui);
    }
}

fn compute_nodal_interpolation(curves: &[BSpline<Point>], degree: usize) -> BSplineSurf<Point> {
    let mut control_points = Vec::new();
    for c in curves.iter() {
        for pt in &c.control_points[..] {
            control_points.push(*pt);
        }
    }
    // Setup the bases for u and v so we can build the matrices
    let basis_u = BSplineBasis::new(curves[0].degree(), curves[0].knots().map(|x| *x).collect());
    let abscissa_u = basis_u.greville_abscissa();
    // Is the result right for cubic?
    let basis_v = BSplineBasis::clamped_uniform(1, curves.len());
    let abscissa_v = basis_v.greville_abscissa();

    // This is actually the N matrix in the 12/5 notes.
    let f = Matrix::from_fn(curves.len(), abscissa_v.len(),
                            |i, j| basis_v.eval(abscissa_v[i], j));
    let x_pos: Vec<_> = control_points.iter().map(|x| x.pos[0]).collect();
    let y_pos: Vec<_> = control_points.iter().map(|x| x.pos[1]).collect();
    let z_pos: Vec<_> = control_points.iter().map(|x| x.pos[2]).collect();
    // Are these dimensions right?
    let r_mats = vec![Matrix::new(curves.len(), curves[0].control_points.len(), x_pos),
                 Matrix::new(curves.len(), curves[0].control_points.len(), y_pos),
                 Matrix::new(curves.len(), curves[0].control_points.len(), z_pos)];
    let mut result_mats = Vec::with_capacity(r_mats.len());

    // Solve each column
    let mut x = 0;
    for r in &r_mats[..] {
        let mut res_mat = Matrix::zeros(curves.len(), curves[0].control_points.len());
        for j in 0..curves[0].control_points.len() {
            let rhs = Vector::new((0..curves.len()).map(|i| r[[i, j]]).collect::<Vec<f32>>());
            let result = f.solve(rhs).expect("System could not be solved!?");
            for i in 0..curves.len() {
                res_mat[[i, j]] = result[i];
            }
        }
        result_mats.push(res_mat);
        x = x + 1;
    }
    let mut surf_mesh = Vec::new();
    for i in 0..curves.len() {
        let mut mesh_row = Vec::new();
        for j in 0..curves[0].control_points.len() {
            let p = Point::new(result_mats[0][[i, j]], result_mats[1][[i, j]], result_mats[2][[i, j]]);
            mesh_row.push(p);
        }
        surf_mesh.push(mesh_row);
    }
    BSplineSurf::new((basis_v.degree(), basis_u.degree()), (basis_v.knots, basis_u.knots), surf_mesh)
}

