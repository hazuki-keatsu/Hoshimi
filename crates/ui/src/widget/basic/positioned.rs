//! Positioned Widget
//!
//! Widget to position a child within a Stack.

use std::any::{Any, TypeId};

use hoshimi_shared::{Constraints, Offset, Size};

use crate::key::WidgetKey;
use crate::painter::Painter;
use crate::render::{RenderObject, RenderObjectState};
use crate::widget::Widget;
use crate::impl_render_object_common;

/// Positioned widget that positions its child relative to a Stack
#[derive(Debug)]
pub struct Positioned {
    /// Child widget
    pub child: Box<dyn Widget>,
    
    /// Distance from the left edge
    pub left: Option<f32>,
    
    /// Distance from the top edge
    pub top: Option<f32>,
    
    /// Distance from the right edge
    pub right: Option<f32>,
    
    /// Distance from the bottom edge
    pub bottom: Option<f32>,
    
    /// Fixed width
    pub width: Option<f32>,
    
    /// Fixed height
    pub height: Option<f32>,
    
    /// Optional widget key
    pub key: Option<WidgetKey>,
}

impl Positioned {
    /// Create a new positioned widget
    pub fn new(child: impl Widget + 'static) -> Self {
        Self {
            child: Box::new(child),
            left: None,
            top: None,
            right: None,
            bottom: None,
            width: None,
            height: None,
            key: None,
        }
    }
    
    /// Position with all edges specified
    pub fn fill(child: impl Widget + 'static) -> Self {
        Self {
            child: Box::new(child),
            left: Some(0.0),
            top: Some(0.0),
            right: Some(0.0),
            bottom: Some(0.0),
            width: None,
            height: None,
            key: None,
        }
    }
    
    /// Position at a specific point with optional size
    pub fn at(child: impl Widget + 'static, left: f32, top: f32) -> Self {
        Self {
            child: Box::new(child),
            left: Some(left),
            top: Some(top),
            right: None,
            bottom: None,
            width: None,
            height: None,
            key: None,
        }
    }
    
    /// Set left distance
    pub fn with_left(mut self, left: f32) -> Self {
        self.left = Some(left);
        self
    }
    
    /// Set top distance
    pub fn with_top(mut self, top: f32) -> Self {
        self.top = Some(top);
        self
    }
    
    /// Set right distance
    pub fn with_right(mut self, right: f32) -> Self {
        self.right = Some(right);
        self
    }
    
    /// Set bottom distance
    pub fn with_bottom(mut self, bottom: f32) -> Self {
        self.bottom = Some(bottom);
        self
    }
    
    /// Set width
    pub fn with_width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }
    
    /// Set height
    pub fn with_height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }
    
    /// Set the widget key
    pub fn with_key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.key = Some(key.into());
        self
    }
}

impl Widget for Positioned {
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
        Box::new(PositionedRenderObject::new(
            self.child.create_render_object(),
            self.left,
            self.top,
            self.right,
            self.bottom,
            self.width,
            self.height,
        ))
    }
    
    fn update_render_object(&self, render_object: &mut dyn RenderObject) {
        if let Some(pos_ro) = render_object.as_any_mut().downcast_mut::<PositionedRenderObject>() {
            pos_ro.left = self.left;
            pos_ro.top = self.top;
            pos_ro.right = self.right;
            pos_ro.bottom = self.bottom;
            pos_ro.width = self.width;
            pos_ro.height = self.height;
            pos_ro.state.mark_needs_layout();
        }
    }
    
    fn should_update(&self, old: &dyn Widget) -> bool {
        if let Some(old_pos) = old.as_any().downcast_ref::<Positioned>() {
            self.left != old_pos.left ||
            self.top != old_pos.top ||
            self.right != old_pos.right ||
            self.bottom != old_pos.bottom ||
            self.width != old_pos.width ||
            self.height != old_pos.height
        } else {
            true
        }
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_boxed(&self) -> Box<dyn Widget> {
        Box::new(Positioned {
            child: self.child.clone_boxed(),
            left: self.left,
            top: self.top,
            right: self.right,
            bottom: self.bottom,
            width: self.width,
            height: self.height,
            key: self.key.clone(),
        })
    }
}

/// Render object for Positioned widget
#[derive(Debug)]
pub struct PositionedRenderObject {
    state: RenderObjectState,
    child: Box<dyn RenderObject>,
    left: Option<f32>,
    top: Option<f32>,
    right: Option<f32>,
    bottom: Option<f32>,
    width: Option<f32>,
    height: Option<f32>,
}

impl PositionedRenderObject {
    fn new(
        child: Box<dyn RenderObject>,
        left: Option<f32>,
        top: Option<f32>,
        right: Option<f32>,
        bottom: Option<f32>,
        width: Option<f32>,
        height: Option<f32>,
    ) -> Self {
        Self {
            state: RenderObjectState::new(),
            child,
            left,
            top,
            right,
            bottom,
            width,
            height,
        }
    }
    
    /// Check if this positioned widget has position info
    pub fn is_positioned(&self) -> bool {
        self.left.is_some() || self.top.is_some() ||
        self.right.is_some() || self.bottom.is_some()
    }
}

impl RenderObject for PositionedRenderObject {
    impl_render_object_common!(state);
    
    fn layout(&mut self, constraints: Constraints) -> Size {
        // Calculate child constraints based on position
        let child_constraints = self.compute_child_constraints(constraints);
        
        let child_size = self.child.layout(child_constraints);
        
        // Calculate position
        let x = if let Some(left) = self.left {
            left
        } else if let Some(right) = self.right {
            constraints.max_width - right - child_size.width
        } else {
            0.0
        };
        
        let y = if let Some(top) = self.top {
            top
        } else if let Some(bottom) = self.bottom {
            constraints.max_height - bottom - child_size.height
        } else {
            0.0
        };
        
        self.child.set_offset(Offset::new(x, y));
        
        // The positioned widget's size is determined by its position within the parent
        let width = if let (Some(left), Some(right)) = (self.left, self.right) {
            constraints.max_width - left - right
        } else {
            self.width.unwrap_or(child_size.width)
        };
        
        let height = if let (Some(top), Some(bottom)) = (self.top, self.bottom) {
            constraints.max_height - top - bottom
        } else {
            self.height.unwrap_or(child_size.height)
        };
        
        let size = Size::new(width, height);
        self.state.size = size;
        self.state.needs_layout = false;
        
        size
    }
    
    fn paint(&self, painter: &mut dyn Painter) {
        painter.save();
        painter.translate(self.state.offset);
        self.child.paint(painter);
        painter.restore();
    }
    
    fn children(&self) -> Vec<&dyn RenderObject> {
        vec![self.child.as_ref()]
    }
    
    fn children_mut(&mut self) -> Vec<&mut dyn RenderObject> {
        vec![self.child.as_mut()]
    }
    
    fn add_child(&mut self, child: Box<dyn RenderObject>) {
        self.child = child;
        self.state.mark_needs_layout();
    }
}

impl PositionedRenderObject {
    fn compute_child_constraints(&self, parent_constraints: Constraints) -> Constraints {
        // If both left and right are specified, width is determined
        let (min_width, max_width) = if let Some(width) = self.width {
            (width, width)
        } else if let (Some(left), Some(right)) = (self.left, self.right) {
            let width = (parent_constraints.max_width - left - right).max(0.0);
            (width, width)
        } else {
            (0.0, parent_constraints.max_width)
        };
        
        // If both top and bottom are specified, height is determined
        let (min_height, max_height) = if let Some(height) = self.height {
            (height, height)
        } else if let (Some(top), Some(bottom)) = (self.top, self.bottom) {
            let height = (parent_constraints.max_height - top - bottom).max(0.0);
            (height, height)
        } else {
            (0.0, parent_constraints.max_height)
        };
        
        Constraints::new(min_width, max_width, min_height, max_height)
    }
}
