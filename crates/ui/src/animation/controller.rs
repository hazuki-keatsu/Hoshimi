//! Animation Controller
//!
//! Manages the state and lifecycle of animations, including playback control,
//! repeat modes, and status tracking.
//!
//! # Usage
//!
//! ```ignore
//! use hoshimi_ui::animation::{AnimationController, Tween, Curve};
//!
//! let tween = Tween::new(0.0, 1.0).with_duration(1.0);
//! let mut controller = AnimationController::new(tween);
//!
//! // Start the animation
//! controller.play();
//!
//! // Update each frame (delta_time in seconds)
//! controller.update(0.016);
//!
//! // Get current value
//! let opacity = controller.value();
//! ```

use super::curve::Curve;
use super::tween::{Interpolate, Tween};

/// Animation playback status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AnimationStatus {
    /// Animation has not started or was reset
    #[default]
    Idle,
    /// Animation is playing forward
    Forward,
    /// Animation is playing in reverse
    Reverse,
    /// Animation has completed
    Completed,
    /// Animation is paused
    Paused,
}

impl AnimationStatus {
    /// Check if the animation is currently animating
    pub fn is_animating(&self) -> bool {
        matches!(self, AnimationStatus::Forward | AnimationStatus::Reverse)
    }

    /// Check if the animation has finished
    pub fn is_completed(&self) -> bool {
        matches!(self, AnimationStatus::Completed)
    }

    /// Check if the animation is idle (not started or reset)
    pub fn is_idle(&self) -> bool {
        matches!(self, AnimationStatus::Idle)
    }
}

/// Animation repeat behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RepeatMode {
    /// Play once and stop
    #[default]
    Once,
    /// Loop forever
    Loop,
    /// Loop a specific number of times
    Count(u32),
    /// Play forward then reverse (ping-pong), forever
    PingPong,
    /// Play forward then reverse, a specific number of times
    PingPongCount(u32),
}

/// Animation controller that manages playback of a tween
#[derive(Debug, Clone)]
pub struct AnimationController<T: Interpolate> {
    /// The underlying tween
    tween: Tween<T>,
    /// Current elapsed time in seconds
    elapsed: f32,
    /// Current playback status
    status: AnimationStatus,
    /// Repeat behavior
    repeat_mode: RepeatMode,
    /// Current repeat count
    repeat_count: u32,
    /// Whether currently playing in reverse (for ping-pong)
    is_reversed: bool,
    /// Playback speed multiplier (1.0 = normal)
    speed: f32,
    /// Delay before starting (seconds)
    delay: f32,
    /// Time spent in delay
    delay_elapsed: f32,
}

impl<T: Interpolate> AnimationController<T> {
    /// Create a new animation controller with a tween
    pub fn new(tween: Tween<T>) -> Self {
        Self {
            tween,
            elapsed: 0.0,
            status: AnimationStatus::Idle,
            repeat_mode: RepeatMode::Once,
            repeat_count: 0,
            is_reversed: false,
            speed: 1.0,
            delay: 0.0,
            delay_elapsed: 0.0,
        }
    }

    /// Set the repeat mode
    pub fn with_repeat(mut self, mode: RepeatMode) -> Self {
        self.repeat_mode = mode;
        self
    }

    /// Set the playback speed multiplier
    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }

    /// Set a delay before the animation starts
    pub fn with_delay(mut self, delay: f32) -> Self {
        self.delay = delay;
        self
    }

    /// Start playing the animation forward
    pub fn play(&mut self) {
        self.status = AnimationStatus::Forward;
        self.is_reversed = false;
    }

    /// Start playing the animation in reverse
    pub fn play_reverse(&mut self) {
        self.status = AnimationStatus::Reverse;
        self.is_reversed = true;
        self.elapsed = self.tween.duration;
    }

    /// Pause the animation
    pub fn pause(&mut self) {
        if self.status.is_animating() {
            self.status = AnimationStatus::Paused;
        }
    }

    /// Resume a paused animation
    pub fn resume(&mut self) {
        if self.status == AnimationStatus::Paused {
            self.status = if self.is_reversed {
                AnimationStatus::Reverse
            } else {
                AnimationStatus::Forward
            };
        }
    }

    /// Stop and reset the animation
    pub fn stop(&mut self) {
        self.status = AnimationStatus::Idle;
        self.elapsed = 0.0;
        self.repeat_count = 0;
        self.is_reversed = false;
        self.delay_elapsed = 0.0;
    }

    /// Reset the animation to the beginning without changing status
    pub fn reset(&mut self) {
        self.elapsed = 0.0;
        self.repeat_count = 0;
        self.is_reversed = false;
        self.delay_elapsed = 0.0;
    }

    /// Jump to a specific progress (0.0 to 1.0)
    pub fn seek(&mut self, progress: f32) {
        self.elapsed = self.tween.duration * progress.clamp(0.0, 1.0);
    }

    /// Update the animation with delta time (in seconds)
    ///
    /// Returns true if the animation is still running
    pub fn update(&mut self, delta: f32) -> bool {
        if !self.status.is_animating() {
            return false;
        }

        // Handle delay
        if self.delay_elapsed < self.delay {
            self.delay_elapsed += delta * self.speed;
            return true;
        }

        let effective_delta = delta * self.speed;

        match self.status {
            AnimationStatus::Forward => {
                self.elapsed += effective_delta;

                if self.elapsed >= self.tween.duration {
                    self.handle_completion();
                }
            }
            AnimationStatus::Reverse => {
                self.elapsed -= effective_delta;

                if self.elapsed <= 0.0 {
                    self.handle_completion();
                }
            }
            _ => {}
        }

        self.status.is_animating()
    }

    /// Handle animation completion
    fn handle_completion(&mut self) {
        match self.repeat_mode {
            RepeatMode::Once => {
                self.elapsed = if self.is_reversed {
                    0.0
                } else {
                    self.tween.duration
                };
                self.status = AnimationStatus::Completed;
            }
            RepeatMode::Loop => {
                self.elapsed = if self.is_reversed {
                    self.tween.duration
                } else {
                    0.0
                };
            }
            RepeatMode::Count(max) => {
                self.repeat_count += 1;
                if self.repeat_count >= max {
                    self.elapsed = if self.is_reversed {
                        0.0
                    } else {
                        self.tween.duration
                    };
                    self.status = AnimationStatus::Completed;
                } else {
                    self.elapsed = if self.is_reversed {
                        self.tween.duration
                    } else {
                        0.0
                    };
                }
            }
            RepeatMode::PingPong => {
                self.is_reversed = !self.is_reversed;
                self.elapsed = if self.is_reversed {
                    self.tween.duration
                } else {
                    0.0
                };
                self.status = if self.is_reversed {
                    AnimationStatus::Reverse
                } else {
                    AnimationStatus::Forward
                };
            }
            RepeatMode::PingPongCount(max) => {
                self.repeat_count += 1;
                if self.repeat_count >= max * 2 {
                    self.elapsed = if self.is_reversed {
                        0.0
                    } else {
                        self.tween.duration
                    };
                    self.status = AnimationStatus::Completed;
                } else {
                    self.is_reversed = !self.is_reversed;
                    self.elapsed = if self.is_reversed {
                        self.tween.duration
                    } else {
                        0.0
                    };
                    self.status = if self.is_reversed {
                        AnimationStatus::Reverse
                    } else {
                        AnimationStatus::Forward
                    };
                }
            }
        }
    }

    /// Get the current animated value
    pub fn value(&self) -> T {
        let progress = if self.tween.duration > 0.0 {
            (self.elapsed / self.tween.duration).clamp(0.0, 1.0)
        } else {
            1.0
        };
        self.tween.value_at(progress)
    }

    /// Get the current progress (0.0 to 1.0)
    pub fn progress(&self) -> f32 {
        if self.tween.duration > 0.0 {
            (self.elapsed / self.tween.duration).clamp(0.0, 1.0)
        } else {
            1.0
        }
    }

    /// Get the current status
    pub fn status(&self) -> AnimationStatus {
        self.status
    }

    /// Check if the animation is currently animating
    pub fn is_animating(&self) -> bool {
        self.status.is_animating()
    }

    /// Check if the animation has completed
    pub fn is_completed(&self) -> bool {
        self.status.is_completed()
    }

    /// Get the duration in seconds
    pub fn duration(&self) -> f32 {
        self.tween.duration
    }

    /// Get the elapsed time in seconds
    pub fn elapsed(&self) -> f32 {
        self.elapsed
    }

    /// Get a reference to the underlying tween
    pub fn tween(&self) -> &Tween<T> {
        &self.tween
    }

    /// Get a mutable reference to the underlying tween
    pub fn tween_mut(&mut self) -> &mut Tween<T> {
        &mut self.tween
    }
}

// ============================================================================
// AnimationControllerGroup
// ============================================================================

/// A group of animation controllers that can be updated together
pub struct AnimationGroup {
    controllers: Vec<Box<dyn AnimationUpdatable>>,
}

/// Trait for types that can be updated as part of an animation group
pub trait AnimationUpdatable {
    /// Update the animation
    fn update(&mut self, delta: f32) -> bool;

    /// Check if animating
    fn is_animating(&self) -> bool;

    /// Check if completed
    fn is_completed(&self) -> bool;

    /// Play the animation
    fn play(&mut self);

    /// Stop the animation
    fn stop(&mut self);
}

impl<T: Interpolate> AnimationUpdatable for AnimationController<T> {
    fn update(&mut self, delta: f32) -> bool {
        AnimationController::update(self, delta)
    }

    fn is_animating(&self) -> bool {
        AnimationController::is_animating(self)
    }

    fn is_completed(&self) -> bool {
        AnimationController::is_completed(self)
    }

    fn play(&mut self) {
        AnimationController::play(self)
    }

    fn stop(&mut self) {
        AnimationController::stop(self)
    }
}

impl AnimationGroup {
    /// Create a new empty animation group
    pub fn new() -> Self {
        Self {
            controllers: Vec::new(),
        }
    }

    /// Add an animation controller to the group
    pub fn add<T: Interpolate + 'static>(&mut self, controller: AnimationController<T>) {
        self.controllers.push(Box::new(controller));
    }

    /// Update all animations in the group
    pub fn update(&mut self, delta: f32) {
        for controller in &mut self.controllers {
            controller.update(delta);
        }
    }

    /// Play all animations
    pub fn play_all(&mut self) {
        for controller in &mut self.controllers {
            controller.play();
        }
    }

    /// Stop all animations
    pub fn stop_all(&mut self) {
        for controller in &mut self.controllers {
            controller.stop();
        }
    }

    /// Check if any animation is still running
    pub fn is_animating(&self) -> bool {
        self.controllers.iter().any(|c| c.is_animating())
    }

    /// Check if all animations have completed
    pub fn all_completed(&self) -> bool {
        self.controllers.iter().all(|c| c.is_completed())
    }

    /// Remove completed animations
    pub fn remove_completed(&mut self) {
        self.controllers.retain(|c| !c.is_completed());
    }

    /// Clear all animations
    pub fn clear(&mut self) {
        self.controllers.clear();
    }

    /// Get the number of animations
    pub fn len(&self) -> usize {
        self.controllers.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.controllers.is_empty()
    }
}

impl Default for AnimationGroup {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Factory functions for common animation controllers
// ============================================================================

/// Create a fade-in animation controller
pub fn fade_in_controller(duration: f32) -> AnimationController<f32> {
    AnimationController::new(
        Tween::new(0.0, 1.0)
            .with_duration(duration)
            .with_curve(Curve::EaseOut),
    )
}

/// Create a fade-out animation controller
pub fn fade_out_controller(duration: f32) -> AnimationController<f32> {
    AnimationController::new(
        Tween::new(1.0, 0.0)
            .with_duration(duration)
            .with_curve(Curve::EaseIn),
    )
}

/// Create a pulse animation controller (scale 1.0 -> 1.1 -> 1.0)
pub fn pulse_controller(duration: f32) -> AnimationController<f32> {
    AnimationController::new(
        Tween::new(1.0, 1.1)
            .with_duration(duration / 2.0)
            .with_curve(Curve::EaseInOut),
    )
    .with_repeat(RepeatMode::PingPong)
}

/// Create a blink animation controller (opacity 1.0 -> 0.0 -> 1.0)
pub fn blink_controller(duration: f32) -> AnimationController<f32> {
    AnimationController::new(
        Tween::new(1.0, 0.0)
            .with_duration(duration / 2.0)
            .with_curve(Curve::EaseInOut),
    )
    .with_repeat(RepeatMode::PingPong)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_playback() {
        let tween = Tween::new(0.0_f32, 100.0).with_duration(1.0);
        let mut controller = AnimationController::new(tween);

        assert_eq!(controller.status(), AnimationStatus::Idle);

        controller.play();
        assert_eq!(controller.status(), AnimationStatus::Forward);

        controller.update(0.5);
        assert!((controller.value() - 50.0).abs() < 0.01);

        controller.update(0.5);
        assert_eq!(controller.status(), AnimationStatus::Completed);
        assert!((controller.value() - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_loop() {
        let tween = Tween::new(0.0_f32, 100.0).with_duration(1.0);
        let mut controller = AnimationController::new(tween).with_repeat(RepeatMode::Loop);

        controller.play();
        controller.update(1.0);

        assert_eq!(controller.status(), AnimationStatus::Forward);
        assert!((controller.value() - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_ping_pong() {
        let tween = Tween::new(0.0_f32, 100.0).with_duration(1.0);
        let mut controller = AnimationController::new(tween).with_repeat(RepeatMode::PingPong);

        controller.play();
        controller.update(1.0);

        assert_eq!(controller.status(), AnimationStatus::Reverse);

        controller.update(1.0);
        assert_eq!(controller.status(), AnimationStatus::Forward);
    }

    #[test]
    fn test_pause_resume() {
        let tween = Tween::new(0.0_f32, 100.0).with_duration(1.0);
        let mut controller = AnimationController::new(tween);

        controller.play();
        controller.update(0.5);

        controller.pause();
        assert_eq!(controller.status(), AnimationStatus::Paused);

        controller.update(0.5);
        assert!((controller.value() - 50.0).abs() < 0.01); // Should not change

        controller.resume();
        controller.update(0.5);
        assert_eq!(controller.status(), AnimationStatus::Completed);
    }
}
