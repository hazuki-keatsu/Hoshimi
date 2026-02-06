//! GestureDetector Widget
//!
//! Detects various gestures and emits UI messages.

use std::any::{Any, TypeId};

use hoshimi_shared::{Constraints, Offset, Size};

use crate::events::{EventResult, HitTestResult, InputEvent, UIMessage};
use crate::key::WidgetKey;
use crate::painter::Painter;
use crate::render::{RenderObject, RenderObjectState};
use crate::widget::Widget;
use crate::impl_render_object_common;

/// Gesture types that can be detected
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GestureType {
    Tap,
    DoubleTap,
    LongPress,
    Pan,
}

/// Configuration for gesture callbacks
#[derive(Debug, Clone)]
pub struct GestureCallbacks {
    /// Message to send on tap
    pub on_tap: Option<UIMessage>,
    
    /// Message to send on double tap
    pub on_double_tap: Option<UIMessage>,
    
    /// Message to send on long press
    pub on_long_press: Option<UIMessage>,
    
    /// Button ID for button click messages
    pub button_id: Option<String>,
}

impl Default for GestureCallbacks {
    fn default() -> Self {
        Self {
            on_tap: None,
            on_double_tap: None,
            on_long_press: None,
            button_id: None,
        }
    }
}

/// GestureDetector widget for handling user interactions
#[derive(Debug)]
pub struct GestureDetector {
    /// Child widget
    pub child: Box<dyn Widget>,
    
    /// Gesture callbacks configuration
    pub callbacks: GestureCallbacks,
    
    /// Whether the gesture detector absorbs events
    pub absorb_events: bool,
    
    /// Optional widget key
    pub key: Option<WidgetKey>,
}

impl GestureDetector {
    /// Create a new gesture detector wrapping a child widget
    pub fn new(child: impl Widget + 'static) -> Self {
        Self {
            child: Box::new(child),
            callbacks: GestureCallbacks::default(),
            absorb_events: true,
            key: None,
        }
    }
    
    /// Set tap callback (sends DialogConfirm message)
    pub fn on_tap_confirm(mut self) -> Self {
        self.callbacks.on_tap = Some(UIMessage::DialogConfirm);
        self
    }
    
    /// Set tap callback with button ID
    pub fn on_tap_button(mut self, button_id: impl Into<String>) -> Self {
        let id = button_id.into();
        self.callbacks.button_id = Some(id.clone());
        self.callbacks.on_tap = Some(UIMessage::ButtonClick { id });
        self
    }
    
    /// Set tap callback with custom message
    pub fn on_tap(mut self, message: UIMessage) -> Self {
        self.callbacks.on_tap = Some(message);
        self
    }
    
    /// Set double tap callback
    pub fn on_double_tap(mut self, message: UIMessage) -> Self {
        self.callbacks.on_double_tap = Some(message);
        self
    }
    
    /// Set long press callback
    pub fn on_long_press(mut self, message: UIMessage) -> Self {
        self.callbacks.on_long_press = Some(message);
        self
    }
    
    /// Set whether to absorb events
    pub fn with_absorb_events(mut self, absorb: bool) -> Self {
        self.absorb_events = absorb;
        self
    }
    
    /// Set the widget key
    pub fn with_key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.key = Some(key.into());
        self
    }
}

impl Widget for GestureDetector {
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
        let child_ro = self.child.create_render_object();
        
        Box::new(GestureDetectorRenderObject::new(
            child_ro,
            self.callbacks.clone(),
            self.absorb_events,
        ))
    }
    
    fn update_render_object(&self, render_object: &mut dyn RenderObject) {
        if let Some(gesture_ro) = render_object.as_any_mut().downcast_mut::<GestureDetectorRenderObject>() {
            gesture_ro.callbacks = self.callbacks.clone();
            gesture_ro.absorb_events = self.absorb_events;
        }
    }
    
    fn should_update(&self, old: &dyn Widget) -> bool {
        if let Some(old_gesture) = old.as_any().downcast_ref::<GestureDetector>() {
            self.absorb_events != old_gesture.absorb_events
        } else {
            true
        }
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Render object for GestureDetector widget
#[derive(Debug)]
pub struct GestureDetectorRenderObject {
    state: RenderObjectState,
    child: Box<dyn RenderObject>,
    callbacks: GestureCallbacks,
    absorb_events: bool,
    
    /// Messages to emit (collected during event handling)
    pending_messages: Vec<UIMessage>,
    
    /// Hover state
    is_hovered: bool,
    
    /// Press state
    is_pressed: bool,
}

impl GestureDetectorRenderObject {
    fn new(
        child: Box<dyn RenderObject>,
        callbacks: GestureCallbacks,
        absorb_events: bool,
    ) -> Self {
        Self {
            state: RenderObjectState::new(),
            child,
            callbacks,
            absorb_events,
            pending_messages: Vec::new(),
            is_hovered: false,
            is_pressed: false,
        }
    }
    
    /// Take pending messages (called by the tree after event processing)
    pub fn take_messages(&mut self) -> Vec<UIMessage> {
        std::mem::take(&mut self.pending_messages)
    }
}

impl RenderObject for GestureDetectorRenderObject {
    impl_render_object_common!(state);
    
    fn layout(&mut self, constraints: Constraints) -> Size {
        // GestureDetector takes the size of its child
        let child_size = self.child.layout(constraints);
        self.child.set_offset(Offset::zero());
        
        self.state.size = child_size;
        self.state.needs_layout = false;
        
        child_size
    }
    
    fn paint(&self, painter: &mut dyn Painter) {
        painter.save();
        painter.translate(self.state.offset);
        self.child.paint(painter);
        painter.restore();
    }
    
    fn hit_test(&self, position: Offset) -> HitTestResult {
        let rect = self.state.get_rect();
        
        if rect.contains(position) {
            if self.absorb_events {
                HitTestResult::Hit
            } else {
                HitTestResult::HitTransparent
            }
        } else {
            HitTestResult::Miss
        }
    }
    
    fn handle_event(&mut self, event: &InputEvent) -> EventResult {
        match event {
            InputEvent::Tap { position } => {
                let rect = self.state.get_rect();
                if rect.contains(*position) {
                    if let Some(ref message) = self.callbacks.on_tap {
                        self.pending_messages.push(message.clone());
                        return EventResult::Consumed;
                    }
                    return EventResult::Handled;
                }
            }
            
            InputEvent::LongPress { position } => {
                let rect = self.state.get_rect();
                if rect.contains(*position) {
                    if let Some(ref message) = self.callbacks.on_long_press {
                        self.pending_messages.push(message.clone());
                        return EventResult::Consumed;
                    }
                    return EventResult::Handled;
                }
            }
            
            InputEvent::Hover { position, entered } => {
                let rect = self.state.get_rect();
                if rect.contains(*position) {
                    self.is_hovered = *entered;
                    return EventResult::Handled;
                }
            }
            
            InputEvent::MouseDown { position, .. } => {
                let rect = self.state.get_rect();
                if rect.contains(*position) {
                    self.is_pressed = true;
                    return EventResult::Handled;
                }
            }
            
            InputEvent::MouseUp { position, .. } => {
                if self.is_pressed {
                    self.is_pressed = false;
                    let rect = self.state.get_rect();
                    if rect.contains(*position) {
                        // This was a tap
                        if let Some(ref message) = self.callbacks.on_tap {
                            self.pending_messages.push(message.clone());
                            return EventResult::Consumed;
                        }
                    }
                }
            }
            
            _ => {}
        }
        
        EventResult::Ignored
    }
    
    fn children(&self) -> Vec<&dyn RenderObject> {
        vec![self.child.as_ref()]
    }
    
    fn children_mut(&mut self) -> Vec<&mut dyn RenderObject> {
        vec![self.child.as_mut()]
    }
    
    fn add_child(&mut self, child: Box<dyn RenderObject>) {
        self.child = child;
    }
    
    fn child_count(&self) -> usize {
        1
    }
}
