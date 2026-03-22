//! Animated Opacity Widget
//!
//! A widget that animates the opacity of its child.
//! 
//! Note: Since the Painter trait doesn't have a global opacity setting,
//! this widget stores the opacity value that can be used by child widgets
//! for their own rendering (e.g., images can use draw_image_with_alpha).

use std::any::{Any, TypeId};

use hoshimi_types::{Constraints, Offset, Rect, Size};

use crate::animation::{AnimationController, Curve, Tween};
use crate::events::{EventResult, HitTestResult, InputEvent};
use crate::key::WidgetKey;
use crate::painter::{Painter, TextMeasurer};
use crate::render_object::{
    Animatable, EventHandlable, Layoutable, Lifecycle, Paintable, Parent, RenderObject,
    RenderObjectState,
};
use crate::widget::Widget;

/// A widget that animates the opacity of its child
/// 
/// This widget tracks an opacity value that animates smoothly when changed.
/// Child widgets can query the current opacity through the RenderObject.
#[derive(Debug)]
pub struct AnimatedOpacity {
    /// Child widget
    pub child: Box<dyn Widget>,
    /// Target opacity (0.0 to 1.0)
    pub opacity: f32,
    /// Animation duration in seconds
    pub duration: f32,
    /// Animation curve
    pub curve: Curve,
    /// Optional widget key
    pub key: Option<WidgetKey>,
}

impl AnimatedOpacity {
    /// Create a new animated opacity widget
    pub fn new(child: impl Widget + 'static, opacity: f32) -> Self {
        Self {
            child: Box::new(child),
            opacity: opacity.clamp(0.0, 1.0),
            duration: 0.3,
            curve: Curve::EaseInOut,
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

    /// Set the widget key
    pub fn with_key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.key = Some(key.into());
        self
    }
}

impl Widget for AnimatedOpacity {
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
        Box::new(AnimatedOpacityRenderObject {
            state: RenderObjectState::default(),
            child: self.child.create_render_object(),
            current_opacity: self.opacity,
            target_opacity: self.opacity,
            controller: None,
            duration: self.duration,
            curve: self.curve,
        })
    }

    fn update_render_object(&self, render_object: &mut dyn RenderObject) {
        if let Some(ro) = render_object.as_any_mut().downcast_mut::<AnimatedOpacityRenderObject>() {
            // Only start animation if target changed
            if (ro.target_opacity - self.opacity).abs() > f32::EPSILON {
                ro.target_opacity = self.opacity;
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
        Box::new(AnimatedOpacity {
            child: self.child.clone_boxed(),
            opacity: self.opacity,
            duration: self.duration,
            curve: self.curve,
            key: self.key.clone(),
        })
    }
}

/// RenderObject for AnimatedOpacity
#[derive(Debug)]
pub struct AnimatedOpacityRenderObject {
    state: RenderObjectState,
    child: Box<dyn RenderObject>,
    current_opacity: f32,
    target_opacity: f32,
    controller: Option<AnimationController<f32>>,
    duration: f32,
    curve: Curve,
}

impl AnimatedOpacityRenderObject {
    fn start_animation(&mut self) {
        let tween = Tween::new(self.current_opacity, self.target_opacity)
            .with_duration(self.duration)
            .with_curve(self.curve);
        
        let mut controller = AnimationController::new(tween);
        controller.play();
        self.controller = Some(controller);
    }

    /// Get current opacity (0.0 to 1.0)
    pub fn current_opacity(&self) -> f32 {
        self.current_opacity
    }
}

impl Animatable for AnimatedOpacityRenderObject {
    fn update(&mut self, delta: f32) {
        if let Some(ref mut controller) = self.controller {
            controller.update(delta);
            self.current_opacity = controller.value();
            
            if controller.is_completed() {
                self.current_opacity = self.target_opacity;
                self.controller = None;
            }
        }
    }

    fn is_animating(&self) -> bool {
        self.controller.as_ref().map_or(false, |c| c.is_animating())
    }
}

impl Layoutable for AnimatedOpacityRenderObject {
    fn layout(&mut self, constraints: Constraints, text_measurer: &dyn TextMeasurer) -> Size {
        let child_size = self.child.layout(constraints, text_measurer);
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

impl Paintable for AnimatedOpacityRenderObject {
    fn paint(&self, painter: &mut dyn Painter) {
        if self.current_opacity <= 0.0 {
            return;
        }

        painter.save();
        painter.translate(self.state.offset);

        // Note: The actual opacity application depends on the Painter implementation
        // The child should be rendered, and opacity can be applied at a higher level
        // or by specific widgets (like Image) that support alpha
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

impl EventHandlable for AnimatedOpacityRenderObject {
    fn hit_test(&self, position: Offset) -> HitTestResult {
        if self.current_opacity <= 0.0 {
            return HitTestResult::Miss;
        }

        let local = Offset::new(
            position.x - self.state.offset.x,
            position.y - self.state.offset.y,
        );
        self.child.hit_test(local)
    }

    fn handle_event(&mut self, event: &InputEvent) -> EventResult {
        self.child.handle_event(event)
    }
}

impl Lifecycle for AnimatedOpacityRenderObject {
    fn on_mount(&mut self) {
        self.child.on_mount();
    }

    fn on_unmount(&mut self) {
        self.child.on_unmount();
    }
}

impl Parent for AnimatedOpacityRenderObject {
    fn children(&self) -> Vec<&dyn RenderObject> {
        vec![self.child.as_ref()]
    }

    fn children_mut(&mut self) -> Vec<&mut dyn RenderObject> {
        vec![self.child.as_mut()]
    }
}

impl RenderObject for AnimatedOpacityRenderObject {
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
