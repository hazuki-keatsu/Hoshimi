//! Slide Transition Widget
//!
//! A widget that slides its child in from a direction.

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

/// Direction for slide transitions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SlideDirection {
    /// Slide from left
    Left,
    /// Slide from right
    #[default]
    Right,
    /// Slide from top
    Top,
    /// Slide from bottom
    Bottom,
}

impl SlideDirection {
    /// Convert direction to offset multiplier
    fn to_offset(&self, size: Size) -> Offset {
        match self {
            SlideDirection::Left => Offset::new(-size.width, 0.0),
            SlideDirection::Right => Offset::new(size.width, 0.0),
            SlideDirection::Top => Offset::new(0.0, -size.height),
            SlideDirection::Bottom => Offset::new(0.0, size.height),
        }
    }
}

/// A widget that slides its child in/out based on visibility
#[derive(Debug)]
pub struct SlideTransition {
    /// Child widget
    pub child: Box<dyn Widget>,
    /// Whether the child should be visible (slid in)
    pub visible: bool,
    /// Direction to slide from
    pub direction: SlideDirection,
    /// Animation duration in seconds
    pub duration: f32,
    /// Animation curve
    pub curve: Curve,
    /// Optional widget key
    pub key: Option<WidgetKey>,
}

impl SlideTransition {
    /// Create a new slide transition widget
    pub fn new(child: impl Widget + 'static, visible: bool) -> Self {
        Self {
            child: Box::new(child),
            visible,
            direction: SlideDirection::Right,
            duration: 0.3,
            curve: Curve::EaseInOut,
            key: None,
        }
    }

    /// Create a visible slide transition
    pub fn visible(child: impl Widget + 'static) -> Self {
        Self::new(child, true)
    }

    /// Create a hidden slide transition
    pub fn hidden(child: impl Widget + 'static) -> Self {
        Self::new(child, false)
    }

    /// Set the slide direction
    pub fn from_direction(mut self, direction: SlideDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Slide from left
    pub fn from_left(mut self) -> Self {
        self.direction = SlideDirection::Left;
        self
    }

    /// Slide from right
    pub fn from_right(mut self) -> Self {
        self.direction = SlideDirection::Right;
        self
    }

    /// Slide from top
    pub fn from_top(mut self) -> Self {
        self.direction = SlideDirection::Top;
        self
    }

    /// Slide from bottom
    pub fn from_bottom(mut self) -> Self {
        self.direction = SlideDirection::Bottom;
        self
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

impl Widget for SlideTransition {
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
        // Start from off-screen if visible=true (will animate in on_mount)
        Box::new(SlideTransitionRenderObject {
            state: RenderObjectState::default(),
            child: self.child.create_render_object(),
            slide_progress: if self.visible { 1.0 } else { 1.0 },
            visible: self.visible,
            direction: self.direction,
            controller: None,
            duration: self.duration,
            curve: self.curve,
            needs_entrance_animation: self.visible,
        })
    }

    fn update_render_object(&self, render_object: &mut dyn RenderObject) {
        if let Some(ro) = render_object.as_any_mut().downcast_mut::<SlideTransitionRenderObject>() {
            // Start animation if visibility changed
            if ro.visible != self.visible {
                ro.visible = self.visible;
                ro.direction = self.direction;
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
        Box::new(SlideTransition {
            child: self.child.clone_boxed(),
            visible: self.visible,
            direction: self.direction,
            duration: self.duration,
            curve: self.curve,
            key: self.key.clone(),
        })
    }
}

/// RenderObject for SlideTransition
#[derive(Debug)]
pub struct SlideTransitionRenderObject {
    state: RenderObjectState,
    child: Box<dyn RenderObject>,
    /// Progress of slide (0 = fully visible, 1 = fully hidden)
    slide_progress: f32,
    visible: bool,
    direction: SlideDirection,
    controller: Option<AnimationController<f32>>,
    duration: f32,
    curve: Curve,
    /// Whether entrance animation should play on mount
    needs_entrance_animation: bool,
}

impl SlideTransitionRenderObject {
    fn start_animation(&mut self) {
        let (start, end) = if self.visible {
            (self.slide_progress, 0.0) // Slide in
        } else {
            (self.slide_progress, 1.0) // Slide out
        };
        
        let tween = Tween::new(start, end)
            .with_duration(self.duration)
            .with_curve(self.curve);
        
        let mut controller = AnimationController::new(tween);
        controller.play();
        self.controller = Some(controller);
    }

    /// Check if currently visible (not fully slid out)
    pub fn is_visible(&self) -> bool {
        self.slide_progress < 1.0
    }

    /// Get current slide offset
    fn current_offset(&self) -> Offset {
        let full_offset = self.direction.to_offset(self.state.size);
        Offset::new(
            full_offset.x * self.slide_progress,
            full_offset.y * self.slide_progress,
        )
    }
}

impl Animatable for SlideTransitionRenderObject {
    fn update(&mut self, delta: f32) {
        if let Some(ref mut controller) = self.controller {
            controller.update(delta);
            self.slide_progress = controller.value();
            
            if controller.is_completed() {
                self.slide_progress = if self.visible { 0.0 } else { 1.0 };
                self.controller = None;
            }
        }
    }

    fn is_animating(&self) -> bool {
        self.controller.as_ref().map_or(false, |c| c.is_animating())
    }
}

impl Layoutable for SlideTransitionRenderObject {
    // Custom layout to start animation after first layout
    fn layout(&mut self, constraints: Constraints, text_measurer: &dyn TextMeasurer) -> Size {
        let child_size = self.child.layout(constraints, text_measurer);
        self.child.set_offset(Offset::ZERO);
        self.state.size = child_size;

        // Start entrance animation after first layout when size is known
        if self.needs_entrance_animation && child_size.width > 0.0 && child_size.height > 0.0 {
            self.needs_entrance_animation = false;
            self.start_animation();
        }

        child_size
    }

    // Custom get_rect to include slide offset
    fn get_rect(&self) -> Rect {
        let offset = self.current_offset();
        Rect::new(
            self.state.offset.x + offset.x,
            self.state.offset.y + offset.y,
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

    fn needs_layout(&self) -> bool {
        self.state.needs_layout
    }

    fn mark_needs_layout(&mut self) {
        self.state.needs_layout = true;
    }
}

impl Paintable for SlideTransitionRenderObject {
    fn paint(&self, painter: &mut dyn Painter) {
        if self.slide_progress >= 1.0 {
            return;
        }

        painter.save();
        // First translate to own position, then apply slide offset
        painter.translate(self.state.offset);
        let offset = self.current_offset();
        painter.translate(offset);
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

impl EventHandlable for SlideTransitionRenderObject {
    fn hit_test(&self, position: Offset) -> HitTestResult {
        // Only accept hits when fully visible
        if self.slide_progress > 0.0 {
            return HitTestResult::Miss;
        }

        let local = Offset::new(
            position.x - self.state.offset.x,
            position.y - self.state.offset.y,
        );
        self.child.hit_test(local)
    }

    fn handle_event(&mut self, event: &InputEvent) -> EventResult {
        if self.slide_progress > 0.0 {
            return EventResult::Ignored;
        }
        self.child.handle_event(event)
    }
}

impl Lifecycle for SlideTransitionRenderObject {
    fn on_mount(&mut self) {
        self.child.on_mount();
        // Note: entrance animation is started in layout() after size is known
    }

    fn on_unmount(&mut self) {
        self.child.on_unmount();
    }
}

impl Parent for SlideTransitionRenderObject {
    fn children(&self) -> Vec<&dyn RenderObject> {
        vec![self.child.as_ref()]
    }

    fn children_mut(&mut self) -> Vec<&mut dyn RenderObject> {
        vec![self.child.as_mut()]
    }
}

impl RenderObject for SlideTransitionRenderObject {
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
