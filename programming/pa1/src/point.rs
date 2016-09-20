use std::ops::{Mul, Add, Sub, Div};
use std::f32;

use bezier::ProjectToSegment;

pub fn clamp(x: f32, min: f32, max: f32) -> f32 {
    if x < min {
        min
    } else if x > max {
        max
    } else {
        x
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Point {
    pub pos: [f32; 2],
}
impl Point {
    pub fn new(x: f32, y: f32) -> Point {
        Point { pos: [x, y] }
    }
    pub fn dot(&self, a: &Point) -> f32 {
        self.pos[0] * a.pos[0] + self.pos[1] * a.pos[1]
    }
    pub fn length(&self) -> f32 {
        f32::sqrt(self.dot(&self))
    }
}
implement_vertex!(Point, pos);

impl Mul<f32> for Point {
    type Output = Point;
    fn mul(self, rhs: f32) -> Point {
        Point { pos: [self.pos[0] * rhs, self.pos[1] * rhs] }
    }
}
impl Div<f32> for Point {
    type Output = Point;
    fn div(self, rhs: f32) -> Point {
        Point { pos: [self.pos[0] / rhs, self.pos[1] / rhs] }
    }
}
impl Add for Point {
    type Output = Point;
    fn add(self, rhs: Point) -> Point {
        Point { pos: [self.pos[0] + rhs.pos[0], self.pos[1] + rhs.pos[1]] }
    }
}
impl Sub for Point {
    type Output = Point;
    fn sub(self, rhs: Point) -> Point {
        Point { pos: [self.pos[0] - rhs.pos[0], self.pos[1] - rhs.pos[1]] }
    }
}
impl ProjectToSegment for Point {
    fn project(&self, a: &Point, b: &Point) -> (f32, f32) {
        let v = *b - *a;
        let dir = *self - *a;
        let t = clamp(dir.dot(&v) / v.dot(&v), 0.0, 1.0);
        let p = *a + v * t;
        let d = (p - *self).length();
        (d, t)
    }
}

