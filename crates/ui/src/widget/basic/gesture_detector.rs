//! GestureDetector Widget
//!
//! Detects various gestures and emits UI messages.
//!
//! # Example
//!
//! ```ignore
//! GestureDetector::new(some_widget)
//!     .on_tap("my_button")           // Emits UIMessage::Gesture { id: "my_button", kind: Tap }
//!     .on_long_press("my_button")    // Emits UIMessage::Gesture { id: "my_button", kind: LongPress }
//! ```

use std::any::{Any, TypeId};

use hoshimi_types::{Constraints, Offset, Rect, Size};

use crate::events::{EventResult, GestureKind, HitTestResult, InputEvent, UIMessage};
use crate::key::WidgetKey;
use crate::painter::Painter;
use crate::render_object::{
    EventHandlable, Layoutable, Lifecycle, Paintable, Parent, RenderObject, RenderObjectState,
};
use crate::widget::Widget;

/// Configuration for gesture callbacks (ID-based)
#[derive(Debug, Clone, Default)]
pub struct GestureConfig {
    /// ID for tap gesture
    pub tap_id: Option<String>,
    
    /// ID for double tap gesture
    pub double_tap_id: Option<String>,
    
    /// ID for long press gesture
    pub long_press_id: Option<String>,
    
    /// ID for press (mouse/touch down) gesture
    pub press_id: Option<String>,
    
    /// ID for release (mouse/touch up) gesture
    pub release_id: Option<String>,
    
    /// ID for pan gesture
    pub pan_id: Option<String>,
}

/// GestureDetector widget for handling user interactions
/// 
/// A simplified gesture detector that uses ID-based callbacks.
/// When a gesture is detected, it emits `UIMessage::Gesture { id, kind }`.
#[derive(Debug)]
pub struct GestureDetector {
    /// Child widget
    pub child: Box<dyn Widget>,
    
    /// Gesture configuration
    pub config: GestureConfig,
    
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
            config: GestureConfig::default(),
            absorb_events: true,
            key: None,
        }
    }
    
    /// Set tap callback with ID
    /// 
    /// When tapped, emits `UIMessage::Gesture { id, kind: GestureKind::Tap }`
    pub fn on_tap(mut self, id: impl Into<String>) -> Self {
        self.config.tap_id = Some(id.into());
        self
    }
    
    /// Set double tap callback with ID
    /// 
    /// When double-tapped, emits `UIMessage::Gesture { id, kind: GestureKind::DoubleTap }`
    pub fn on_double_tap(mut self, id: impl Into<String>) -> Self {
        self.config.double_tap_id = Some(id.into());
        self
    }
    
    /// Set long press callback with ID
    /// 
    /// When long-pressed, emits `UIMessage::Gesture { id, kind: GestureKind::LongPress }`
    pub fn on_long_press(mut self, id: impl Into<String>) -> Self {
        self.config.long_press_id = Some(id.into());
        self
    }
    
    /// Set pan callback with ID
    /// 
    /// When panning, emits `UIMessage::Gesture { id, kind: GestureKind::PanStart/PanUpdate/PanEnd }`
    pub fn on_pan(mut self, id: impl Into<String>) -> Self {
        self.config.pan_id = Some(id.into());
        self
    }
    
    /// Set press callback with ID
    /// 
    /// When pressed (mouse/touch down), emits `UIMessage::Gesture { id, kind: GestureKind::Press }`
    pub fn on_press(mut self, id: impl Into<String>) -> Self {
        self.config.press_id = Some(id.into());
        self
    }
    
    /// Set release callback with ID
    /// 
    /// When released (mouse/touch up), emits `UIMessage::Gesture { id, kind: GestureKind::Release }`
    pub fn on_release(mut self, id: impl Into<String>) -> Self {
        self.config.release_id = Some(id.into());
        self
    }
    
    /// Set whether to absorb events (default: true)
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
            self.config.clone(),
            self.absorb_events,
        ))
    }
    
    fn update_render_object(&self, render_object: &mut dyn RenderObject) {
        if let Some(gesture_ro) = render_object.as_any_mut().downcast_mut::<GestureDetectorRenderObject>() {
            gesture_ro.config = self.config.clone();
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

    fn clone_boxed(&self) -> Box<dyn Widget> {
        Box::new(GestureDetector {
            child: self.child.clone_boxed(),
            config: self.config.clone(),
            absorb_events: self.absorb_events,
            key: self.key.clone(),
        })
    }
}

/// Render object for GestureDetector widget
#[derive(Debug)]
pub struct GestureDetectorRenderObject {
    state: RenderObjectState,
    child: Box<dyn RenderObject>,
    config: GestureConfig,
    absorb_events: bool,
    
    /// Hover state
    is_hovered: bool,
    
    /// Press state
    is_pressed: bool,
    
    /// Pan state for tracking drag gestures
    is_panning: bool,
}

impl GestureDetectorRenderObject {
    fn new(
        child: Box<dyn RenderObject>,
        config: GestureConfig,
        absorb_events: bool,
    ) -> Self {
        Self {
            state: RenderObjectState::new(),
            child,
            config,
            absorb_events,
            is_hovered: false,
            is_pressed: false,
            is_panning: false,
        }
    }
}

impl Layoutable for GestureDetectorRenderObject {
    fn layout(&mut self, constraints: Constraints) -> Size {
        // GestureDetector takes the size of its child
        let child_size = self.child.layout(constraints);
        self.child.set_offset(Offset::zero());

        self.state.size = child_size;
        self.state.needs_layout = false;

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

impl Paintable for GestureDetectorRenderObject {
    fn paint(&self, painter: &mut dyn Painter) {
        painter.save();
        painter.translate(self.state.offset);
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

impl EventHandlable for GestureDetectorRenderObject {
    fn hit_test(&self, position: Offset) -> HitTestResult {
        // Use local rect (at origin) since position is in local coordinates
        let rect = Rect::from_size(self.state.size);

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
        // Use local rect (at origin) since event position is in local coordinates
        let local_rect = Rect::from_size(self.state.size);

        match event {
            InputEvent::Tap { position } => {
                if local_rect.contains(*position) {
                    if let Some(id) = self.config.tap_id.clone() {
                        return EventResult::Message(UIMessage::Gesture {
                            id,
                            kind: GestureKind::Tap,
                        });
                    }
                    return EventResult::Handled;
                }
            }

            InputEvent::LongPress { position } => {
                if local_rect.contains(*position) {
                    if let Some(id) = self.config.long_press_id.clone() {
                        return EventResult::Message(UIMessage::Gesture {
                            id,
                            kind: GestureKind::LongPress,
                        });
                    }
                    return EventResult::Handled;
                }
            }

            InputEvent::Hover { position, entered } => {
                if local_rect.contains(*position) {
                    self.is_hovered = *entered;
                    return EventResult::Handled;
                }
            }

            InputEvent::MouseDown { position, .. } => {
                if local_rect.contains(*position) {
                    self.is_pressed = true;

                    // Emit press event if configured
                    if let Some(id) = self.config.press_id.clone() {
                        return EventResult::Message(UIMessage::Gesture {
                            id,
                            kind: GestureKind::Press,
                        });
                    }

                    // Start pan if configured
                    if let Some(id) = self.config.pan_id.clone() {
                        self.is_panning = true;
                        return EventResult::Message(UIMessage::Gesture {
                            id,
                            kind: GestureKind::PanStart,
                        });
                    }

                    return EventResult::Handled;
                }
            }

            InputEvent::MouseMove { position, .. } => {
                // Handle pan update
                if self.is_panning {
                    if let Some(id) = self.config.pan_id.clone() {
                        return EventResult::Message(UIMessage::Gesture {
                            id,
                            kind: GestureKind::PanUpdate,
                        });
                    }
                }

                // Update hover state
                let was_hovered = self.is_hovered;
                self.is_hovered = local_rect.contains(*position);
                if was_hovered != self.is_hovered {
                    return EventResult::Handled;
                }
            }

            InputEvent::MouseUp { position, .. } => {
                // End pan if panning
                if self.is_panning {
                    self.is_panning = false;
                    if let Some(id) = self.config.pan_id.clone() {
                        return EventResult::Message(UIMessage::Gesture {
                            id,
                            kind: GestureKind::PanEnd,
                        });
                    }
                }

                // Emit release event if configured and was pressed
                if self.is_pressed {
                    self.is_pressed = false;
                    if let Some(id) = self.config.release_id.clone() {
                        return EventResult::Message(UIMessage::Gesture {
                            id,
                            kind: GestureKind::Release,
                        });
                    }
                    if local_rect.contains(*position) && self.config.tap_id.is_some() {
                        return EventResult::Handled;
                    }
                }
            }

            _ => {}
        }

        EventResult::Ignored
    }
}

impl Lifecycle for GestureDetectorRenderObject {
    fn on_mount(&mut self) {
        self.child.on_mount();
    }

    fn on_unmount(&mut self) {
        self.child.on_unmount();
    }
}

impl Parent for GestureDetectorRenderObject {
    fn children(&self) -> Vec<&dyn RenderObject> {
        vec![self.child.as_ref()]
    }

    fn children_mut(&mut self) -> Vec<&mut dyn RenderObject> {
        vec![self.child.as_mut()]
    }
}

impl RenderObject for GestureDetectorRenderObject {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
