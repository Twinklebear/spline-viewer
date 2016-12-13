use std::fmt::Debug;
use std::slice::Iter;
use std::f32;
use std::iter;
use std::slice;

use bezier::{Interpolate, ProjectToSegment};

/// Represents a B-spline curve that will use polynomials of the specified degree
/// to interpolate between the control points given the knots.
#[derive(Clone, Debug)]
pub struct BSpline<T> {
    /// Degree of the polynomial that we use to make the curve segments
    degree: usize,
    /// Control points for the curve
    pub control_points: Vec<T>,
    /// The knot vector
    knots: Vec<f32>,
}

impl<T: Interpolate + Copy + Debug> BSpline<T> {
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
        if !knots.is_empty() && knots.len() != control_points.len() + degree + 1 {
            panic!(format!("Invalid number of knots, got {}, expected {}", knots.len(),
                control_points.len() + degree + 1));
        }
        knots.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mut spline = BSpline { degree: degree, control_points: control_points, knots: knots };
        if spline.knots.is_empty() {
            spline.fill_knot_vector(true, true);
        }
        spline
    }
    /// Create a new empty BSpline.
    pub fn empty() -> BSpline<T> {
        BSpline { degree: 0, control_points: Vec::new(), knots: Vec::new() }
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
    /// Get the curve degree
    pub fn degree(&self) -> usize {
        self.degree
    }
    /// Get the min and max knot domain values for finding the `t` range to compute
    /// the curve over. The curve is only defined over the inclusive range `[min, max]`,
    /// passing a `t` value outside of this range will result in an assert on debug builds
    /// and likely a crash on release builds.
    pub fn knot_domain(&self) -> (f32, f32) {
        (self.knots[self.degree], self.knots[self.knots.len() - 1 - self.degree])
    }
    /// Get an iterator over the knots within the domain
    pub fn knot_domain_iter(&self) -> iter::Take<iter::Skip<slice::Iter<f32>>> {
        self.knots.iter().skip(self.degree).take(self.knots.len() - 2 * self.degree)
    }
    /// Get the max degree of curve that this set of control points can support
    pub fn max_possible_degree(&self) -> usize {
        if self.control_points.is_empty() {
            0
        } else {
            self.control_points.len() - 1
        }
    }
    /// Change the degree of the curve
    pub fn set_degree(&mut self, degree: usize) {
        assert!(degree <= self.max_possible_degree());
        let was_clamped = self.is_clamped();
        self.degree = degree;
        self.fill_knot_vector(was_clamped, was_clamped);
    }
    /// Remove a point from the curve
    pub fn remove_point(&mut self, i: usize) {
        self.control_points.remove(i);
        if self.control_points.len() <= self.degree {
            self.degree -= 1;
        }
        self.generate_knot_vector();
    }
    /// Toggle whether the curve should be open/clamped (Elaine: floating/open)
    pub fn set_clamped(&mut self, clamped: bool) {
        self.fill_knot_vector(clamped, clamped);
    }
    pub fn is_clamped(&self) -> bool {
        let left_clamped = self.knots.iter().take(self.degree + 1)
            .fold(true, |acc, x| *x == self.knots[0] && acc);
        let right_clamped = self.knots.iter().rev().take(self.degree + 1)
            .fold(true, |acc, x| *x == self.knots[self.knots.len() - 1] && acc);
        left_clamped && right_clamped
    }
    /// Compute the number of knots required for this curve
    fn knots_required(&self) -> usize {
        self.control_points.len() + self.degree + 1
    }
    /// Regenerate the knot vector to update it for changing degree/control points based on
    /// whether it was open/clamped before (Elaine: terms floating/open)
    fn generate_knot_vector(&mut self) {
        // Check if we're clamped on the left/right (Elaine calls this end condition open)
        let left_clamped = self.knots.iter().take(self.degree + 1)
            .fold(true, |acc, x| *x == self.knots[0] && acc);
        let right_clamped = self.knots.iter().rev().take(self.degree + 1)
            .fold(true, |acc, x| *x == self.knots[self.knots.len() - 1] && acc);
        self.fill_knot_vector(left_clamped, right_clamped);
    }
    /// Fill the knot vector for this curve for the new number of points/degree
    fn fill_knot_vector(&mut self, left_clamped: bool, right_clamped: bool) {
        self.knots.clear();
        let mut x = 0.0;
        for i in 0..self.knots_required() {
            self.knots.push(x);
            if !(left_clamped && i < self.degree)
                && !(right_clamped && i >= self.knots_required() - 1 - self.degree) {
                    x += 1.0;
                }
        }
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

impl<T: Interpolate + ProjectToSegment + Copy + Debug> BSpline<T> {
    /// Insert a new point into the curve. The point will be inserted near the existing
    /// control points that it's closest too. Returns the index the point was
    /// inserted at.
    pub fn insert_point(&mut self, t: T) -> usize {
        if self.control_points.len() == 1 {
            self.control_points.push(t);
            return 1;
        }
        // Go through all segments of the control polygon and find the nearest one
        let nearest = self.control_points.windows(2).enumerate()
            .map(|(i, x)| {
                let proj = t.project(&x[0], &x[1]);
                (i, proj.0, proj.1)
            })
            .fold((0, f32::MAX, 0.0), |acc, (i, d, l)| {
                if d < acc.1 {
                    (i, d, l)
                } else {
                    acc
                }
            });
        // Check if we're appending or prepending the point
        let idx = if nearest.0 == 0 && nearest.2 == 0.0 {
            self.control_points.insert(0, t);
            0
        } else if nearest.0 == self.control_points.len() - 2 && nearest.2 == 1.0 {
            self.control_points.push(t);
            self.control_points.len() - 1
        } else {
            self.control_points.insert(nearest.0 + 1, t);
            nearest.0 + 1
        };
        self.generate_knot_vector();
        idx
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

