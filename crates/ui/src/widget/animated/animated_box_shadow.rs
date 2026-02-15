//! Animated Box Shadow Widget
//!
//! A widget that animates box shadow changes (color, offset, blur, spread).
//! 
//! When the shadow properties change, the transition is smoothly interpolated
//! over the specified duration using the chosen animation curve.

use std::any::{Any, TypeId};

use hoshimi_types::{BoxShadow, Color, Constraints, Offset, Rect, Size};

use crate::animation::{AnimationController, Curve, Tween};
use crate::events::{EventResult, HitTestResult, InputEvent};
use crate::key::WidgetKey;
use crate::painter::{Painter, TextMeasurer};
use crate::render_object::{
    Animatable, EventHandlable, Layoutable, Lifecycle, Paintable, Parent, RenderObject,
    RenderObjectState,
};
use crate::widget::Widget;

/// A widget that animates box shadow changes
/// 
/// When the shadow properties change, the widget smoothly animates
/// from the current shadow to the new shadow state.
/// 
/// # Example
/// 
/// ```ignore
/// use hoshimi_ui::widget::AnimatedBoxShadow;
/// use hoshimi_types::{BoxShadow, Color, Offset};
/// 
/// let shadow = BoxShadow::new(
///     Color::rgba(0, 0, 0, 80),
///     Offset::new(4.0, 4.0),
///     8.0,  // blur_radius
///     0.0,  // spread_radius
/// );
/// 
/// AnimatedBoxShadow::new(child, shadow)
///     .with_duration(0.3)
///     .with_curve(Curve::EaseInOut)
/// ```
#[derive(Debug)]
pub struct AnimatedBoxShadow {
    /// Child widget
    pub child: Box<dyn Widget>,
    /// Target box shadow
    pub shadow: BoxShadow,
    /// Animation duration in seconds
    pub duration: f32,
    /// Animation curve
    pub curve: Curve,
    /// Optional widget key
    pub key: Option<WidgetKey>,
}

impl AnimatedBoxShadow {
    /// Create a new animated box shadow widget
    pub fn new(child: impl Widget + 'static, shadow: BoxShadow) -> Self {
        Self {
            child: Box::new(child),
            shadow,
            duration: 0.3,
            curve: Curve::EaseInOut,
            key: None,
        }
    }

    /// Create with no shadow (transparent)
    pub fn none(child: impl Widget + 'static) -> Self {
        Self::new(
            child,
            BoxShadow::new(Color::transparent(), Offset::zero(), 0.0, 0.0),
        )
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

impl Widget for AnimatedBoxShadow {
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
        Box::new(AnimatedBoxShadowRenderObject {
            state: RenderObjectState::default(),
            child: self.child.create_render_object(),
            current_shadow: self.shadow,
            target_shadow: self.shadow,
            controller: None,
            duration: self.duration,
            curve: self.curve,
        })
    }

    fn update_render_object(&self, render_object: &mut dyn RenderObject) {
        if let Some(ro) = render_object.as_any_mut().downcast_mut::<AnimatedBoxShadowRenderObject>() {
            // Only start animation if shadow changed
            if ro.target_shadow != self.shadow {
                ro.target_shadow = self.shadow;
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
        Box::new(AnimatedBoxShadow {
            child: self.child.clone_boxed(),
            shadow: self.shadow,
            duration: self.duration,
            curve: self.curve,
            key: self.key.clone(),
        })
    }
}

/// RenderObject for AnimatedBoxShadow
#[derive(Debug)]
pub struct AnimatedBoxShadowRenderObject {
    state: RenderObjectState,
    child: Box<dyn RenderObject>,
    current_shadow: BoxShadow,
    target_shadow: BoxShadow,
    controller: Option<AnimationController<BoxShadow>>,
    duration: f32,
    curve: Curve,
}

impl AnimatedBoxShadowRenderObject {
    fn start_animation(&mut self) {
        let tween = Tween::new(self.current_shadow, self.target_shadow)
            .with_duration(self.duration)
            .with_curve(self.curve);
        
        let mut controller = AnimationController::new(tween);
        controller.play();
        self.controller = Some(controller);
    }

    /// Get current shadow state
    pub fn current_shadow(&self) -> &BoxShadow {
        &self.current_shadow
    }
}

impl Animatable for AnimatedBoxShadowRenderObject {
    fn update(&mut self, delta: f32) {
        if let Some(ref mut controller) = self.controller {
            controller.update(delta);
            self.current_shadow = controller.value();
            
            if controller.is_completed() {
                self.current_shadow = self.target_shadow;
                self.controller = None;
            }
        }
    }

    fn is_animating(&self) -> bool {
        self.controller.as_ref().map_or(false, |c| c.is_animating())
    }
}

impl Layoutable for AnimatedBoxShadowRenderObject {
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

impl Paintable for AnimatedBoxShadowRenderObject {
    fn paint(&self, painter: &mut dyn Painter) {
        painter.save();
        painter.translate(self.state.offset);
        
        // Draw the shadow behind the child
        let shadow = &self.current_shadow;
        let child_size = self.child.get_size();
        let child_rect = Rect::from_size(child_size);
        
        // Only draw shadow if it has visible color
        if shadow.color.a > 0.0 {
            let shadow_rect = child_rect
                .translate(shadow.offset)
                .inflate(shadow.spread_radius);
            
            painter.fill_blurred_rect(shadow_rect, shadow.color, shadow.blur_radius);
        }
        
        // Draw the child on top
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

impl EventHandlable for AnimatedBoxShadowRenderObject {
    fn hit_test(&self, position: Offset) -> HitTestResult {
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

impl Lifecycle for AnimatedBoxShadowRenderObject {
    fn on_mount(&mut self) {
        self.child.on_mount();
    }

    fn on_unmount(&mut self) {
        self.child.on_unmount();
    }
}

impl Parent for AnimatedBoxShadowRenderObject {
    fn children(&self) -> Vec<&dyn RenderObject> {
        vec![self.child.as_ref()]
    }

    fn children_mut(&mut self) -> Vec<&mut dyn RenderObject> {
        vec![self.child.as_mut()]
    }
}

impl RenderObject for AnimatedBoxShadowRenderObject {
    fn tick(&mut self, delta: f32) -> bool {
        Animatable::update(self, delta);
        let self_animating = Animatable::is_animating(self);

        let child_animating = self.child.tick(delta);

        if self_animating {
            self.state.mark_needs_paint();
        }

        self_animating || child_animating
    }

    fn is_dynamic(&self) -> bool {
        self.controller.is_some()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
