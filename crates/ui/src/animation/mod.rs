//! Animation Module
//!
//! This module provides a complete animation system for the Hoshimi UI framework,
//! including easing curves, tweening, and animation controllers.
//!
//! # Overview
//!
//! The animation system consists of three main components:
//!
//! - **Curves** ([`Curve`]): Easing functions that control animation timing
//! - **Tweens** ([`Tween`]): Interpolation between two values over time
//! - **Controllers** ([`AnimationController`]): Manage animation playback state
//!
//! # Quick Start
//!
//! ```ignore
//! use hoshimi_ui::animation::*;
//!
//! // Create a simple opacity animation
//! let tween = Tween::new(0.0_f32, 1.0)
//!     .with_duration(0.5)
//!     .with_curve(Curve::EaseOut);
//!
//! let mut controller = AnimationController::new(tween);
//! controller.play();
//!
//! // In your update loop:
//! loop {
//!     let delta_time = 0.016; // ~60fps
//!     controller.update(delta_time);
//!
//!     let current_opacity = controller.value();
//!     // Use current_opacity for rendering...
//!
//!     if controller.is_completed() {
//!         break;
//!     }
//! }
//! ```
//!
//! # Animation Curves
//!
//! The [`Curve`] enum provides many built-in easing functions:
//!
//! - Linear, quadratic, cubic, quartic, quintic
//! - Sinusoidal, exponential, circular
//! - Back (overshoot), elastic, bounce
//! - Custom cubic Bezier curves
//!
//! ```ignore
//! let linear = Curve::Linear;
//! let ease_in_out = Curve::EaseInOut;
//! let bounce = Curve::EaseOutBounce;
//! let custom = Curve::cubic_bezier(0.25, 0.1, 0.25, 1.0);
//! ```
//!
//! # Tweens
//!
//! [`Tween`] handles interpolation between values. Any type implementing
//! [`Interpolate`] can be animated:
//!
//! ```ignore
//! use hoshimi_types::{Offset, Color};
//!
//! // Float tween
//! let opacity = Tween::new(0.0_f32, 1.0);
//!
//! // Position tween
//! let position = Tween::new(
//!     Offset::new(0.0, 0.0),
//!     Offset::new(100.0, 200.0),
//! );
//!
//! // Color tween
//! let color = Tween::new(
//!     Color::RED,
//!     Color::BLUE,
//! );
//! ```
//!
//! # Animation Controllers
//!
//! [`AnimationController`] manages playback state including:
//!
//! - Play/pause/stop/resume
//! - Repeat modes (once, loop, ping-pong)
//! - Playback speed
//! - Delay before start
//!
//! ```ignore
//! let mut anim = AnimationController::new(tween)
//!     .with_repeat(RepeatMode::PingPong)
//!     .with_speed(2.0)
//!     .with_delay(0.5);
//!
//! anim.play();
//! ```
//!
//! # Tween Sequences
//!
//! Use [`TweenSequence`] to chain multiple animations:
//!
//! ```ignore
//! let sequence = TweenSequence::new()
//!     .add(Tween::new(0.0, 50.0).with_curve(Curve::EaseIn), 1.0)
//!     .add(Tween::new(50.0, 100.0).with_curve(Curve::EaseOut), 2.0);
//!
//! let value = sequence.value_at(0.5); // Get value at 50% progress
//! ```
//!
//! # Animation Groups
//!
//! Use [`AnimationGroup`] to manage multiple animations together:
//!
//! ```ignore
//! let mut group = AnimationGroup::new();
//! group.add(opacity_controller);
//! group.add(position_controller);
//!
//! group.play_all();
//!
//! // Update all animations at once
//! group.update(delta_time);
//! ```

mod controller;
mod curve;
mod tween;

// Re-export all public items
pub use controller::{
    blink_controller, fade_in_controller, fade_out_controller, pulse_controller,
    AnimationController, AnimationGroup, AnimationStatus, AnimationUpdatable, RepeatMode,
};
pub use curve::Curve;
pub use tween::{
    color_tween, fade_in, fade_out, opacity_tween, position_tween, scale_tween, size_tween,
    Interpolate, Tween, TweenSequence, TweenSequenceItem,
};
