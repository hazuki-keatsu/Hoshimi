//! Padding Widget
//!
//! Adds padding around its child.

use std::any::{Any, TypeId};

use hoshimi_shared::{Constraints, EdgeInsets, Offset, Size};

use crate::key::WidgetKey;
use crate::painter::Painter;
use crate::render::{RenderObject, RenderObjectState};
use crate::widget::Widget;
use crate::impl_render_object_common;

/// Padding widget that adds space around its child
#[derive(Debug)]
pub struct Padding {
    /// Child widget
    pub child: Box<dyn Widget>,
    
    /// Padding insets
    pub padding: EdgeInsets,
    
    /// Optional widget key
    pub key: Option<WidgetKey>,
}

impl Padding {
    /// Create a new padding widget
    pub fn new(padding: EdgeInsets, child: impl Widget + 'static) -> Self {
        Self {
            child: Box::new(child),
            padding,
            key: None,
        }
    }
    
    /// Create padding with uniform value on all sides
    pub fn all(value: f32, child: impl Widget + 'static) -> Self {
        Self::new(EdgeInsets::all(value), child)
    }
    
    /// Create padding with symmetric horizontal and vertical values
    pub fn symmetric(horizontal: f32, vertical: f32, child: impl Widget + 'static) -> Self {
        Self::new(EdgeInsets::symmetric(horizontal, vertical), child)
    }
    
    /// Create padding with only horizontal values
    pub fn horizontal(value: f32, child: impl Widget + 'static) -> Self {
        Self::new(EdgeInsets::horizontal(value), child)
    }
    
    /// Create padding with only vertical values
    pub fn vertical(value: f32, child: impl Widget + 'static) -> Self {
        Self::new(EdgeInsets::vertical(value), child)
    }
    
    /// Set the widget key
    pub fn with_key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.key = Some(key.into());
        self
    }
}

impl Widget for Padding {
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
        Box::new(PaddingRenderObject::new(
            self.child.create_render_object(),
            self.padding,
        ))
    }
    
    fn update_render_object(&self, render_object: &mut dyn RenderObject) {
        if let Some(padding_ro) = render_object.as_any_mut().downcast_mut::<PaddingRenderObject>() {
            padding_ro.padding = self.padding;
            padding_ro.state.mark_needs_layout();
        }
    }
    
    fn should_update(&self, old: &dyn Widget) -> bool {
        if let Some(old_padding) = old.as_any().downcast_ref::<Padding>() {
            self.padding != old_padding.padding
        } else {
            true
        }
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Render object for Padding widget
#[derive(Debug)]
pub struct PaddingRenderObject {
    state: RenderObjectState,
    child: Box<dyn RenderObject>,
    padding: EdgeInsets,
}

impl PaddingRenderObject {
    fn new(child: Box<dyn RenderObject>, padding: EdgeInsets) -> Self {
        Self {
            state: RenderObjectState::new(),
            child,
            padding,
        }
    }
}

impl RenderObject for PaddingRenderObject {
    impl_render_object_common!(state);
    
    fn layout(&mut self, constraints: Constraints) -> Size {
        let horizontal_padding = self.padding.left + self.padding.right;
        let vertical_padding = self.padding.top + self.padding.bottom;
        
        // Deflate constraints for child
        let child_constraints = Constraints::new(
            (constraints.min_width - horizontal_padding).max(0.0),
            (constraints.max_width - horizontal_padding).max(0.0),
            (constraints.min_height - vertical_padding).max(0.0),
            (constraints.max_height - vertical_padding).max(0.0),
        );
        
        let child_size = self.child.layout(child_constraints);
        
        // Position child with left/top padding offset
        self.child.set_offset(Offset::new(self.padding.left, self.padding.top));
        
        // Calculate final size
        let size = constraints.constrain(Size::new(
            child_size.width + horizontal_padding,
            child_size.height + vertical_padding,
        ));
        
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
    
    fn child_count(&self) -> usize {
        1
    }
}
