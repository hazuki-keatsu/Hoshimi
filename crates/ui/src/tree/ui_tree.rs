//! UI Tree Module
//!
//! Manages the widget and render object trees with incremental updates
//! using the diff and reconciler system.

use std::collections::HashMap;

use hoshimi_types::{Constraints, Size};
use tracing::{debug, trace};

use crate::events::{EventResult, InputEvent, UIMessage};
use crate::gesture::{GestureConfig, InputEventQueue};
use crate::key::WidgetKey;
use crate::painter::{Painter, TextMeasurer};
use crate::render_object::RenderObject;
use crate::widget::Widget;

use super::diff::WidgetDiffer;
use super::reconciler::Reconciler;

/// UI Tree that manages the widget hierarchy
pub struct UiTree {
    /// Root render object
    root: Option<Box<dyn RenderObject>>,
    
    /// Cached widget tree for diffing
    root_widget: Option<Box<dyn Widget>>,
    
    /// Key-based render object cache for diffing
    key_cache: HashMap<WidgetKey, usize>,
    
    /// Input event queue with gesture detection
    event_queue: InputEventQueue,
    
    /// Pending messages from event handling
    pending_messages: Vec<UIMessage>,
    
    /// Tree size constraints
    constraints: Constraints,
    
    /// Last computed size
    last_size: Size,
    
    /// Whether the tree needs layout
    needs_layout: bool,
    
    /// Whether the tree needs paint
    needs_paint: bool,
}

impl UiTree {
    /// Create a new empty UI tree
    pub fn new() -> Self {
        Self {
            root: None,
            root_widget: None,
            key_cache: HashMap::new(),
            event_queue: InputEventQueue::new(),
            pending_messages: Vec::new(),
            constraints: Constraints::loose(Size::new(800.0, 600.0)),
            last_size: Size::ZERO,
            needs_layout: true,
            needs_paint: true,
        }
    }
    
    /// Create a UI tree with root widget
    pub fn with_root(widget: impl Widget + 'static) -> Self {
        let mut tree = Self::new();
        tree.set_root(widget);
        tree
    }
    
    /// Set the root widget
    pub fn set_root(&mut self, widget: impl Widget + 'static) {
        debug!("Setting new root widget: {:?}", std::any::type_name_of_val(&widget));
        
        // Unmount old tree if exists
        if let Some(ref mut old_root) = self.root {
            Reconciler::unmount_recursive(old_root.as_mut());
        }
        
        // Build new render tree using reconciler
        let render_object = Reconciler::build_tree(&widget);
        
        // Cache widget for future diffing
        self.root_widget = Some(widget.clone_boxed());
        self.root = Some(render_object);
        self.key_cache.clear();
        self.needs_layout = true;
        self.needs_paint = true;
    }
    
    /// Update the root widget using diff algorithm
    /// 
    /// This performs an incremental update by:
    /// 1. Comparing the new widget with the existing tree using WidgetDiffer
    /// 2. Computing minimal diff operations
    /// 3. Applying only necessary changes via the Reconciler
    pub fn update_root(&mut self, widget: &dyn Widget) {
        match (&mut self.root, &self.root_widget) {
            (Some(root), Some(old_widget)) => {
                if let Some(diff) = WidgetDiffer::diff_widget(old_widget.as_ref(), widget) {
                    // Apply diff via reconciler
                    trace!("Applying incremental diff update");
                    Reconciler::reconcile(root.as_mut(), widget, &diff);
                } else {
                    // Types incompatible - full replacement
                    debug!("Widget types incompatible, replacing subtree");
                    let new_root = Reconciler::replace_subtree(root.as_mut(), widget);
                    self.root = Some(new_root);
                }
            }
            _ => {
                // No existing root, build new tree
                debug!("No existing root, building new tree");
                self.root = Some(Reconciler::build_tree(widget));
            }
        }
        self.root_widget = Some(widget.clone_boxed());
        self.needs_layout = true;
        self.needs_paint = true;
    }
    
    /// Update with explicit old and new widget trees
    /// 
    /// This method is provided for cases where you have both the old and new
    /// widget trees available. For most cases, prefer using `update_root()`
    /// which automatically uses the cached widget tree.
    #[deprecated(since = "0.2.0", note = "use update_root() instead, which now caches the widget tree")]
    pub fn update_with_diff(&mut self, _old_widget: &dyn Widget, new_widget: &dyn Widget) {
        // Delegate to update_root which now handles diffing internally
        self.update_root(new_widget);
    }
    
    /// Set size constraints
    pub fn set_constraints(&mut self, constraints: Constraints) {
        if self.constraints != constraints {
            self.constraints = constraints;
            self.needs_layout = true;
        }
    }
    
    /// Set size from width and height
    pub fn set_size(&mut self, width: f32, height: f32) {
        self.set_constraints(Constraints::loose(Size::new(width, height)));
    }
    
    /// Perform layout if needed
    pub fn layout_if_needed(&mut self, text_measurer: &dyn TextMeasurer) {
        if !self.needs_layout {
            return;
        }
        
        self.layout(text_measurer);
    }
    
    /// Force layout
    pub fn layout(&mut self, text_measurer: &dyn TextMeasurer) {
        if let Some(ref mut root) = self.root {
            trace!("Performing layout with constraints: {:?}", self.constraints);
            self.last_size = root.layout(self.constraints, text_measurer);
            self.needs_layout = false;
            self.needs_paint = true;
        }
    }
    
    /// Paint the tree
    pub fn paint(&mut self, painter: &mut dyn Painter) {
        // Painter implements TextMeasurer, so we can use it for layout
        self.layout_if_needed(painter as &dyn TextMeasurer);
        
        if let Some(ref root) = self.root {
            // trace!("Painting UI tree");
            root.paint(painter);
            self.needs_paint = false;
        }
    }
    
    /// Update all animations in the tree with the given delta time
    /// 
    /// This should be called every frame before painting to advance animations.
    /// Returns `true` if any animation is still running (needs more frames).
    /// 
    /// # Arguments
    /// * `delta` - Time elapsed since last frame in seconds
    /// 
    /// # Example
    /// ```ignore
    /// let delta = (now - last_time).as_secs_f32();
    /// if ui_tree.tick(delta) {
    ///     // Animations are running, need to keep rendering
    /// }
    /// ui_tree.paint(&mut painter);
    /// ```
    pub fn tick(&mut self, delta: f32) -> bool {
        if let Some(ref mut root) = self.root {
            let animating = root.tick(delta);
            if animating {
                self.needs_paint = true;
            }
            animating
        } else {
            false
        }
    }
    
    /// Handle input event directly (bypasses event queue)
    /// 
    /// For most cases, prefer using `push_event()` and `process_events()` instead,
    /// which provides automatic gesture detection.
    pub fn handle_event(&mut self, event: &InputEvent) -> EventResult {
        if let Some(ref mut root) = self.root {
            let result = Self::dispatch_event_recursive(root.as_mut(), event, &mut self.pending_messages);
            result
        } else {
            EventResult::Ignored
        }
    }
    
    /// Recursively dispatch an event through the render tree
    /// 
    /// Events are dispatched depth-first (children first), and propagation
    /// stops when a widget consumes the event (returns Handled, Consumed, or Message).
    /// 
    /// Coordinates are transformed to each child's local coordinate space.
    fn dispatch_event_recursive(
        render_object: &mut dyn RenderObject,
        event: &InputEvent,
        messages: &mut Vec<UIMessage>,
    ) -> EventResult {
        // First, try to dispatch to children (depth-first)
        for child in render_object.children_mut() {
            // Transform event coordinates to child's local coordinate space
            let child_offset = child.get_offset();
            let local_event = event.with_offset(child_offset);
            
            let result = Self::dispatch_event_recursive(child, &local_event, messages);
            if result.should_stop() {
                // Messages are already collected by the recursive call
                return result;
            }
        }
        
        // Then let this render object handle the event
        let result = render_object.handle_event(event);
        
        // Collect message if any (only at the source, not when propagating)
        if let EventResult::Message(msg) = &result {
            messages.push(msg.clone());
        }
        
        result
    }
    
    /// Push an input event to the event queue
    /// 
    /// The event will be processed by the gesture detector, which may generate
    /// additional high-level gesture events (Tap, LongPress, etc.)
    /// 
    /// Call `process_events()` to dispatch queued events to the UI tree.
    /// 
    /// # Example
    /// ```ignore
    /// // In your event loop:
    /// ui_tree.push_event(InputEvent::MouseDown { position, button });
    /// 
    /// // Later, process all queued events:
    /// ui_tree.process_events();
    /// ```
    pub fn push_event(&mut self, event: InputEvent) {
        self.event_queue.push(event);
    }
    
    /// Push a raw input event without gesture detection
    /// 
    /// Use this for events that should not trigger gesture detection,
    /// or for pre-processed gesture events.
    pub fn push_event_raw(&mut self, event: InputEvent) {
        self.event_queue.push_raw(event);
    }
    
    /// Process all queued events and dispatch them to the UI tree
    /// 
    /// This method drains the event queue, passing each event to the widget tree
    /// for handling. Generated UIMessages are collected in `pending_messages`.
    /// 
    /// Returns the number of events processed.
    /// 
    /// # Example
    /// ```ignore
    /// // Push events from your platform's event loop
    /// for sdl_event in event_pump.poll_iter() {
    ///     if let Some(input_event) = convert_sdl_event(&sdl_event) {
    ///         ui_tree.push_event(input_event);
    ///     }
    /// }
    /// 
    /// // Process all queued events
    /// ui_tree.process_events();
    /// 
    /// // Handle any generated messages
    /// for message in ui_tree.take_messages() {
    ///     handle_message(message);
    /// }
    /// ```
    pub fn process_events(&mut self) -> usize {
        let mut count = 0;
        
        while let Some(event) = self.event_queue.pop() {
            trace!("Processing event: {:?}", event);
            let result = self.handle_event(&event);
            
            match &result {
                EventResult::Handled | EventResult::Consumed => {
                    trace!("Event handled/consumed");
                }
                EventResult::Message(msg) => {
                    debug!("Event produced message: {:?}", msg);
                }
                EventResult::Ignored => {}
            }
            
            count += 1;
        }
        
        count
    }
    
    /// Check if there are pending events in the queue
    pub fn has_pending_events(&self) -> bool {
        !self.event_queue.is_empty()
    }
    
    /// Get the number of pending events in the queue
    pub fn pending_event_count(&self) -> usize {
        self.event_queue.len()
    }
    
    /// Configure the gesture detector
    /// 
    /// Allows customizing gesture detection thresholds like tap distance
    /// and long press duration.
    pub fn set_gesture_config(&mut self, config: GestureConfig) {
        self.event_queue.gesture_detector_mut().set_config(config);
    }
    
    /// Reset the gesture detector state
    /// 
    /// Call this when focus is lost or gestures should be cancelled.
    pub fn reset_gesture_state(&mut self) {
        self.event_queue.reset_gesture_state();
    }
    
    /// Hit test at position
    pub fn hit_test(&self, x: f32, y: f32) -> bool {
        if let Some(ref root) = self.root {
            root.hit_test(hoshimi_types::Offset::new(x, y)).is_hit()
        } else {
            false
        }
    }
    
    /// Take pending messages
    pub fn take_messages(&mut self) -> Vec<UIMessage> {
        std::mem::take(&mut self.pending_messages)
    }
    
    /// Check if there are pending messages
    pub fn has_messages(&self) -> bool {
        !self.pending_messages.is_empty()
    }
    
    /// Get the last computed size
    pub fn size(&self) -> Size {
        self.last_size
    }
    
    /// Check if layout is needed
    pub fn needs_layout(&self) -> bool {
        self.needs_layout
    }
    
    /// Check if paint is needed
    pub fn needs_paint(&self) -> bool {
        self.needs_paint
    }
    
    /// Mark the tree as needing layout
    pub fn mark_needs_layout(&mut self) {
        self.needs_layout = true;
    }
    
    /// Mark the tree as needing paint
    pub fn mark_needs_paint(&mut self) {
        self.needs_paint = true;
    }
    
    /// Get a reference to the root render object
    pub fn root(&self) -> Option<&dyn RenderObject> {
        self.root.as_ref().map(|r| r.as_ref())
    }
    
    /// Get a mutable reference to the root render object
    pub fn root_mut(&mut self) -> Option<&mut dyn RenderObject> {
        self.root.as_mut().map(|r| r.as_mut())
    }
}

impl Default for UiTree {
    fn default() -> Self {
        Self::new()
    }
}
