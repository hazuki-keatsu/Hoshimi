//! UI Tree Module
//!
//! Manages the widget and render object trees.

use std::collections::HashMap;

use hoshimi_shared::{Constraints, Size};
use tracing::{debug, trace};

use crate::events::{EventResult, InputEvent, UIMessage};
use crate::key::WidgetKey;
use crate::painter::Painter;
use crate::render::RenderObject;
use crate::widget::Widget;

/// UI Tree that manages the widget hierarchy
pub struct UiTree {
    /// Root render object
    root: Option<Box<dyn RenderObject>>,
    
    /// Key-based render object cache for diffing
    key_cache: HashMap<WidgetKey, usize>,
    
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
            key_cache: HashMap::new(),
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
        
        // Create render object from widget
        let render_object = widget.create_render_object();
        self.root = Some(render_object);
        self.key_cache.clear();
        self.needs_layout = true;
        self.needs_paint = true;
    }
    
    /// Update the root widget (performs diffing)
    pub fn update_root(&mut self, widget: &dyn Widget) {
        if let Some(ref mut root) = self.root {
            // Check if we can update in place
            if widget.should_update(widget) {
                trace!("Updating root widget in place");
                widget.update_render_object(root.as_mut());
            } else {
                // Need to rebuild
                debug!("Rebuilding root widget");
                self.root = Some(widget.create_render_object());
            }
            self.needs_layout = true;
            self.needs_paint = true;
        } else {
            // No existing root, create new one
            self.root = Some(widget.create_render_object());
            self.needs_layout = true;
            self.needs_paint = true;
        }
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
    pub fn layout_if_needed(&mut self) {
        if !self.needs_layout {
            return;
        }
        
        self.layout();
    }
    
    /// Force layout
    pub fn layout(&mut self) {
        if let Some(ref mut root) = self.root {
            trace!("Performing layout with constraints: {:?}", self.constraints);
            self.last_size = root.layout(self.constraints);
            self.needs_layout = false;
            self.needs_paint = true;
        }
    }
    
    /// Paint the tree
    pub fn paint(&mut self, painter: &mut dyn Painter) {
        self.layout_if_needed();
        
        if let Some(ref root) = self.root {
            trace!("Painting UI tree");
            root.paint(painter);
            self.needs_paint = false;
        }
    }
    
    /// Handle input event
    pub fn handle_event(&mut self, event: &InputEvent) -> EventResult {
        if let Some(ref mut root) = self.root {
            let result = root.handle_event(event);
            
            // Collect any messages
            if let EventResult::Message(msg) = &result {
                self.pending_messages.push(msg.clone());
            }
            
            result
        } else {
            EventResult::Ignored
        }
    }
    
    /// Hit test at position
    pub fn hit_test(&self, x: f32, y: f32) -> bool {
        if let Some(ref root) = self.root {
            root.hit_test(hoshimi_shared::Offset::new(x, y)).is_hit()
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
