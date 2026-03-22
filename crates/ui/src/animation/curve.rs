//! Animation Curves (Easing Functions)
//!
//! This module provides various easing functions that control
//! the rate of change of an animation over time.
//!
//! # Usage
//!
//! ```ignore
//! use hoshimi_ui::animation::Curve;
//!
//! let curve = Curve::EaseInOut;
//! let value = curve.transform(0.5); // Returns eased value for t=0.5
//! ```

use std::f32::consts::PI;

/// Animation curve that transforms a linear 0.0-1.0 progress value
/// into an eased value.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Curve {
    /// Linear interpolation (no easing)
    #[default]
    Linear,

    /// Quadratic ease-in (slow start)
    EaseIn,

    /// Quadratic ease-out (slow end)
    EaseOut,

    /// Quadratic ease-in-out (slow start and end)
    EaseInOut,

    /// Cubic ease-in
    EaseInCubic,

    /// Cubic ease-out
    EaseOutCubic,

    /// Cubic ease-in-out
    EaseInOutCubic,

    /// Quartic ease-in
    EaseInQuart,

    /// Quartic ease-out
    EaseOutQuart,

    /// Quartic ease-in-out
    EaseInOutQuart,

    /// Quintic ease-in
    EaseInQuint,

    /// Quintic ease-out
    EaseOutQuint,

    /// Quintic ease-in-out
    EaseInOutQuint,

    /// Sinusoidal ease-in
    EaseInSine,

    /// Sinusoidal ease-out
    EaseOutSine,

    /// Sinusoidal ease-in-out
    EaseInOutSine,

    /// Exponential ease-in
    EaseInExpo,

    /// Exponential ease-out
    EaseOutExpo,

    /// Exponential ease-in-out
    EaseInOutExpo,

    /// Circular ease-in
    EaseInCirc,

    /// Circular ease-out
    EaseOutCirc,

    /// Circular ease-in-out
    EaseInOutCirc,

    /// Back ease-in (overshoots at start)
    EaseInBack,

    /// Back ease-out (overshoots at end)
    EaseOutBack,

    /// Back ease-in-out (overshoots at both ends)
    EaseInOutBack,

    /// Elastic ease-in (bouncy at start)
    EaseInElastic,

    /// Elastic ease-out (bouncy at end)
    EaseOutElastic,

    /// Elastic ease-in-out (bouncy at both ends)
    EaseInOutElastic,

    /// Bounce ease-in
    EaseInBounce,

    /// Bounce ease-out
    EaseOutBounce,

    /// Bounce ease-in-out
    EaseInOutBounce,

    /// Custom cubic Bezier curve
    #[doc(hidden)]
    CubicBezier {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
    },
}

impl Curve {
    /// Transform a linear progress value (0.0 to 1.0) to an eased value
    ///
    /// # Arguments
    /// * `t` - Progress value between 0.0 and 1.0
    ///
    /// # Returns
    /// The eased value (may exceed 0.0-1.0 range for some curves)
    pub fn transform(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);

        match self {
            Curve::Linear => t,

            // Quadratic
            Curve::EaseIn => t * t,
            Curve::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            Curve::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }

            // Cubic
            Curve::EaseInCubic => t * t * t,
            Curve::EaseOutCubic => 1.0 - (1.0 - t).powi(3),
            Curve::EaseInOutCubic => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }

            // Quartic
            Curve::EaseInQuart => t * t * t * t,
            Curve::EaseOutQuart => 1.0 - (1.0 - t).powi(4),
            Curve::EaseInOutQuart => {
                if t < 0.5 {
                    8.0 * t * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(4) / 2.0
                }
            }

            // Quintic
            Curve::EaseInQuint => t * t * t * t * t,
            Curve::EaseOutQuint => 1.0 - (1.0 - t).powi(5),
            Curve::EaseInOutQuint => {
                if t < 0.5 {
                    16.0 * t * t * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(5) / 2.0
                }
            }

            // Sinusoidal
            Curve::EaseInSine => 1.0 - (t * PI / 2.0).cos(),
            Curve::EaseOutSine => (t * PI / 2.0).sin(),
            Curve::EaseInOutSine => -(((t * PI).cos() - 1.0) / 2.0),

            // Exponential
            Curve::EaseInExpo => {
                if t == 0.0 {
                    0.0
                } else {
                    (2.0_f32).powf(10.0 * t - 10.0)
                }
            }
            Curve::EaseOutExpo => {
                if t == 1.0 {
                    1.0
                } else {
                    1.0 - (2.0_f32).powf(-10.0 * t)
                }
            }
            Curve::EaseInOutExpo => {
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else if t < 0.5 {
                    (2.0_f32).powf(20.0 * t - 10.0) / 2.0
                } else {
                    (2.0 - (2.0_f32).powf(-20.0 * t + 10.0)) / 2.0
                }
            }

            // Circular
            Curve::EaseInCirc => 1.0 - (1.0 - t * t).sqrt(),
            Curve::EaseOutCirc => (1.0 - (t - 1.0).powi(2)).sqrt(),
            Curve::EaseInOutCirc => {
                if t < 0.5 {
                    (1.0 - (1.0 - (2.0 * t).powi(2)).sqrt()) / 2.0
                } else {
                    ((1.0 - (-2.0 * t + 2.0).powi(2)).sqrt() + 1.0) / 2.0
                }
            }

            // Back (overshoot)
            Curve::EaseInBack => {
                const C1: f32 = 1.70158;
                const C3: f32 = C1 + 1.0;
                C3 * t * t * t - C1 * t * t
            }
            Curve::EaseOutBack => {
                const C1: f32 = 1.70158;
                const C3: f32 = C1 + 1.0;
                1.0 + C3 * (t - 1.0).powi(3) + C1 * (t - 1.0).powi(2)
            }
            Curve::EaseInOutBack => {
                const C1: f32 = 1.70158;
                const C2: f32 = C1 * 1.525;
                if t < 0.5 {
                    ((2.0 * t).powi(2) * ((C2 + 1.0) * 2.0 * t - C2)) / 2.0
                } else {
                    ((2.0 * t - 2.0).powi(2) * ((C2 + 1.0) * (t * 2.0 - 2.0) + C2) + 2.0) / 2.0
                }
            }

            // Elastic
            Curve::EaseInElastic => {
                const C4: f32 = (2.0 * PI) / 3.0;
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else {
                    -(2.0_f32).powf(10.0 * t - 10.0) * ((t * 10.0 - 10.75) * C4).sin()
                }
            }
            Curve::EaseOutElastic => {
                const C4: f32 = (2.0 * PI) / 3.0;
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else {
                    (2.0_f32).powf(-10.0 * t) * ((t * 10.0 - 0.75) * C4).sin() + 1.0
                }
            }
            Curve::EaseInOutElastic => {
                const C5: f32 = (2.0 * PI) / 4.5;
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else if t < 0.5 {
                    -((2.0_f32).powf(20.0 * t - 10.0) * ((20.0 * t - 11.125) * C5).sin()) / 2.0
                } else {
                    ((2.0_f32).powf(-20.0 * t + 10.0) * ((20.0 * t - 11.125) * C5).sin()) / 2.0
                        + 1.0
                }
            }

            // Bounce
            Curve::EaseInBounce => 1.0 - Curve::EaseOutBounce.transform(1.0 - t),
            Curve::EaseOutBounce => {
                const N1: f32 = 7.5625;
                const D1: f32 = 2.75;

                if t < 1.0 / D1 {
                    N1 * t * t
                } else if t < 2.0 / D1 {
                    let t = t - 1.5 / D1;
                    N1 * t * t + 0.75
                } else if t < 2.5 / D1 {
                    let t = t - 2.25 / D1;
                    N1 * t * t + 0.9375
                } else {
                    let t = t - 2.625 / D1;
                    N1 * t * t + 0.984375
                }
            }
            Curve::EaseInOutBounce => {
                if t < 0.5 {
                    (1.0 - Curve::EaseOutBounce.transform(1.0 - 2.0 * t)) / 2.0
                } else {
                    (1.0 + Curve::EaseOutBounce.transform(2.0 * t - 1.0)) / 2.0
                }
            }

            // Cubic Bezier
            Curve::CubicBezier { x1, y1, x2, y2 } => {
                cubic_bezier_at(t, *x1, *y1, *x2, *y2)
            }
        }
    }

    /// Create a custom cubic Bezier curve
    ///
    /// Control points are (x1, y1) and (x2, y2), with implicit
    /// start point (0, 0) and end point (1, 1).
    pub const fn cubic_bezier(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Curve::CubicBezier { x1, y1, x2, y2 }
    }

    /// CSS ease curve equivalent
    pub const fn ease() -> Self {
        Curve::CubicBezier {
            x1: 0.25,
            y1: 0.1,
            x2: 0.25,
            y2: 1.0,
        }
    }

    /// CSS ease-in curve equivalent
    pub const fn css_ease_in() -> Self {
        Curve::CubicBezier {
            x1: 0.42,
            y1: 0.0,
            x2: 1.0,
            y2: 1.0,
        }
    }

    /// CSS ease-out curve equivalent
    pub const fn css_ease_out() -> Self {
        Curve::CubicBezier {
            x1: 0.0,
            y1: 0.0,
            x2: 0.58,
            y2: 1.0,
        }
    }

    /// CSS ease-in-out curve equivalent
    pub const fn css_ease_in_out() -> Self {
        Curve::CubicBezier {
            x1: 0.42,
            y1: 0.0,
            x2: 0.58,
            y2: 1.0,
        }
    }
}

/// Calculate y value for a cubic Bezier curve at given t
fn cubic_bezier_at(t: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    // Newton-Raphson iteration to find t for x
    let mut guess = t;

    for _ in 0..8 {
        let x = cubic_bezier_sample(guess, x1, x2);
        let diff = x - t;
        if diff.abs() < 1e-6 {
            break;
        }
        let dx = cubic_bezier_derivative(guess, x1, x2);
        if dx.abs() < 1e-6 {
            break;
        }
        guess -= diff / dx;
    }

    cubic_bezier_sample(guess, y1, y2)
}

/// Sample the Bezier curve
fn cubic_bezier_sample(t: f32, p1: f32, p2: f32) -> f32 {
    // B(t) = 3(1-t)²t·P1 + 3(1-t)t²·P2 + t³
    let t2 = t * t;
    let t3 = t2 * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;

    3.0 * mt2 * t * p1 + 3.0 * mt * t2 * p2 + t3
}

/// Derivative of the Bezier curve
fn cubic_bezier_derivative(t: f32, p1: f32, p2: f32) -> f32 {
    let t2 = t * t;
    let mt = 1.0 - t;

    3.0 * mt * mt * p1 + 6.0 * mt * t * (p2 - p1) + 3.0 * t2 * (1.0 - p2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear() {
        let curve = Curve::Linear;
        assert_eq!(curve.transform(0.0), 0.0);
        assert_eq!(curve.transform(0.5), 0.5);
        assert_eq!(curve.transform(1.0), 1.0);
    }

    #[test]
    fn test_ease_in_out() {
        let curve = Curve::EaseInOut;
        assert_eq!(curve.transform(0.0), 0.0);
        assert_eq!(curve.transform(1.0), 1.0);
        // Should be slower at start and end
        assert!(curve.transform(0.25) < 0.25);
        assert!(curve.transform(0.75) > 0.75);
    }

    #[test]
    fn test_clamp() {
        let curve = Curve::Linear;
        assert_eq!(curve.transform(-0.5), 0.0);
        assert_eq!(curve.transform(1.5), 1.0);
    }
}
