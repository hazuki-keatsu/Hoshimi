//! Center Widget
//!
//! Centers its child within itself.

use std::any::{Any, TypeId};

use hoshimi_types::{Constraints, Offset, Size};

use crate::key::WidgetKey;
use crate::painter::Painter;
use crate::render::{RenderObject, RenderObjectState};
use crate::widget::Widget;
use crate::impl_render_object_common;

/// Center widget that centers its child
#[derive(Debug)]
pub struct Center {
    /// Child widget
    pub child: Box<dyn Widget>,
    
    /// Width factor (0.0-1.0) - multiplies available width
    pub width_factor: Option<f32>,
    
    /// Height factor (0.0-1.0) - multiplies available height
    pub height_factor: Option<f32>,
    
    /// Optional widget key
    pub key: Option<WidgetKey>,
}

impl Center {
    /// Create a new center widget
    pub fn new(child: impl Widget + 'static) -> Self {
        Self {
            child: Box::new(child),
            width_factor: None,
            height_factor: None,
            key: None,
        }
    }
    
    /// Set width factor
    pub fn with_width_factor(mut self, factor: f32) -> Self {
        self.width_factor = Some(factor);
        self
    }
    
    /// Set height factor
    pub fn with_height_factor(mut self, factor: f32) -> Self {
        self.height_factor = Some(factor);
        self
    }
    
    /// Set the widget key
    pub fn with_key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.key = Some(key.into());
        self
    }
}

impl Widget for Center {
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
        Box::new(CenterRenderObject::new(
            self.child.create_render_object(),
            self.width_factor,
            self.height_factor,
        ))
    }
    
    fn update_render_object(&self, render_object: &mut dyn RenderObject) {
        if let Some(center_ro) = render_object.as_any_mut().downcast_mut::<CenterRenderObject>() {
            center_ro.width_factor = self.width_factor;
            center_ro.height_factor = self.height_factor;
            center_ro.state.mark_needs_layout();
        }
    }
    
    fn should_update(&self, old: &dyn Widget) -> bool {
        if let Some(old_center) = old.as_any().downcast_ref::<Center>() {
            self.width_factor != old_center.width_factor ||
            self.height_factor != old_center.height_factor
        } else {
            true
        }
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_boxed(&self) -> Box<dyn Widget> {
        Box::new(Center {
            child: self.child.clone_boxed(),
            width_factor: self.width_factor,
            height_factor: self.height_factor,
            key: self.key.clone(),
        })
    }
}

/// Render object for Center widget
#[derive(Debug)]
pub struct CenterRenderObject {
    state: RenderObjectState,
    child: Box<dyn RenderObject>,
    width_factor: Option<f32>,
    height_factor: Option<f32>,
}

impl CenterRenderObject {
    fn new(
        child: Box<dyn RenderObject>,
        width_factor: Option<f32>,
        height_factor: Option<f32>,
    ) -> Self {
        Self {
            state: RenderObjectState::new(),
            child,
            width_factor,
            height_factor,
        }
    }
}

impl RenderObject for CenterRenderObject {
    impl_render_object_common!(state);
    
    fn layout(&mut self, constraints: Constraints) -> Size {
        // Let child be as small as it wants
        let child_constraints = constraints.loosen();
        let child_size = self.child.layout(child_constraints);
        
        // Determine our size
        let width = match self.width_factor {
            Some(factor) => child_size.width * factor,
            None => constraints.max_width,
        };
        
        let height = match self.height_factor {
            Some(factor) => child_size.height * factor,
            None => constraints.max_height,
        };
        
        let size = constraints.constrain(Size::new(width, height));
        
        // Center the child
        let offset = Offset::new(
            (size.width - child_size.width) / 2.0,
            (size.height - child_size.height) / 2.0,
        );
        self.child.set_offset(offset);
        
        self.state.size = size;
        self.state.needs_layout = false;
        
        size
    }
    
    fn get_min_intrinsic_width(&self, height: f32) -> f32 {
        self.child.get_min_intrinsic_width(height)
    }
    
    fn get_max_intrinsic_width(&self, height: f32) -> f32 {
        self.child.get_max_intrinsic_width(height)
    }
    
    fn get_min_intrinsic_height(&self, width: f32) -> f32 {
        self.child.get_min_intrinsic_height(width)
    }
    
    fn get_max_intrinsic_height(&self, width: f32) -> f32 {
        self.child.get_max_intrinsic_height(width)
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
