use std::fmt::Debug;
use std::iter;
use std::slice;

use bezier::Interpolate;
use bspline::BSpline;

/// Represents a B-spline surface that will use polynomials of the
/// specified degree along u and v to to interpolate the control mesh
/// using the knots along u and v.
#[derive(Clone, Debug)]
pub struct BSplineSurf<T> {
    degree_u: usize,
    degree_v: usize,
    pub knots_u: Vec<f32>,
    pub knots_v: Vec<f32>,
    pub control_mesh: Vec<Vec<T>>,
}


impl<T: Interpolate + Copy + Debug> BSplineSurf<T> {
    /// Make a new tensor product B-spline surface. The surface will be the product
    /// of a degree.0, knots.0 and degree.1, knots.1 B-spline using the control mesh.
    pub fn new(degree: (usize, usize), knots: (Vec<f32>, Vec<f32>), control_mesh: Vec<Vec<T>>) -> BSplineSurf<T> {
        if control_mesh.is_empty() {
            panic!("Surface control mesh cannot be empty!");
        }
        // TODO: Validate params
        BSplineSurf { degree_u: degree.0, degree_v: degree.1,
                      knots_u: knots.0, knots_v: knots.1,
                      control_mesh: control_mesh
                    }
    }
    /// Get the u curve degree
    pub fn degree_u(&self) -> usize {
        self.degree_u
    }
    /// Get the v curve degree
    pub fn degree_v(&self) -> usize {
        self.degree_v
    }
    /// Get the min and max knot domain values for finding the `t` range to compute
    /// the curve over in the u axis. The curve is only defined over the inclusive range `[min, max]`,
    /// passing a `t` value outside of this range will result in an assert on debug builds
    /// and likely a crash on release builds.
    pub fn knot_domain_u(&self) -> (f32, f32) {
        (self.knots_u[self.degree_u], self.knots_u[self.knots_u.len() - 1 - self.degree_u])
    }
    /// Get an iterator over the knots within the domain
    pub fn knot_domain_u_iter(&self) -> iter::Take<iter::Skip<slice::Iter<f32>>> {
        self.knots_u.iter().skip(self.degree_u).take(self.knots_u.len() - 2 * self.degree_u)
    }
    pub fn greville_abscissa_u(&self) -> Vec<f32> {
        let mut abscissa = Vec::with_capacity(self.control_mesh.len());
        let domain = self.knot_domain_u();
        for i in 0..self.control_mesh.len() {
            let g = self.knots_u.iter().enumerate().skip_while(|&(c, ref x)| c < i + 1)
                .take_while(|&(c, ref x)| c <= i + self.degree_u)
                .map(|(c, x)| x)
                .fold(0.0, |acc, x| acc + *x) / self.degree_u as f32;
            // TODO: Shouldn't this not be necessary? How can I get an abscissa outside
            // the knot domain?
            if g >= domain.0 && g <= domain.1 {
                abscissa.push(g);
            }
        }
        abscissa
    }
    /// Get a vector with the Greville abscissa for the u axis
    /// Get the min and max knot domain values for finding the `t` range to compute
    /// the curve over in the v axis. The curve is only defined over the inclusive range `[min, max]`,
    /// passing a `t` value outside of this range will result in an assert on debug builds
    /// and likely a crash on release builds.
    pub fn knot_domain_v(&self) -> (f32, f32) {
        (self.knots_v[self.degree_v], self.knots_v[self.knots_v.len() - 1 - self.degree_v])
    }
    pub fn knot_domain_v_iter(&self) -> iter::Take<iter::Skip<slice::Iter<f32>>> {
        self.knots_v.iter().skip(self.degree_v).take(self.knots_v.len() - 2 * self.degree_v)
    }
    /// Get a vector with the Greville abscissa for the v axis
    pub fn greville_abscissa_v(&self) -> Vec<f32> {
        let mut abscissa = Vec::with_capacity(self.control_mesh[0].len());
        let domain = self.knot_domain_v();
        for i in 0..self.control_mesh[0].len() {
            let g = self.knots_v.iter().enumerate().skip_while(|&(c, ref x)| c < i + 1)
                .take_while(|&(c, ref x)| c <= i + self.degree_v)
                .map(|(c, x)| x)
                .fold(0.0, |acc, x| acc + *x) / self.degree_v as f32;
            // TODO: Shouldn't this not be necessary? How can I get an abscissa outside
            // the knot domain?
            if g >= domain.0 && g <= domain.1 {
                abscissa.push(g);
            }
        }
        abscissa
    }
    /// Compute an isoline along v for a fixed value of u
    pub fn isoline_v(&self, u: f32) -> BSpline<T> {
        // Build and evaluate B-splines for each column of the control mesh to build the control
        // points for the isoline.
        let mut isoline_ctrl_pts = Vec::with_capacity(self.control_mesh.len());
        for j in 0..self.control_mesh[0].len() {
            let mut column = Vec::with_capacity(self.control_mesh.len());
            for i in 0..self.control_mesh.len() {
                column.push(self.control_mesh[i][j]);
            }
            // Build the column of points
            let spline = BSpline::new(self.degree_u, column, self.knots_u.clone());
            isoline_ctrl_pts.push(spline.point(u));
        }
        BSpline::new(self.degree_v, isoline_ctrl_pts, self.knots_v.clone())
    }
    /// Compute an isoline along u for a fixed value of v
    pub fn isoline_u(&self, v: f32) -> BSpline<T> {
        // Build and evaluate B-splines for each row of the control mesh to build the control
        // points for the isoline.
        let mut isoline_ctrl_pts = Vec::with_capacity(self.control_mesh.len());
        for i in 0..self.control_mesh.len() {
            let spline = BSpline::new(self.degree_v, self.control_mesh[i].clone(), self.knots_v.clone());
            isoline_ctrl_pts.push(spline.point(v));
        }
        BSpline::new(self.degree_u, isoline_ctrl_pts, self.knots_u.clone())
    }
}

