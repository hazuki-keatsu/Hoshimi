//! Animated Scale Widget
//!
//! A widget that animates the scale of its child.

use std::any::{Any, TypeId};

use hoshimi_shared::{Alignment, Offset};

use crate::animation::{AnimationController, Curve, Tween};
use crate::events::{HitTestResult};
use crate::key::WidgetKey;
use crate::painter::Painter;
use crate::render::{Animatable, RenderObject, RenderObjectState};
use crate::widget::Widget;

/// A widget that animates the scale of its child
#[derive(Debug)]
pub struct AnimatedScale {
    /// Child widget
    pub child: Box<dyn Widget>,
    /// Target scale (1.0 = normal size)
    pub scale: f32,
    /// Animation duration in seconds
    pub duration: f32,
    /// Animation curve
    pub curve: Curve,
    /// Scale origin alignment
    pub alignment: Alignment,
    /// Optional widget key
    pub key: Option<WidgetKey>,
}

impl AnimatedScale {
    /// Create a new animated scale widget
    pub fn new(child: impl Widget + 'static, scale: f32) -> Self {
        Self {
            child: Box::new(child),
            scale,
            duration: 0.3,
            curve: Curve::EaseInOut,
            alignment: Alignment::CENTER,
            key: None,
        }
    }

    /// Set the animation duration
    pub fn with_duration(mut self, duration: f32) -> Self {
        self.duration = duration;
        self
    }

    /// Set the animation curve
    pub fn with_curve(mut self, curve: Curve) -> Self {
        self.curve = curve;
        self
    }

    /// Set the scale origin alignment
    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Set the widget key
    pub fn with_key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.key = Some(key.into());
        self
    }
}

impl Widget for AnimatedScale {
    fn widget_type(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn key(&self) -> Option<WidgetKey> {
        self.key.clone()
    }

    fn children(&self) -> Vec<&dyn Widget> {
        vec![self.child.as_ref()]
    }

    fn create_render_object(&self) -> Box<dyn RenderObject> {
        // Start from scale 0 (will animate to target on_mount)
        Box::new(AnimatedScaleRenderObject {
            state: RenderObjectState::default(),
            child: self.child.create_render_object(),
            current_scale: 0.0,
            target_scale: self.scale,
            controller: None,
            duration: self.duration,
            curve: self.curve,
            alignment: self.alignment,
            needs_entrance_animation: true,
        })
    }

    fn update_render_object(&self, render_object: &mut dyn RenderObject) {
        if let Some(ro) = render_object.as_any_mut().downcast_mut::<AnimatedScaleRenderObject>() {
            // Only start animation if target changed
            if (ro.target_scale - self.scale).abs() > f32::EPSILON {
                ro.target_scale = self.scale;
                ro.duration = self.duration;
                ro.curve = self.curve;
                ro.start_animation();
            }
            
            ro.alignment = self.alignment;
            
            // Update child
            self.child.update_render_object(ro.child.as_mut());
        }
    }

    fn should_update(&self, _old: &dyn Widget) -> bool {
        true
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_boxed(&self) -> Box<dyn Widget> {
        Box::new(AnimatedScale {
            child: self.child.clone_boxed(),
            scale: self.scale,
            duration: self.duration,
            curve: self.curve,
            alignment: self.alignment,
            key: self.key.clone(),
        })
    }
}

/// RenderObject for AnimatedScale
#[derive(Debug)]
pub struct AnimatedScaleRenderObject {
    state: RenderObjectState,
    child: Box<dyn RenderObject>,
    current_scale: f32,
    target_scale: f32,
    controller: Option<AnimationController<f32>>,
    duration: f32,
    curve: Curve,
    alignment: Alignment,
    /// Whether entrance animation should play on mount
    needs_entrance_animation: bool,
}

impl AnimatedScaleRenderObject {
    fn start_animation(&mut self) {
        let tween = Tween::new(self.current_scale, self.target_scale)
            .with_duration(self.duration)
            .with_curve(self.curve);
        
        let mut controller = AnimationController::new(tween);
        controller.play();
        self.controller = Some(controller);
    }

    /// Get current scale
    pub fn current_scale(&self) -> f32 {
        self.current_scale
    }

    /// Calculate the scale origin based on alignment
    fn scale_origin(&self) -> Offset {
        // Convert alignment from (-1..1) range to (0..1) factor
        let ax = (self.alignment.x + 1.0) / 2.0;
        let ay = (self.alignment.y + 1.0) / 2.0;
        Offset::new(
            self.state.size.width * ax,
            self.state.size.height * ay,
        )
    }
}

impl Animatable for AnimatedScaleRenderObject {
    fn update(&mut self, delta: f32) {
        if let Some(ref mut controller) = self.controller {
            controller.update(delta);
            self.current_scale = controller.value();
            
            if controller.is_completed() {
                self.current_scale = self.target_scale;
                self.controller = None;
            }
        }
    }

    fn is_animating(&self) -> bool {
        self.controller.as_ref().map_or(false, |c| c.is_animating())
    }
}

impl RenderObject for AnimatedScaleRenderObject {
    crate::impl_single_child_layout!(state, child);
    crate::impl_animated_tick!(state, child);
    crate::impl_render_object_common!(state);

    fn on_mount(&mut self) {
        self.child.on_mount();
        // Start entrance animation when mounted (AnimatedScale doesn't need size for animation)
        if self.needs_entrance_animation {
            self.needs_entrance_animation = false;
            self.start_animation();
        }
    }

    fn on_unmount(&mut self) {
        self.child.on_unmount();
    }

    fn children(&self) -> Vec<&dyn RenderObject> {
        vec![self.child.as_ref()]
    }

    fn children_mut(&mut self) -> Vec<&mut dyn RenderObject> {
        vec![self.child.as_mut()]
    }

    fn paint(&self, painter: &mut dyn Painter) {
        if self.current_scale <= 0.0 {
            return;
        }

        painter.save();
        
        // First translate to own position
        painter.translate(self.state.offset);
        
        // Move to scale origin, scale, then move back
        let origin = self.scale_origin();
        painter.translate(origin);
        painter.scale(self.current_scale, self.current_scale);
        painter.translate(Offset::new(-origin.x, -origin.y));
        
        self.child.paint(painter);
        painter.restore();
    }

    fn hit_test(&self, position: Offset) -> HitTestResult {
        if self.current_scale <= 0.0 {
            return HitTestResult::Miss;
        }

        // Transform the hit point by inverse scale
        let origin = self.scale_origin();
        let local = Offset::new(
            (position.x - self.state.offset.x - origin.x) / self.current_scale + origin.x,
            (position.y - self.state.offset.y - origin.y) / self.current_scale + origin.y,
        );
        self.child.hit_test(local)
    }
}
