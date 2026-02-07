//! Animated Widgets Module
//!
//! Contains widgets that automatically animate property changes.

mod animated_box_shadow;
mod animated_opacity;
mod animated_position;
mod animated_scale;
mod fade_transition;
mod slide_transition;

pub use animated_box_shadow::{AnimatedBoxShadow, AnimatedBoxShadowRenderObject};
pub use animated_opacity::{AnimatedOpacity, AnimatedOpacityRenderObject};
pub use animated_position::{AnimatedPosition, AnimatedPositionRenderObject};
pub use animated_scale::{AnimatedScale, AnimatedScaleRenderObject};
pub use fade_transition::{FadeTransition, FadeTransitionRenderObject};
pub use slide_transition::{SlideDirection, SlideTransition, SlideTransitionRenderObject};
