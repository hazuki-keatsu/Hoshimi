//! Tween (In-between) Animation System
//!
//! Provides interpolation between values over time with various
//! data types and animation curves.
//!
//! # Usage
//!
//! ```ignore
//! use hoshimi_ui::animation::{Tween, Curve};
//!
//! // Create a tween that animates from 0 to 100
//! let tween = Tween::new(0.0, 100.0)
//!     .with_duration(1.0)
//!     .with_curve(Curve::EaseInOut);
//!
//! // Get value at 50% progress
//! let value = tween.value_at(0.5); // Returns ~50.0 (depending on curve)
//! ```

use hoshimi_shared::{Color, Offset, Size};

use super::curve::Curve;

/// Trait for types that can be interpolated (tweened)
pub trait Interpolate: Clone {
    /// Linearly interpolate between self and other
    ///
    /// # Arguments
    /// - `other` - The target value
    /// - `t` - Progress value between 0.0 and 1.0
    fn lerp(&self, other: &Self, t: f32) -> Self;
}

// ============================================================================
// Interpolate implementations for basic types
// ============================================================================

impl Interpolate for f32 {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        self + (other - self) * t
    }
}

impl Interpolate for f64 {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        self + (other - self) * t as f64
    }
}

impl Interpolate for i32 {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        (*self as f32 + (*other - *self) as f32 * t).round() as i32
    }
}

impl Interpolate for u8 {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        (*self as f32 + (*other as f32 - *self as f32) * t).round() as u8
    }
}

impl Interpolate for Offset {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        Offset::new(self.x.lerp(&other.x, t), self.y.lerp(&other.y, t))
    }
}

impl Interpolate for Size {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        Size::new(
            self.width.lerp(&other.width, t),
            self.height.lerp(&other.height, t),
        )
    }
}

impl Interpolate for Color {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        Color::new(
            self.r.lerp(&other.r, t),
            self.g.lerp(&other.g, t),
            self.b.lerp(&other.b, t),
            self.a.lerp(&other.a, t),
        )
    }
}

// Tuple interpolation for common patterns
impl<A: Interpolate, B: Interpolate> Interpolate for (A, B) {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        (self.0.lerp(&other.0, t), self.1.lerp(&other.1, t))
    }
}

impl<A: Interpolate, B: Interpolate, C: Interpolate> Interpolate for (A, B, C) {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        (
            self.0.lerp(&other.0, t),
            self.1.lerp(&other.1, t),
            self.2.lerp(&other.2, t),
        )
    }
}

// ============================================================================
// Tween
// ============================================================================

/// A tween animation that interpolates between two values
#[derive(Debug, Clone)]
pub struct Tween<T: Interpolate> {
    /// Starting value
    pub begin: T,
    /// Ending value
    pub end: T,
    /// Duration in seconds
    pub duration: f32,
    /// Animation curve
    pub curve: Curve,
}

impl<T: Interpolate> Tween<T> {
    /// Create a new tween from begin to end
    pub fn new(begin: T, end: T) -> Self {
        Self {
            begin,
            end,
            duration: 1.0,
            curve: Curve::Linear,
        }
    }

    /// Set the duration in seconds
    pub fn with_duration(mut self, duration: f32) -> Self {
        self.duration = duration;
        self
    }

    /// Set the animation curve
    pub fn with_curve(mut self, curve: Curve) -> Self {
        self.curve = curve;
        self
    }

    /// Get the interpolated value at a given progress (0.0 to 1.0)
    pub fn value_at(&self, progress: f32) -> T {
        let eased = self.curve.transform(progress);
        self.begin.lerp(&self.end, eased)
    }

    /// Get the value at a given elapsed time
    pub fn value_at_time(&self, elapsed: f32) -> T {
        let progress = if self.duration > 0.0 {
            (elapsed / self.duration).clamp(0.0, 1.0)
        } else {
            1.0
        };
        self.value_at(progress)
    }

    /// Check if the animation is complete at the given elapsed time
    pub fn is_complete(&self, elapsed: f32) -> bool {
        elapsed >= self.duration
    }

    /// Create a reversed tween (end to begin)
    pub fn reversed(&self) -> Self {
        Self {
            begin: self.end.clone(),
            end: self.begin.clone(),
            duration: self.duration,
            curve: self.curve,
        }
    }
}

// ============================================================================
// TweenSequence
// ============================================================================

/// A sequence item with a weight
#[derive(Debug, Clone)]
pub struct TweenSequenceItem<T: Interpolate> {
    /// The tween for this segment
    pub tween: Tween<T>,
    /// The relative weight of this segment
    pub weight: f32,
}

impl<T: Interpolate> TweenSequenceItem<T> {
    /// Create a new sequence item
    pub fn new(tween: Tween<T>, weight: f32) -> Self {
        Self { tween, weight }
    }
}

/// A sequence of tweens that play in order
#[derive(Debug, Clone)]
pub struct TweenSequence<T: Interpolate> {
    items: Vec<TweenSequenceItem<T>>,
    total_weight: f32,
}

impl<T: Interpolate> TweenSequence<T> {
    /// Create a new empty sequence
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            total_weight: 0.0,
        }
    }

    /// Add a tween to the sequence with a weight
    pub fn add(mut self, tween: Tween<T>, weight: f32) -> Self {
        self.total_weight += weight;
        self.items.push(TweenSequenceItem::new(tween, weight));
        self
    }

    /// Get the value at a given progress (0.0 to 1.0)
    pub fn value_at(&self, progress: f32) -> Option<T> {
        if self.items.is_empty() || self.total_weight <= 0.0 {
            return None;
        }

        let progress = progress.clamp(0.0, 1.0);
        let target_weight = progress * self.total_weight;

        let mut accumulated = 0.0;
        for item in &self.items {
            let item_end = accumulated + item.weight;

            if target_weight <= item_end {
                // Found the current segment
                let segment_progress = if item.weight > 0.0 {
                    (target_weight - accumulated) / item.weight
                } else {
                    1.0
                };
                return Some(item.tween.value_at(segment_progress));
            }

            accumulated = item_end;
        }

        // Return the end value of the last item
        self.items.last().map(|item| item.tween.end.clone())
    }
}

impl<T: Interpolate> Default for TweenSequence<T> {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Common Tween Factory Functions
// ============================================================================

/// Create an opacity tween (0.0 = transparent, 1.0 = opaque)
pub fn opacity_tween(from: f32, to: f32, duration: f32, curve: Curve) -> Tween<f32> {
    Tween::new(from, to).with_duration(duration).with_curve(curve)
}

/// Create a fade-in tween
pub fn fade_in(duration: f32) -> Tween<f32> {
    Tween::new(0.0, 1.0)
        .with_duration(duration)
        .with_curve(Curve::EaseOut)
}

/// Create a fade-out tween
pub fn fade_out(duration: f32) -> Tween<f32> {
    Tween::new(1.0, 0.0)
        .with_duration(duration)
        .with_curve(Curve::EaseIn)
}

/// Create a position tween
pub fn position_tween(from: Offset, to: Offset, duration: f32, curve: Curve) -> Tween<Offset> {
    Tween::new(from, to).with_duration(duration).with_curve(curve)
}

/// Create a scale tween
pub fn scale_tween(from: f32, to: f32, duration: f32, curve: Curve) -> Tween<f32> {
    Tween::new(from, to).with_duration(duration).with_curve(curve)
}

/// Create a color tween
pub fn color_tween(from: Color, to: Color, duration: f32, curve: Curve) -> Tween<Color> {
    Tween::new(from, to).with_duration(duration).with_curve(curve)
}

/// Create a size tween
pub fn size_tween(from: Size, to: Size, duration: f32, curve: Curve) -> Tween<Size> {
    Tween::new(from, to).with_duration(duration).with_curve(curve)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f32_lerp() {
        assert_eq!(0.0_f32.lerp(&100.0, 0.0), 0.0);
        assert_eq!(0.0_f32.lerp(&100.0, 0.5), 50.0);
        assert_eq!(0.0_f32.lerp(&100.0, 1.0), 100.0);
    }

    #[test]
    fn test_tween_value_at() {
        let tween = Tween::new(0.0_f32, 100.0)
            .with_duration(2.0)
            .with_curve(Curve::Linear);

        assert_eq!(tween.value_at(0.0), 0.0);
        assert_eq!(tween.value_at(0.5), 50.0);
        assert_eq!(tween.value_at(1.0), 100.0);
    }

    #[test]
    fn test_tween_value_at_time() {
        let tween = Tween::new(0.0_f32, 100.0)
            .with_duration(2.0)
            .with_curve(Curve::Linear);

        assert_eq!(tween.value_at_time(0.0), 0.0);
        assert_eq!(tween.value_at_time(1.0), 50.0);
        assert_eq!(tween.value_at_time(2.0), 100.0);
        // Clamped
        assert_eq!(tween.value_at_time(3.0), 100.0);
    }

    #[test]
    fn test_offset_lerp() {
        let a = Offset::new(0.0, 0.0);
        let b = Offset::new(100.0, 200.0);

        let mid = a.lerp(&b, 0.5);
        assert_eq!(mid.x, 50.0);
        assert_eq!(mid.y, 100.0);
    }

    #[test]
    fn test_tween_sequence() {
        let seq = TweenSequence::new()
            .add(Tween::new(0.0_f32, 50.0), 1.0)
            .add(Tween::new(50.0_f32, 100.0), 1.0);

        assert_eq!(seq.value_at(0.0), Some(0.0));
        assert_eq!(seq.value_at(0.25), Some(25.0));
        assert_eq!(seq.value_at(0.5), Some(50.0));
        assert_eq!(seq.value_at(0.75), Some(75.0));
        assert_eq!(seq.value_at(1.0), Some(100.0));
    }
}
