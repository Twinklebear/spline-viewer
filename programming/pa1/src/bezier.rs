//! This module provides functionality for computing a Bezier curve
//! defined by a set of control points on any type that can be linearly interpolated.

use std::ops::{Mul, Add};
use std::slice::Iter;

/// The interpolate trait is used to linearly interpolate between two types (or in the
/// case of Quaternions, spherically linearly interpolate). The B-spline curve uses this
/// trait to compute points on the curve for the given parameter value.
///
/// A default implementation of this trait is provided for all `T` that are `Mul<f32, Output = T>
/// + Add<Output = T> + Copy` as these are the only operations needed to linearly interpolate the
/// values. Any type implementing this trait should perform whatever the appropriate linear
/// interpolaton is for the type.
pub trait Interpolate {
    /// Linearly interpolate between `self` and `other` using `t`, for example with floats:
    ///
    /// ```text
    /// self * (1.0 - t) + other * t
    /// ```
    ///
    /// If the result returned is not a correct linear interpolation of the values the
    /// curve produced using the value may not be correct.
    fn interpolate(&self, other: &Self, t: f32) -> Self;
}

impl<T: Mul<f32, Output = T> + Add<Output = T> + Copy> Interpolate for T {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        *self * (1.0 - t) + *other * t
    }
}

/// Represents a Bezier curve that will use polynomials of the specified degree
/// to interpolate between the control points given the knots.
#[derive(Clone, Debug)]
pub struct Bezier<T: Interpolate + Copy> {
    /// Control points for the curve
    control_points: Vec<T>,
}

impl<T: Interpolate + Copy> Bezier<T> {
    /// Create a new Bezier curve of formed by interpolating the `control_points`
    pub fn new(control_points: Vec<T>) -> Bezier<T> {
        Bezier { control_points: control_points }
    }
    /// Compute a point on the curve at `t`, the parameter **must** be in the inclusive
    /// range [0, 1]. If `t` is out of bounds this function will assert
    /// on debug builds and on release builds you'll likely get an out of bounds crash.
    pub fn point(&self, t: f32) -> T {
        debug_assert!(t >= 0.0 && t <= 1.0);
        self.de_casteljau(t, self.control_points.len() - 1, 0)
    }
    /// Get an iterator over the control points.
    pub fn control_points(&self) -> Iter<T> {
        self.control_points.iter()
    }
    /// Recursively use de Casteljau's algorithm to compute the desired point
    fn de_casteljau(&self, t: f32, r: usize, i: usize) -> T {
        if r == 0 {
            self.control_points[i]
        } else {
            let a = self.de_casteljau(t, r - 1, i);
            let b = self.de_casteljau(t, r - 1, i + 1);
            a.interpolate(&b, t)
        }
    }
}

