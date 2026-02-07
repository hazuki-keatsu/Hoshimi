//! SizedBox Widget
//!
//! Forces a specific size on its child.

use std::any::{Any, TypeId};

use hoshimi_shared::{Constraints, Offset, Size};

use crate::key::WidgetKey;
use crate::painter::Painter;
use crate::render::{RenderObject, RenderObjectState};
use crate::widget::Widget;
use crate::impl_render_object_common;

/// SizedBox widget that forces a specific size
#[derive(Debug)]
pub struct SizedBox {
    /// Optional child widget
    pub child: Option<Box<dyn Widget>>,
    
    /// Fixed width
    pub width: Option<f32>,
    
    /// Fixed height
    pub height: Option<f32>,
    
    /// Optional widget key
    pub key: Option<WidgetKey>,
}

impl SizedBox {
    /// Create an empty SizedBox with specific size
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            child: None,
            width: Some(width),
            height: Some(height),
            key: None,
        }
    }
    
    /// Create a SizedBox with a child
    pub fn with_child(mut self, child: impl Widget + 'static) -> Self {
        self.child = Some(Box::new(child));
        self
    }
    
    /// Create a SizedBox that expands to fill parent
    pub fn expand() -> Self {
        Self {
            child: None,
            width: None,
            height: None,
            key: None,
        }
    }
    
    /// Create a SizedBox that shrinks to fit child
    pub fn shrink() -> Self {
        Self {
            child: None,
            width: Some(0.0),
            height: Some(0.0),
            key: None,
        }
    }
    
    /// Create a SizedBox with only width specified
    pub fn from_width(width: f32) -> Self {
        Self {
            child: None,
            width: Some(width),
            height: None,
            key: None,
        }
    }
    
    /// Create a SizedBox with only height specified
    pub fn from_height(height: f32) -> Self {
        Self {
            child: None,
            width: None,
            height: Some(height),
            key: None,
        }
    }
    
    /// Create a square SizedBox
    pub fn square(size: f32) -> Self {
        Self::new(size, size)
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

impl Widget for SizedBox {
    fn widget_type(&self) -> TypeId {
        TypeId::of::<Self>()
    }
    
    fn key(&self) -> Option<WidgetKey> {
        self.key.clone()
    }
    
    fn children(&self) -> Vec<&dyn Widget> {
        match &self.child {
            Some(child) => vec![child.as_ref()],
            None => vec![],
        }
    }
    
    fn create_render_object(&self) -> Box<dyn RenderObject> {
        let child_ro = self.child.as_ref().map(|c| c.create_render_object());
        Box::new(SizedBoxRenderObject::new(
            child_ro,
            self.width,
            self.height,
        ))
    }
    
    fn update_render_object(&self, render_object: &mut dyn RenderObject) {
        if let Some(sized_ro) = render_object.as_any_mut().downcast_mut::<SizedBoxRenderObject>() {
            sized_ro.width = self.width;
            sized_ro.height = self.height;
            sized_ro.state.mark_needs_layout();
        }
    }
    
    fn should_update(&self, old: &dyn Widget) -> bool {
        if let Some(old_sized) = old.as_any().downcast_ref::<SizedBox>() {
            self.width != old_sized.width || self.height != old_sized.height
        } else {
            true
        }
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_boxed(&self) -> Box<dyn Widget> {
        Box::new(SizedBox {
            child: self.child.as_ref().map(|c| c.clone_boxed()),
            width: self.width,
            height: self.height,
            key: self.key.clone(),
        })
    }
}

/// Render object for SizedBox widget
#[derive(Debug)]
pub struct SizedBoxRenderObject {
    state: RenderObjectState,
    child: Option<Box<dyn RenderObject>>,
    width: Option<f32>,
    height: Option<f32>,
}

impl SizedBoxRenderObject {
    fn new(
        child: Option<Box<dyn RenderObject>>,
        width: Option<f32>,
        height: Option<f32>,
    ) -> Self {
        Self {
            state: RenderObjectState::new(),
            child,
            width,
            height,
        }
    }
}

impl RenderObject for SizedBoxRenderObject {
    impl_render_object_common!(state);
    
    fn layout(&mut self, constraints: Constraints) -> Size {
        // Create constraints for the child
        let child_constraints = Constraints::new(
            self.width.unwrap_or(0.0),
            self.width.unwrap_or(constraints.max_width),
            self.height.unwrap_or(0.0),
            self.height.unwrap_or(constraints.max_height),
        );
        
        // Layout child if present
        let child_size = if let Some(child) = &mut self.child {
            child.layout(child_constraints);
            child.set_offset(Offset::ZERO);
            child.get_size()
        } else {
            child_constraints.constrain(Size::ZERO)
        };
        
        // Determine final size
        let width = self.width.unwrap_or(child_size.width);
        let height = self.height.unwrap_or(child_size.height);
        
        let size = constraints.constrain(Size::new(width, height));
        self.state.size = size;
        self.state.needs_layout = false;
        
        size
    }
    
    fn get_min_intrinsic_width(&self, height: f32) -> f32 {
        if let Some(width) = self.width {
            return width;
        }
        if let Some(child) = &self.child {
            return child.get_min_intrinsic_width(height);
        }
        0.0
    }
    
    fn get_max_intrinsic_width(&self, height: f32) -> f32 {
        if let Some(width) = self.width {
            return width;
        }
        if let Some(child) = &self.child {
            return child.get_max_intrinsic_width(height);
        }
        0.0
    }
    
    fn get_min_intrinsic_height(&self, width: f32) -> f32 {
        if let Some(height) = self.height {
            return height;
        }
        if let Some(child) = &self.child {
            return child.get_min_intrinsic_height(width);
        }
        0.0
    }
    
    fn get_max_intrinsic_height(&self, width: f32) -> f32 {
        if let Some(height) = self.height {
            return height;
        }
        if let Some(child) = &self.child {
            return child.get_max_intrinsic_height(width);
        }
        0.0
    }
    
    fn is_relayout_boundary(&self) -> bool {
        // SizedBox is a relayout boundary when both dimensions are fixed
        self.width.is_some() && self.height.is_some()
    }
    
    fn paint(&self, painter: &mut dyn Painter) {
        painter.save();
        painter.translate(self.state.offset);
        
        if let Some(child) = &self.child {
            child.paint(painter);
        }
        
        painter.restore();
    }
    
    fn children(&self) -> Vec<&dyn RenderObject> {
        match &self.child {
            Some(child) => vec![child.as_ref()],
            None => vec![],
        }
    }
    
    fn children_mut(&mut self) -> Vec<&mut dyn RenderObject> {
        match &mut self.child {
            Some(child) => vec![child.as_mut()],
            None => vec![],
        }
    }
    
    fn add_child(&mut self, child: Box<dyn RenderObject>) {
        self.child = Some(child);
        self.state.mark_needs_layout();
    }
    
    fn remove_child(&mut self, _index: usize) -> Option<Box<dyn RenderObject>> {
        self.state.mark_needs_layout();
        self.child.take()
    }
}
