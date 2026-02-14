//! Fade Transition Widget
//!
//! A widget that performs fade in/out transitions between visibility states.

use std::any::{Any, TypeId};

use hoshimi_types::{Constraints, Offset, Rect, Size};

use crate::animation::{AnimationController, Curve, Tween};
use crate::events::{EventResult, HitTestResult, InputEvent};
use crate::key::WidgetKey;
use crate::painter::Painter;
use crate::render_object::{
    Animatable, EventHandlable, Layoutable, Lifecycle, Paintable, Parent, RenderObject,
    RenderObjectState,
};
use crate::widget::Widget;

/// A widget that fades its child in or out based on visibility
#[derive(Debug)]
pub struct FadeTransition {
    /// Child widget
    pub child: Box<dyn Widget>,
    /// Whether the child should be visible
    pub visible: bool,
    /// Animation duration in seconds
    pub duration: f32,
    /// Animation curve
    pub curve: Curve,
    /// Optional widget key
    pub key: Option<WidgetKey>,
}

impl FadeTransition {
    /// Create a new fade transition widget
    pub fn new(child: impl Widget + 'static, visible: bool) -> Self {
        Self {
            child: Box::new(child),
            visible,
            duration: 0.3,
            curve: Curve::EaseInOut,
            key: None,
        }
    }

    /// Create a visible fade transition
    pub fn visible(child: impl Widget + 'static) -> Self {
        Self::new(child, true)
    }

    /// Create a hidden fade transition
    pub fn hidden(child: impl Widget + 'static) -> Self {
        Self::new(child, false)
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

    /// Set the widget key
    pub fn with_key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.key = Some(key.into());
        self
    }
}

impl Widget for FadeTransition {
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
        // Start from invisible if visible=true (will animate in on_mount)
        let initial_opacity = if self.visible { 0.0 } else { 0.0 };
        Box::new(FadeTransitionRenderObject {
            state: RenderObjectState::default(),
            child: self.child.create_render_object(),
            current_opacity: initial_opacity,
            visible: self.visible,
            controller: None,
            duration: self.duration,
            curve: self.curve,
            needs_entrance_animation: self.visible,
        })
    }

    fn update_render_object(&self, render_object: &mut dyn RenderObject) {
        if let Some(ro) = render_object.as_any_mut().downcast_mut::<FadeTransitionRenderObject>() {
            // Start animation if visibility changed
            if ro.visible != self.visible {
                ro.visible = self.visible;
                ro.duration = self.duration;
                ro.curve = self.curve;
                ro.start_animation();
            }
            
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
        Box::new(FadeTransition {
            child: self.child.clone_boxed(),
            visible: self.visible,
            duration: self.duration,
            curve: self.curve,
            key: self.key.clone(),
        })
    }
}

/// RenderObject for FadeTransition
#[derive(Debug)]
pub struct FadeTransitionRenderObject {
    state: RenderObjectState,
    child: Box<dyn RenderObject>,
    current_opacity: f32,
    visible: bool,
    controller: Option<AnimationController<f32>>,
    duration: f32,
    curve: Curve,
    /// Whether entrance animation should play on mount
    needs_entrance_animation: bool,
}

impl FadeTransitionRenderObject {
    fn start_animation(&mut self) {
        let target = if self.visible { 1.0 } else { 0.0 };
        let tween = Tween::new(self.current_opacity, target)
            .with_duration(self.duration)
            .with_curve(self.curve);
        
        let mut controller = AnimationController::new(tween);
        controller.play();
        self.controller = Some(controller);
    }

    /// Check if currently visible (opacity > 0)
    pub fn is_visible(&self) -> bool {
        self.current_opacity > 0.0
    }

    /// Get current opacity
    pub fn current_opacity(&self) -> f32 {
        self.current_opacity
    }
}

impl Animatable for FadeTransitionRenderObject {
    fn update(&mut self, delta: f32) {
        if let Some(ref mut controller) = self.controller {
            controller.update(delta);
            self.current_opacity = controller.value();
            
            if controller.is_completed() {
                self.current_opacity = if self.visible { 1.0 } else { 0.0 };
                self.controller = None;
            }
        }
    }

    fn is_animating(&self) -> bool {
        self.controller.as_ref().map_or(false, |c| c.is_animating())
    }
}

impl Layoutable for FadeTransitionRenderObject {
    fn layout(&mut self, constraints: Constraints) -> Size {
        let child_size = self.child.layout(constraints);
        self.child.set_offset(Offset::ZERO);
        self.state.size = child_size;
        child_size
    }

    fn get_rect(&self) -> Rect {
        self.state.get_rect()
    }

    fn set_offset(&mut self, offset: Offset) {
        self.state.offset = offset;
    }

    fn get_offset(&self) -> Offset {
        self.state.offset
    }

    fn get_size(&self) -> Size {
        self.state.size
    }

    fn needs_layout(&self) -> bool {
        self.state.needs_layout
    }

    fn mark_needs_layout(&mut self) {
        self.state.needs_layout = true;
    }
}

impl Paintable for FadeTransitionRenderObject {
    fn paint(&self, painter: &mut dyn Painter) {
        if self.current_opacity <= 0.0 {
            return;
        }

        painter.save();
        painter.translate(self.state.offset);

        // Note: The actual opacity application depends on the Painter implementation
        // The child should be rendered, opacity can be applied at higher level
        self.child.paint(painter);

        painter.restore();
    }

    fn needs_paint(&self) -> bool {
        self.state.needs_paint
    }

    fn mark_needs_paint(&mut self) {
        self.state.needs_paint = true;
    }
}

impl EventHandlable for FadeTransitionRenderObject {
    fn hit_test(&self, position: Offset) -> HitTestResult {
        // Only accept hits when fully visible
        if self.current_opacity < 1.0 {
            return HitTestResult::Miss;
        }

        let local = Offset::new(
            position.x - self.state.offset.x,
            position.y - self.state.offset.y,
        );
        self.child.hit_test(local)
    }

    fn handle_event(&mut self, event: &InputEvent) -> EventResult {
        if self.current_opacity < 1.0 {
            return EventResult::Ignored;
        }
        self.child.handle_event(event)
    }
}

impl Lifecycle for FadeTransitionRenderObject {
    fn on_mount(&mut self) {
        self.child.on_mount();
        // Start entrance animation when mounted (FadeTransition doesn't need size for animation)
        if self.needs_entrance_animation {
            self.needs_entrance_animation = false;
            self.start_animation();
        }
    }

    fn on_unmount(&mut self) {
        self.child.on_unmount();
    }
}

impl Parent for FadeTransitionRenderObject {
    fn children(&self) -> Vec<&dyn RenderObject> {
        vec![self.child.as_ref()]
    }

    fn children_mut(&mut self) -> Vec<&mut dyn RenderObject> {
        vec![self.child.as_mut()]
    }
}

impl RenderObject for FadeTransitionRenderObject {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn tick(&mut self, delta: f32) -> bool {
        Animatable::update(self, delta);
        let self_animating = Animatable::is_animating(self);

        let child_animating = self.child.tick(delta);

        if self_animating {
            self.state.needs_paint = true;
        }

        self_animating || child_animating
    }
}
