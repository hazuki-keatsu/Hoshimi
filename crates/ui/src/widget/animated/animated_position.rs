//! Animated Position Widget
//!
//! A widget that animates the position offset of its child.

use std::any::{Any, TypeId};

use hoshimi_shared::{Constraints, Offset, Rect, Size};

use crate::animation::{AnimationController, Curve, Tween};
use crate::events::{EventResult, HitTestResult, InputEvent};
use crate::key::WidgetKey;
use crate::painter::Painter;
use crate::render::{RenderObject, RenderObjectState};
use crate::widget::Widget;

/// A widget that animates the position of its child
#[derive(Debug)]
pub struct AnimatedPosition {
    /// Child widget
    pub child: Box<dyn Widget>,
    /// Target X offset
    pub x: f32,
    /// Target Y offset
    pub y: f32,
    /// Animation duration in seconds
    pub duration: f32,
    /// Animation curve
    pub curve: Curve,
    /// Optional widget key
    pub key: Option<WidgetKey>,
}

impl AnimatedPosition {
    /// Create a new animated position widget
    pub fn new(child: impl Widget + 'static, x: f32, y: f32) -> Self {
        Self {
            child: Box::new(child),
            x,
            y,
            duration: 0.3,
            curve: Curve::EaseInOut,
            key: None,
        }
    }

    /// Create with an offset
    pub fn with_offset(child: impl Widget + 'static, offset: Offset) -> Self {
        Self::new(child, offset.x, offset.y)
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

impl Widget for AnimatedPosition {
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
        Box::new(AnimatedPositionRenderObject {
            state: RenderObjectState::default(),
            child: self.child.create_render_object(),
            current_position: Offset::new(self.x, self.y),
            target_position: Offset::new(self.x, self.y),
            controller: None,
            duration: self.duration,
            curve: self.curve,
        })
    }

    fn update_render_object(&self, render_object: &mut dyn RenderObject) {
        if let Some(ro) = render_object.as_any_mut().downcast_mut::<AnimatedPositionRenderObject>() {
            let target = Offset::new(self.x, self.y);
            
            // Only start animation if target changed
            if (ro.target_position.x - target.x).abs() > f32::EPSILON 
                || (ro.target_position.y - target.y).abs() > f32::EPSILON {
                ro.target_position = target;
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
}

/// RenderObject for AnimatedPosition
#[derive(Debug)]
pub struct AnimatedPositionRenderObject {
    state: RenderObjectState,
    child: Box<dyn RenderObject>,
    current_position: Offset,
    target_position: Offset,
    controller: Option<AnimationController<Offset>>,
    duration: f32,
    curve: Curve,
}

impl AnimatedPositionRenderObject {
    fn start_animation(&mut self) {
        let tween = Tween::new(self.current_position, self.target_position)
            .with_duration(self.duration)
            .with_curve(self.curve);
        
        let mut controller = AnimationController::new(tween);
        controller.play();
        self.controller = Some(controller);
    }

    /// Update animation state (call each frame)
    pub fn update(&mut self, delta: f32) {
        if let Some(ref mut controller) = self.controller {
            controller.update(delta);
            self.current_position = controller.value();
            
            if controller.is_completed() {
                self.current_position = self.target_position;
                self.controller = None;
            }
        }
    }

    /// Check if animation is in progress
    pub fn is_animating(&self) -> bool {
        self.controller.as_ref().map_or(false, |c| c.is_animating())
    }

    /// Get current position offset
    pub fn current_position(&self) -> Offset {
        self.current_position
    }
}

impl RenderObject for AnimatedPositionRenderObject {
    fn layout(&mut self, constraints: Constraints) -> Size {
        let child_size = self.child.layout(constraints);
        self.child.set_offset(Offset::ZERO);
        self.state.size = child_size;
        child_size
    }

    fn get_rect(&self) -> Rect {
        Rect::new(
            self.state.offset.x + self.current_position.x,
            self.state.offset.y + self.current_position.y,
            self.state.size.width,
            self.state.size.height,
        )
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

    fn paint(&self, painter: &mut dyn Painter) {
        painter.save();
        painter.translate(self.current_position);
        self.child.paint(painter);
        painter.restore();
    }

    fn hit_test(&self, position: Offset) -> HitTestResult {
        let local = Offset::new(
            position.x - self.state.offset.x - self.current_position.x,
            position.y - self.state.offset.y - self.current_position.y,
        );
        self.child.hit_test(local)
    }

    fn handle_event(&mut self, event: &InputEvent) -> EventResult {
        self.child.handle_event(event)
    }

    fn on_mount(&mut self) {
        self.child.on_mount();
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

    fn needs_layout(&self) -> bool {
        self.state.needs_layout
    }

    fn mark_needs_layout(&mut self) {
        self.state.needs_layout = true;
    }

    fn needs_paint(&self) -> bool {
        self.state.needs_paint
    }

    fn mark_needs_paint(&mut self) {
        self.state.needs_paint = true;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
