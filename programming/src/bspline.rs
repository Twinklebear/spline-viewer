use std::fmt::Debug;
use std::slice::Iter;
use std::f32;

use bezier::{Interpolate, ProjectToSegment};

/// Represents a B-spline curve that will use polynomials of the specified degree
/// to interpolate between the control points given the knots.
#[derive(Clone, Debug)]
pub struct BSpline<T: Interpolate + Copy> {
    /// Degree of the polynomial that we use to make the curve segments
    pub degree: usize,
    /// Control points for the curve
    pub control_points: Vec<T>,
    /// The knot vector
    pub knots: Vec<f32>,
}

impl<T: Interpolate + Copy> BSpline<T> {
    /// Create a new B-spline curve of the desired `degree` that will interpolate
    /// the `control_points` using the `knots`. The knots should be sorted in non-decreasing
    /// order otherwise they will be sorted for you, which may lead to undesired knots
    /// for control points. Note that here we use the interpolating polynomial degree,
    /// if you're familiar with the convention of "B-spline curve order" the degree is `curve_order - 1`.
    ///
    /// Your curve must have a valid number of control points and knots or the function will panic. A B-spline
    /// curve requires at least as one more control point than the degree (`control_points.len() >
    /// degree`) and the number of knots should be equal to `control_points.len() + degree + 1`.
    pub fn new(degree: usize, control_points: Vec<T>, mut knots: Vec<f32>) -> BSpline<T> {
        if control_points.len() <= degree {
            panic!("Too few control points for curve");
        }
        if knots.len() != control_points.len() + degree + 1 {
            panic!(format!("Invalid number of knots, got {}, expected {}", knots.len(),
                control_points.len() + degree + 1));
        }
        knots.sort_by(|a, b| a.partial_cmp(b).unwrap());
        BSpline { degree: degree, control_points: control_points, knots: knots }
    }
    /// Compute a point on the curve at `t`, the parameter **must** be in the inclusive range
    /// of values returned by `knot_domain`. If `t` is out of bounds this function will assert
    /// on debug builds and on release builds you'll likely get an out of bounds crash.
    pub fn point(&self, t: f32) -> T {
        debug_assert!(t >= self.knot_domain().0 && t <= self.knot_domain().1);
        // Find the first index with a knot value greater than the t we're searching for. We want
        // to find i such that: knot[i] <= t < knot[i + 1]
        let i = match upper_bounds(&self.knots[..], t) {
            Some(x) if x == 0 => self.degree,
            Some(x) if x >= self.knots.len() - self.degree - 1 =>
                self.knots.len() - self.degree - 1,
            Some(x) => x,
            None => self.knots.len() - self.degree - 1,
        };
        self.de_boor_iterative(t, i)
    }
    /// Get an iterator over the control points.
    pub fn control_points(&self) -> Iter<T> {
        self.control_points.iter()
    }
    /// Get an iterator over the knots.
    pub fn knots(&self) -> Iter<f32> {
        self.knots.iter()
    }
    /// Get the min and max knot domain values for finding the `t` range to compute
    /// the curve over. The curve is only defined over the inclusive range `[min, max]`,
    /// passing a `t` value outside of this range will result in an assert on debug builds
    /// and likely a crash on release builds.
    pub fn knot_domain(&self) -> (f32, f32) {
        (self.knots[self.degree], self.knots[self.knots.len() - 1 - self.degree])
    }
    /// Iteratively compute de Boor's B-spline algorithm, this computes the recursive
    /// de Boor algorithm tree from the bottom up. At each level we use the results
    /// from the previous one to compute this level and store the results in the
    /// array indices we no longer need to compute the current level (the left one
    /// used computing node j).
    fn de_boor_iterative(&self, t: f32, i_start: usize) -> T {
        let mut tmp = Vec::with_capacity(self.degree + 1);
        for j in 0..self.degree + 1 {
            let p = j + i_start - self.degree - 1;
            tmp.push(self.control_points[p]);
        }
        for lvl in 0..self.degree {
            let k = lvl + 1;
            for j in 0..self.degree - lvl {
                let i = j + k + i_start - self.degree;
                let alpha = (t - self.knots[i - 1]) / (self.knots[i + self.degree - k] - self.knots[i - 1]);
                debug_assert!(!alpha.is_nan());
                tmp[j] = tmp[j].interpolate(&tmp[j + 1], alpha);
            }
        }
        tmp[0]
    }
}

/// Return the index of the first element greater than the value passed.
/// The data **must** be sorted. If no element greater than the value
/// passed is found the function returns None.
fn upper_bounds(data: &[f32], value: f32) -> Option<usize> {
    let mut first = 0usize;
    let mut step;
    let mut count = data.len() as isize;
    while count > 0 {
        step = count / 2;
        let it = first + step as usize;
        if !value.lt(&data[it]) {
            first = it + 1;
            count -= step + 1;
        } else {
            count = step;
        }
    }
    // If we didn't find an element greater than value
    if first == data.len() {
        None
    } else {
        Some(first)
    }
}

