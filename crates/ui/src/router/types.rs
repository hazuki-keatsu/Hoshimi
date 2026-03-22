//! Router Types
//!
//! Core types for the router module including transition types, page parameters,
//! and page state management.

use std::collections::HashMap;
use std::fmt::Debug;

use hoshimi_types::{Offset, Size};

use crate::animation::Curve;

// ============================================================================
// Page Parameters
// ============================================================================

/// Parameters passed to a page during navigation
/// 
/// Supports both typed and dynamic parameters for flexible routing.
#[derive(Debug, Clone, Default)]
pub struct PageParams {
    /// String parameters
    string_params: HashMap<String, String>,
    /// Integer parameters
    int_params: HashMap<String, i64>,
    /// Float parameters
    float_params: HashMap<String, f64>,
    /// Boolean parameters
    bool_params: HashMap<String, bool>,
}

impl PageParams {
    /// Create empty page parameters
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Add a string parameter
    pub fn with_string(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.string_params.insert(key.into(), value.into());
        self
    }
    
    /// Add an integer parameter
    pub fn with_int(mut self, key: impl Into<String>, value: i64) -> Self {
        self.int_params.insert(key.into(), value);
        self
    }
    
    /// Add a float parameter
    pub fn with_float(mut self, key: impl Into<String>, value: f64) -> Self {
        self.float_params.insert(key.into(), value);
        self
    }
    
    /// Add a boolean parameter
    pub fn with_bool(mut self, key: impl Into<String>, value: bool) -> Self {
        self.bool_params.insert(key.into(), value);
        self
    }
    
    /// Get a string parameter
    pub fn get_string(&self, key: &str) -> Option<&str> {
        self.string_params.get(key).map(|s| s.as_str())
    }
    
    /// Get an integer parameter
    pub fn get_int(&self, key: &str) -> Option<i64> {
        self.int_params.get(key).copied()
    }
    
    /// Get a float parameter
    pub fn get_float(&self, key: &str) -> Option<f64> {
        self.float_params.get(key).copied()
    }
    
    /// Get a boolean parameter
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.bool_params.get(key).copied()
    }
}

// ============================================================================
// Page State
// ============================================================================

/// Current state of a page in the router stack
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageState {
    /// Page is currently active and visible
    Active,
    /// Page is paused (behind another page)
    Paused,
    /// Page is being transitioned out
    TransitioningOut,
    /// Page is being transitioned in
    TransitioningIn,
}

impl PageState {
    /// Check if the page is currently visible
    pub fn is_visible(&self) -> bool {
        matches!(self, Self::Active | Self::TransitioningIn | Self::TransitioningOut)
    }
    
    /// Check if the page is interactive
    pub fn is_interactive(&self) -> bool {
        matches!(self, Self::Active)
    }
}

// ============================================================================
// Transition Types
// ============================================================================

/// Direction for slide transitions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SlideDirection {
    /// Slide from left to right
    #[default]
    Left,
    /// Slide from right to left
    Right,
    /// Slide from top to bottom
    Up,
    /// Slide from bottom to top
    Down,
}

impl SlideDirection {
    /// Get the start offset for entering page (normalized, multiply by screen size)
    pub fn enter_start(&self) -> Offset {
        match self {
            Self::Left => Offset::new(1.0, 0.0),
            Self::Right => Offset::new(-1.0, 0.0),
            Self::Up => Offset::new(0.0, 1.0),
            Self::Down => Offset::new(0.0, -1.0),
        }
    }
    
    /// Get the end offset for exiting page (normalized, multiply by screen size)
    pub fn exit_end(&self) -> Offset {
        match self {
            Self::Left => Offset::new(-1.0, 0.0),
            Self::Right => Offset::new(1.0, 0.0),
            Self::Up => Offset::new(0.0, -1.0),
            Self::Down => Offset::new(0.0, 1.0),
        }
    }
    
    /// Get the reverse direction
    pub fn reverse(&self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Up => Self::Down,
            Self::Down => Self::Up,
        }
    }
}

/// Anchor point for scale transitions
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ScaleAnchor {
    /// Scale from center
    #[default]
    Center,
    /// Scale from top-left
    TopLeft,
    /// Scale from top-right
    TopRight,
    /// Scale from bottom-left
    BottomLeft,
    /// Scale from bottom-right
    BottomRight,
    /// Scale from a custom point (0.0-1.0 normalized)
    Custom(f32, f32),
}

impl ScaleAnchor {
    /// Convert to normalized offset (0.0-1.0)
    pub fn to_normalized(&self) -> Offset {
        match self {
            Self::Center => Offset::new(0.5, 0.5),
            Self::TopLeft => Offset::new(0.0, 0.0),
            Self::TopRight => Offset::new(1.0, 0.0),
            Self::BottomLeft => Offset::new(0.0, 1.0),
            Self::BottomRight => Offset::new(1.0, 1.0),
            Self::Custom(x, y) => Offset::new(*x, *y),
        }
    }
    
    /// Get the actual anchor point for a given size
    pub fn to_offset(&self, size: Size) -> Offset {
        let norm = self.to_normalized();
        Offset::new(norm.x * size.width, norm.y * size.height)
    }
}

/// Type of page transition animation
#[derive(Debug, Clone, Default)]
pub enum TransitionType {
    /// No transition, instant switch
    #[default]
    None,
    
    /// Slide transition
    Slide {
        /// Direction of the slide
        direction: SlideDirection,
        /// Duration in seconds
        duration: f32,
        /// Animation curve (optional, uses default if None)
        curve: Option<Curve>,
    },
    
    /// Fade transition (cross-fade)
    Fade {
        /// Duration in seconds
        duration: f32,
        /// Animation curve (optional, uses default if None)
        curve: Option<Curve>,
    },
    
    /// Scale transition (zoom in/out)
    Scale {
        /// Anchor point for scaling
        anchor: ScaleAnchor,
        /// Start scale (for entering: typically < 1.0)
        start_scale: f32,
        /// End scale (typically 1.0)
        end_scale: f32,
        /// Duration in seconds
        duration: f32,
        /// Animation curve (optional, uses default if None)
        curve: Option<Curve>,
    },
    
    /// Combined slide and fade
    SlideAndFade {
        /// Direction of the slide
        direction: SlideDirection,
        /// Duration in seconds
        duration: f32,
        /// Animation curve (optional, uses default if None)
        curve: Option<Curve>,
    },
    
    /// Custom transition with user-defined tweens
    Custom {
        /// Custom transition builder
        builder: CustomTransitionBuilder,
        /// Animation curve (optional, uses default if None)
        curve: Option<Curve>,
    },
}

impl TransitionType {
    /// Create a slide transition with default duration
    pub fn slide(direction: SlideDirection) -> Self {
        Self::Slide {
            direction,
            duration: 0.3,
            curve: None,
        }
    }
    
    /// Create a slide transition from the left
    pub fn slide_left() -> Self {
        Self::slide(SlideDirection::Left)
    }
    
    /// Create a slide transition from the right
    pub fn slide_right() -> Self {
        Self::slide(SlideDirection::Right)
    }
    
    /// Create a slide transition from the top
    pub fn slide_up() -> Self {
        Self::slide(SlideDirection::Up)
    }
    
    /// Create a slide transition from the bottom
    pub fn slide_down() -> Self {
        Self::slide(SlideDirection::Down)
    }
    
    /// Create a fade transition with default duration
    pub fn fade() -> Self {
        Self::Fade { duration: 0.3, curve: None }
    }
    
    /// Create a fade transition with custom duration
    pub fn fade_with_duration(duration: f32) -> Self {
        Self::Fade { duration, curve: None }
    }
    
    /// Create a scale transition
    pub fn scale(anchor: ScaleAnchor, start_scale: f32) -> Self {
        Self::Scale {
            anchor,
            start_scale,
            end_scale: 1.0,
            duration: 0.3,
            curve: None,
        }
    }
    
    /// Create a zoom-in transition from center
    pub fn zoom_in() -> Self {
        Self::scale(ScaleAnchor::Center, 0.8)
    }
    
    /// Create a zoom-out transition from center
    pub fn zoom_out() -> Self {
        Self::Scale {
            anchor: ScaleAnchor::Center,
            start_scale: 1.0,
            end_scale: 0.8,
            duration: 0.3,
            curve: None,
        }
    }
    
    /// Create a slide and fade transition
    pub fn slide_and_fade(direction: SlideDirection) -> Self {
        Self::SlideAndFade {
            direction,
            duration: 0.3,
            curve: None,
        }
    }
    
    /// Set the duration for this transition
    pub fn with_duration(mut self, duration: f32) -> Self {
        match &mut self {
            Self::None => {}
            Self::Slide { duration: d, .. } => *d = duration,
            Self::Fade { duration: d, .. } => *d = duration,
            Self::Scale { duration: d, .. } => *d = duration,
            Self::SlideAndFade { duration: d, .. } => *d = duration,
            Self::Custom { .. } => {}
        }
        self
    }
    
    /// Get the duration of this transition
    pub fn duration(&self) -> f32 {
        match self {
            Self::None => 0.0,
            Self::Slide { duration, .. } => *duration,
            Self::Fade { duration, .. } => *duration,
            Self::Scale { duration, .. } => *duration,
            Self::SlideAndFade { duration, .. } => *duration,
            Self::Custom { builder, .. } => builder.duration,
        }
    }
    
    /// Get the reverse transition (for pop operations)
    pub fn reverse(&self) -> Self {
        match self {
            Self::None => Self::None,
            Self::Slide { direction, duration, curve } => Self::Slide {
                direction: direction.reverse(),
                duration: *duration,
                curve: *curve,
            },
            Self::Fade { duration, curve } => Self::Fade { duration: *duration, curve: *curve },
            Self::Scale { anchor, start_scale, end_scale, duration, curve } => Self::Scale {
                anchor: *anchor,
                start_scale: *end_scale,
                end_scale: *start_scale,
                duration: *duration,
                curve: *curve,
            },
            Self::SlideAndFade { direction, duration, curve } => Self::SlideAndFade {
                direction: direction.reverse(),
                duration: *duration,
                curve: *curve,
            },
            Self::Custom { builder, curve } => Self::Custom {
                builder: builder.clone(),
                curve: *curve,
            },
        }
    }
    
    /// Set the animation curve for this transition
    pub fn with_curve(mut self, curve: Curve) -> Self {
        match &mut self {
            Self::None => {}
            Self::Slide { curve: c, .. } => *c = Some(curve),
            Self::Fade { curve: c, .. } => *c = Some(curve),
            Self::Scale { curve: c, .. } => *c = Some(curve),
            Self::SlideAndFade { curve: c, .. } => *c = Some(curve),
            Self::Custom { curve: c, .. } => *c = Some(curve),
        }
        self
    }
    
    /// Get the animation curve for this transition (if set)
    pub fn curve(&self) -> Option<Curve> {
        match self {
            Self::None => None,
            Self::Slide { curve, .. } => *curve,
            Self::Fade { curve, .. } => *curve,
            Self::Scale { curve, .. } => *curve,
            Self::SlideAndFade { curve, .. } => *curve,
            Self::Custom { curve, .. } => *curve,
        }
    }
}

// ============================================================================
// Custom Transition Builder
// ============================================================================

/// Builder for custom transitions
#[derive(Debug, Clone)]
pub struct CustomTransitionBuilder {
    /// Duration in seconds
    pub duration: f32,
    /// Transform function ID (for serialization)
    pub transform_id: String,
}

impl CustomTransitionBuilder {
    /// Create a new custom transition builder
    pub fn new(duration: f32, transform_id: impl Into<String>) -> Self {
        Self {
            duration,
            transform_id: transform_id.into(),
        }
    }
}

// ============================================================================
// Route Definition
// ============================================================================

/// Result of a navigation operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationResult {
    /// Navigation successful
    Success,
    /// Navigation failed (route not found)
    RouteNotFound,
    /// Navigation failed (stack is empty, cannot pop)
    StackEmpty,
    /// Navigation is already in progress
    TransitionInProgress,
}

impl NavigationResult {
    /// Check if navigation was successful
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success)
    }
}

/// Options for navigation operations
#[derive(Debug, Clone, Default)]
pub struct NavigationOptions {
    /// Override the default transition
    pub transition: Option<TransitionType>,
    /// Clear the stack before navigating
    pub clear_stack: bool,
    /// Replace the current page instead of pushing
    pub replace: bool,
}

impl NavigationOptions {
    /// Create default navigation options
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set a custom transition
    pub fn with_transition(mut self, transition: TransitionType) -> Self {
        self.transition = Some(transition);
        self
    }
    
    /// Clear the stack before navigating
    pub fn clear_stack(mut self) -> Self {
        self.clear_stack = true;
        self
    }
    
    /// Replace the current page
    pub fn replace(mut self) -> Self {
        self.replace = true;
        self
    }
}
