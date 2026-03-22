//! Page Transition System
//!
//! Provides animated transitions between pages using the animation module.
//! Supports various transition types like slide, fade, scale, and custom transitions.

use hoshimi_types::{Offset, Size};

use crate::animation::{AnimationController, AnimationStatus, Curve, Tween};
use crate::painter::Painter;

use super::snapshot::PageSnapshot;
use super::types::{ScaleAnchor, SlideDirection, TransitionType};

// ============================================================================
// Page Transform (for live painting without snapshots)
// ============================================================================

/// Transform parameters for live page painting during transitions.
///
/// Used when snapshot-based rendering is unavailable. The caller applies
/// these transforms to the painter before painting each page's UI tree.
#[derive(Debug, Clone)]
pub struct PageTransform {
    /// Translation offset
    pub offset: Offset,
    /// Optional scale: (sx, sy, anchor_point)
    pub scale: Option<(f32, f32, Offset)>,
}

impl PageTransform {
    /// Identity transform (no change)
    pub fn identity() -> Self {
        Self {
            offset: Offset::ZERO,
            scale: None,
        }
    }

    /// Create a translated transform
    pub fn translated(offset: Offset) -> Self {
        Self {
            offset,
            scale: None,
        }
    }

    /// Create a scaled transform
    pub fn scaled(sx: f32, sy: f32, anchor: Offset) -> Self {
        Self {
            offset: Offset::ZERO,
            scale: Some((sx, sy, anchor)),
        }
    }
}

// ============================================================================
// Transition State
// ============================================================================

/// Current state of a page transition
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionState {
    /// No transition active
    Idle,
    /// Transition is in progress
    Running,
    /// Transition has completed
    Completed,
}

// ============================================================================
// Active Transition
// ============================================================================

/// An active transition between two pages
/// 
/// Manages the animation state and snapshot painting for a page transition.
pub struct ActiveTransition {
    /// Snapshot of the page being transitioned from
    from_snapshot: PageSnapshot,
    
    /// Snapshot of the page being transitioned to
    to_snapshot: PageSnapshot,
    
    /// Main progress controller (0.0 to 1.0)
    progress_controller: AnimationController<f32>,
    
    /// Type of transition being performed
    transition_type: TransitionType,
    
    /// Screen/canvas size for calculations
    size: Size,
    
    /// Animation curve to use
    curve: Curve,
    
    /// Current state
    state: TransitionState,
    
    /// Whether this is a reverse transition (e.g., pop vs push)
    is_reverse: bool,
}

impl std::fmt::Debug for ActiveTransition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActiveTransition")
            .field("transition_type", &self.transition_type)
            .field("size", &self.size)
            .field("state", &self.state)
            .field("is_reverse", &self.is_reverse)
            .field("progress", &self.progress())
            .finish()
    }
}

impl ActiveTransition {
    /// Create a new active transition
    pub fn new(
        from_snapshot: PageSnapshot,
        to_snapshot: PageSnapshot,
        transition_type: TransitionType,
        size: Size,
        is_reverse: bool,
    ) -> Self {
        let duration = transition_type.duration();
        // Use curve from TransitionType if specified, otherwise use default
        let curve = transition_type.curve()
            .unwrap_or_else(|| Self::default_curve_for_transition(&transition_type));
        
        let tween = Tween::new(0.0_f32, 1.0)
            .with_duration(duration)
            .with_curve(curve.clone());
        
        let mut progress_controller = AnimationController::new(tween);
        progress_controller.play();
        
        Self {
            from_snapshot,
            to_snapshot,
            progress_controller,
            transition_type,
            size,
            curve,
            state: TransitionState::Running,
            is_reverse,
        }
    }
    
    /// Get the default animation curve for a transition type
    fn default_curve_for_transition(transition_type: &TransitionType) -> Curve {
        match transition_type {
            TransitionType::None => Curve::Linear,
            TransitionType::Slide { .. } => Curve::EaseOutCubic,
            TransitionType::Fade { .. } => Curve::EaseInOut,
            TransitionType::Scale { .. } => Curve::EaseOutBack,
            TransitionType::SlideAndFade { .. } => Curve::EaseOutCubic,
            TransitionType::Custom { .. } => Curve::EaseInOut,
        }
    }
    
    /// Set a custom animation curve
    pub fn with_curve(mut self, curve: Curve) -> Self {
        self.curve = curve.clone();
        self.progress_controller.tween_mut().curve = curve;
        self
    }
    
    /// Get the current progress (0.0 to 1.0), with curve applied
    pub fn progress(&self) -> f32 {
        // Use value() instead of progress() to get the curve-transformed value
        self.progress_controller.value()
    }
    
    /// Get the raw linear progress (0.0 to 1.0), without curve applied
    pub fn raw_progress(&self) -> f32 {
        self.progress_controller.progress()
    }
    
    /// Get the current state
    pub fn state(&self) -> TransitionState {
        self.state
    }
    
    /// Check if the transition is complete
    pub fn is_complete(&self) -> bool {
        self.state == TransitionState::Completed
    }
    
    /// Check if the transition is running
    pub fn is_running(&self) -> bool {
        self.state == TransitionState::Running
    }
    
    /// Update the transition with delta time
    /// 
    /// Returns true if still animating
    pub fn tick(&mut self, delta: f32) -> bool {
        if self.state != TransitionState::Running {
            return false;
        }
        
        // Update main progress
        let animating = self.progress_controller.update(delta);
        
        // Update dynamic nodes in snapshots
        self.from_snapshot.tick_dynamic_nodes(delta);
        self.to_snapshot.tick_dynamic_nodes(delta);
        
        // Check for completion
        if !animating && self.progress_controller.status() == AnimationStatus::Completed {
            self.state = TransitionState::Completed;
            return false;
        }
        
        true
    }
    
    /// Paint the transition to the painter
    pub fn paint(&self, painter: &mut dyn Painter) {
        let progress = self.progress();
        
        match &self.transition_type {
            TransitionType::None => {
                // No animation - just draw the target
                self.to_snapshot.paint(painter, Offset::ZERO, 1.0);
            }
            
            TransitionType::Slide { direction, .. } => {
                self.paint_slide(painter, progress, direction);
            }
            
            TransitionType::Fade { .. } => {
                self.paint_fade(painter, progress);
            }
            
            TransitionType::Scale { anchor, start_scale, end_scale, .. } => {
                self.paint_scale(painter, progress, anchor, *start_scale, *end_scale);
            }
            
            TransitionType::SlideAndFade { direction, .. } => {
                self.paint_slide_and_fade(painter, progress, direction);
            }
            
            TransitionType::Custom { .. } => {
                // For custom transitions, fall back to fade
                self.paint_fade(painter, progress);
            }
        }
    }
    
    /// Paint a slide transition
    fn paint_slide(&self, painter: &mut dyn Painter, progress: f32, direction: &SlideDirection) {
        // Use direction as-is: exit_transition() already defines the correct direction for pop
        let direction = *direction;
        
        // Calculate offsets
        let exit_end = direction.exit_end();
        let enter_start = direction.enter_start();
        
        // From page: moves from center to exit position
        let from_offset = Offset::new(
            exit_end.x * self.size.width * progress,
            exit_end.y * self.size.height * progress,
        );
        
        // To page: moves from enter position to center
        let to_offset = Offset::new(
            enter_start.x * self.size.width * (1.0 - progress),
            enter_start.y * self.size.height * (1.0 - progress),
        );
        
        // Paint from page (underneath)
        self.from_snapshot.paint(painter, from_offset, 1.0);
        
        // Paint to page (on top)
        self.to_snapshot.paint(painter, to_offset, 1.0);
    }
    
    /// Paint a fade transition (cross-fade)
    fn paint_fade(&self, painter: &mut dyn Painter, progress: f32) {
        // From page fades out
        let from_alpha = 1.0 - progress;
        self.from_snapshot.paint(painter, Offset::ZERO, from_alpha);
        
        // To page fades in
        let to_alpha = progress;
        self.to_snapshot.paint(painter, Offset::ZERO, to_alpha);
    }
    
    /// Paint a scale transition
    fn paint_scale(
        &self,
        painter: &mut dyn Painter,
        progress: f32,
        anchor: &ScaleAnchor,
        start_scale: f32,
        end_scale: f32,
    ) {
        let anchor_offset = anchor.to_offset(self.size);
        
        // From page: scales from 1.0 to end_scale and fades out
        let from_scale = 1.0 + (end_scale - 1.0) * progress;
        let from_alpha = 1.0 - progress;
        
        painter.save();
        painter.translate(anchor_offset);
        painter.scale(from_scale, from_scale);
        painter.translate(Offset::new(-anchor_offset.x, -anchor_offset.y));
        self.from_snapshot.paint(painter, Offset::ZERO, from_alpha);
        painter.restore();
        
        // To page: scales from start_scale to 1.0 and fades in
        let to_scale = start_scale + (1.0 - start_scale) * progress;
        let to_alpha = progress;
        
        painter.save();
        painter.translate(anchor_offset);
        painter.scale(to_scale, to_scale);
        painter.translate(Offset::new(-anchor_offset.x, -anchor_offset.y));
        self.to_snapshot.paint(painter, Offset::ZERO, to_alpha);
        painter.restore();
    }
    
    /// Paint a combined slide and fade transition
    fn paint_slide_and_fade(
        &self,
        painter: &mut dyn Painter,
        progress: f32,
        direction: &SlideDirection,
    ) {
        // Use direction as-is: exit_transition() already defines the correct direction for pop
        let direction = *direction;
        
        let exit_end = direction.exit_end();
        let enter_start = direction.enter_start();
        
        // From page: slides and fades out
        let from_offset = Offset::new(
            exit_end.x * self.size.width * progress * 0.3, // Reduced slide distance
            exit_end.y * self.size.height * progress * 0.3,
        );
        let from_alpha = 1.0 - progress;
        self.from_snapshot.paint(painter, from_offset, from_alpha);
        
        // To page: slides and fades in
        let to_offset = Offset::new(
            enter_start.x * self.size.width * (1.0 - progress) * 0.3,
            enter_start.y * self.size.height * (1.0 - progress) * 0.3,
        );
        let to_alpha = progress;
        self.to_snapshot.paint(painter, to_offset, to_alpha);
    }
    
    /// Get the from snapshot
    pub fn from_snapshot(&self) -> &PageSnapshot {
        &self.from_snapshot
    }
    
    /// Get the to snapshot
    pub fn to_snapshot(&self) -> &PageSnapshot {
        &self.to_snapshot
    }
    
    /// Consume the transition and return the to snapshot
    pub fn into_to_snapshot(self) -> PageSnapshot {
        self.to_snapshot
    }

    /// Check if this is a reverse (pop) transition
    pub fn is_reverse(&self) -> bool {
        self.is_reverse
    }

    /// Get the canvas/screen size
    pub fn size(&self) -> Size {
        self.size
    }

    /// Get transform parameters for live painting both pages.
    ///
    /// Returns `(from_transform, to_transform)` to apply to the exiting and
    /// entering pages respectively. The caller should apply
    /// `clip_rect(screen_rect)` before painting each page.
    pub fn get_live_transforms(&self) -> (PageTransform, PageTransform) {
        let progress = self.progress();

        match &self.transition_type {
            TransitionType::None => {
                (PageTransform::identity(), PageTransform::identity())
            }

            TransitionType::Slide { direction, .. } => {
                // Use direction as-is: exit_transition() already defines the correct direction for pop
                let exit_end = direction.exit_end();
                let enter_start = direction.enter_start();

                let from_offset = Offset::new(
                    exit_end.x * self.size.width * progress,
                    exit_end.y * self.size.height * progress,
                );
                let to_offset = Offset::new(
                    enter_start.x * self.size.width * (1.0 - progress),
                    enter_start.y * self.size.height * (1.0 - progress),
                );

                (
                    PageTransform::translated(from_offset),
                    PageTransform::translated(to_offset),
                )
            }

            TransitionType::Fade { .. } => {
                // Without save_layer_alpha, approximate fade with a very subtle slide
                let slide_amount = 0.05;
                let from_offset =
                    Offset::new(-self.size.width * slide_amount * progress, 0.0);
                let to_offset =
                    Offset::new(self.size.width * slide_amount * (1.0 - progress), 0.0);
                (
                    PageTransform::translated(from_offset),
                    PageTransform::translated(to_offset),
                )
            }

            TransitionType::Scale {
                anchor,
                start_scale,
                end_scale,
                ..
            } => {
                let anchor_offset = anchor.to_offset(self.size);
                let from_scale = 1.0 + (end_scale - 1.0) * progress;
                let to_scale = start_scale + (1.0 - start_scale) * progress;
                (
                    PageTransform::scaled(from_scale, from_scale, anchor_offset),
                    PageTransform::scaled(to_scale, to_scale, anchor_offset),
                )
            }

            TransitionType::SlideAndFade { direction, .. } => {
                // Use direction as-is: exit_transition() already defines the correct direction for pop
                let exit_end = direction.exit_end();
                let enter_start = direction.enter_start();

                let from_offset = Offset::new(
                    exit_end.x * self.size.width * progress * 0.3,
                    exit_end.y * self.size.height * progress * 0.3,
                );
                let to_offset = Offset::new(
                    enter_start.x * self.size.width * (1.0 - progress) * 0.3,
                    enter_start.y * self.size.height * (1.0 - progress) * 0.3,
                );

                (
                    PageTransform::translated(from_offset),
                    PageTransform::translated(to_offset),
                )
            }

            TransitionType::Custom { .. } => {
                // Fallback: slide left
                let from_offset =
                    Offset::new(-self.size.width * progress, 0.0);
                let to_offset =
                    Offset::new(self.size.width * (1.0 - progress), 0.0);
                (
                    PageTransform::translated(from_offset),
                    PageTransform::translated(to_offset),
                )
            }
        }
    }
}

// ============================================================================
// Transition Builder
// ============================================================================

/// Builder for creating page transitions with custom configuration
#[derive(Debug, Clone)]
pub struct TransitionBuilder {
    transition_type: TransitionType,
    curve: Option<Curve>,
    duration_override: Option<f32>,
}

impl TransitionBuilder {
    /// Create a new transition builder with a transition type
    pub fn new(transition_type: TransitionType) -> Self {
        Self {
            transition_type,
            curve: None,
            duration_override: None,
        }
    }
    
    /// Create a slide transition builder
    pub fn slide(direction: SlideDirection) -> Self {
        Self::new(TransitionType::slide(direction))
    }
    
    /// Create a fade transition builder
    pub fn fade() -> Self {
        Self::new(TransitionType::fade())
    }
    
    /// Create a scale transition builder
    pub fn scale(anchor: ScaleAnchor, start_scale: f32) -> Self {
        Self::new(TransitionType::scale(anchor, start_scale))
    }
    
    /// Set a custom animation curve
    pub fn with_curve(mut self, curve: Curve) -> Self {
        self.curve = Some(curve);
        self
    }
    
    /// Set a custom duration
    pub fn with_duration(mut self, duration: f32) -> Self {
        self.duration_override = Some(duration);
        self
    }
    
    /// Build an active transition
    pub fn build(
        self,
        from_snapshot: PageSnapshot,
        to_snapshot: PageSnapshot,
        size: Size,
        is_reverse: bool,
    ) -> ActiveTransition {
        let mut transition_type = self.transition_type;
        
        // Apply duration override
        if let Some(duration) = self.duration_override {
            transition_type = transition_type.with_duration(duration);
        }
        
        let mut transition = ActiveTransition::new(
            from_snapshot,
            to_snapshot,
            transition_type,
            size,
            is_reverse,
        );
        
        // Apply curve override
        if let Some(curve) = self.curve {
            transition = transition.with_curve(curve);
        }
        
        transition
    }
}

impl Default for TransitionBuilder {
    fn default() -> Self {
        Self::new(TransitionType::slide_left())
    }
}

// ============================================================================
// Transition Presets
// ============================================================================

/// Common transition presets
pub mod presets {
    use super::*;
    
    /// iOS-style push transition (slide from right)
    pub fn ios_push() -> TransitionBuilder {
        TransitionBuilder::slide(SlideDirection::Left)
            .with_curve(Curve::EaseOutCubic)
            .with_duration(0.35)
    }
    
    /// iOS-style pop transition (slide to right)
    pub fn ios_pop() -> TransitionBuilder {
        TransitionBuilder::slide(SlideDirection::Right)
            .with_curve(Curve::EaseOutCubic)
            .with_duration(0.35)
    }
    
    /// Android-style fade transition
    pub fn material_fade() -> TransitionBuilder {
        TransitionBuilder::fade()
            .with_curve(Curve::EaseInOut)
            .with_duration(0.3)
    }
    
    /// Material shared axis transition (X axis)
    pub fn material_shared_axis_x() -> TransitionBuilder {
        TransitionBuilder::new(TransitionType::slide_and_fade(SlideDirection::Left))
            .with_curve(Curve::EaseInOutCubic)
            .with_duration(0.3)
    }
    
    /// Material shared axis transition (Y axis)
    pub fn material_shared_axis_y() -> TransitionBuilder {
        TransitionBuilder::new(TransitionType::slide_and_fade(SlideDirection::Up))
            .with_curve(Curve::EaseInOutCubic)
            .with_duration(0.3)
    }
    
    /// Zoom transition (scale from center)
    pub fn zoom() -> TransitionBuilder {
        TransitionBuilder::scale(ScaleAnchor::Center, 0.9)
            .with_curve(Curve::EaseOutBack)
            .with_duration(0.3)
    }
    
    /// Modal presentation (slide from bottom)
    pub fn modal_present() -> TransitionBuilder {
        TransitionBuilder::slide(SlideDirection::Up)
            .with_curve(Curve::EaseOutCubic)
            .with_duration(0.4)
    }
    
    /// Modal dismissal (slide to bottom)
    pub fn modal_dismiss() -> TransitionBuilder {
        TransitionBuilder::slide(SlideDirection::Down)
            .with_curve(Curve::EaseInCubic)
            .with_duration(0.3)
    }
    
    /// No animation (instant)
    pub fn instant() -> TransitionBuilder {
        TransitionBuilder::new(TransitionType::None)
    }
}
