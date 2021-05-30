use std::f32;

/// Just the basis functions for a B-spline, can return the B-spline
/// basis function values for specific basis functions at desired t values
pub struct BSplineBasis {
    /// Degree of the polynomial that we use to make the curve segments
    degree: usize,
    /// The knot vector
    pub knots: Vec<f32>,
    modified_knot: usize,
}

impl BSplineBasis {
    pub fn new(degree: usize, mut knots: Vec<f32>) -> BSplineBasis {
        knots.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mut modified_knot = 0;
        for i in 0..knots.len() - 1 {
            if knots[i] < knots[i + 1] {
                modified_knot = i;
            }
        }
        BSplineBasis {
            degree: degree,
            knots: knots,
            modified_knot: modified_knot,
        }
    }
    /// Make a new basis with a generated uniform clamped knot vector
    pub fn clamped_uniform(degree: usize, num_points: usize) -> BSplineBasis {
        let knots = BSplineBasis::generate_knot_vector(true, num_points + degree + 1, degree);
        let mut modified_knot = 0;
        for i in 0..knots.len() - 1 {
            if knots[i] < knots[i + 1] {
                modified_knot = i;
            }
        }
        BSplineBasis {
            degree: degree,
            knots: knots,
            modified_knot: modified_knot,
        }
    }
    /// Get the curve degree
    pub fn degree(&self) -> usize {
        self.degree
    }
    pub fn knot_domain(&self) -> (f32, f32) {
        (
            self.knots[self.degree],
            self.knots[self.knots.len() - 1 - self.degree],
        )
    }
    pub fn greville_abscissa(&self) -> Vec<f32> {
        let num_abscissa = self.knots.len() - self.degree - 1;
        let mut abscissa = Vec::with_capacity(num_abscissa);
        let domain = self.knot_domain();
        for i in 0..num_abscissa {
            let g = self
                .knots
                .iter()
                .enumerate()
                .skip_while(|&(c, _)| c < i + 1)
                .take_while(|&(c, _)| c <= i + self.degree)
                .map(|(_, x)| x)
                .fold(0.0, |acc, x| acc + *x)
                / self.degree as f32;
            // TODO: Shouldn't this not be necessary? How can I get an abscissa outside
            // the knot domain?
            if g >= domain.0 && g <= domain.1 {
                abscissa.push(g);
            }
        }
        abscissa
    }
    pub fn eval(&self, t: f32, fcn: usize) -> f32 {
        debug_assert!(t >= self.knot_domain().0 && t <= self.knot_domain().1);
        self.evaluate_basis(t, fcn, self.degree)
    }
    /// TODO: Make this fucking work.
    fn evaluate_basis(&self, t: f32, i: usize, k: usize) -> f32 {
        if k == 0 {
            if t >= self.knots[i] {
                // Modified open end condition
                if i == self.modified_knot && t <= self.knots[i + 1] {
                    1.0
                } else if t < self.knots[i + 1] {
                    1.0
                } else {
                    0.0
                }
            } else {
                0.0
            }
        } else if t >= self.knots[i] && t <= self.knots[i + k + 1] {
            let mut a = (t - self.knots[i]) / (self.knots[i + k] - self.knots[i]);
            let mut b = (self.knots[i + k + 1] - t) / (self.knots[i + k + 1] - self.knots[i + 1]);
            if !a.is_finite() {
                a = 0.0;
            }
            if !b.is_finite() {
                b = 0.0;
            }
            let c = self.evaluate_basis(t, i, k - 1);
            let d = self.evaluate_basis(t, i + 1, k - 1);
            a * c + b * d
        } else {
            0.0
        }
    }
    /// Fill the knot vector for this curve for the new number of points/degree
    fn generate_knot_vector(clamped: bool, knots_required: usize, degree: usize) -> Vec<f32> {
        let mut knots = Vec::with_capacity(knots_required);
        let mut x = 0.0;
        for i in 0..knots_required {
            knots.push(x);
            if !(clamped && i < degree) && !(clamped && i >= knots_required - 1 - degree) {
                x += 1.0;
            }
        }
        knots
    }
}
