//! Container Widget
//!
//! A single-child widget that provides padding, decoration, and size constraints.

use std::any::{Any, TypeId};

use hoshimi_types::{
    Alignment, BoxDecoration, Constraints, EdgeInsets, Offset, Size,
};

use crate::key::WidgetKey;
use crate::painter::Painter;
use crate::render_object::{RenderObject, RenderObjectState};
use crate::widget::Widget;
use crate::impl_render_object_common;

/// Container widget for decorating and sizing a single child
#[derive(Debug)]
pub struct Container {
    /// Child widget
    pub child: Option<Box<dyn Widget>>,
    
    /// Padding inside the container
    pub padding: EdgeInsets,
    
    /// Margin outside the container
    pub margin: EdgeInsets,
    
    /// Explicit width
    pub width: Option<f32>,
    
    /// Explicit height
    pub height: Option<f32>,
    
    /// Box decoration (background, border, etc.)
    pub decoration: Option<BoxDecoration>,
    
    /// Child alignment within the container
    pub alignment: Option<Alignment>,
    
    /// Optional widget key
    pub key: Option<WidgetKey>,
}

impl Container {
    /// Create an empty container
    pub fn new() -> Self {
        Self {
            child: None,
            padding: EdgeInsets::zero(),
            margin: EdgeInsets::zero(),
            width: None,
            height: None,
            decoration: None,
            alignment: None,
            key: None,
        }
    }
    
    /// Create a container with a child
    pub fn with_child(child: impl Widget + 'static) -> Self {
        Self {
            child: Some(Box::new(child)),
            padding: EdgeInsets::zero(),
            margin: EdgeInsets::zero(),
            width: None,
            height: None,
            decoration: None,
            alignment: None,
            key: None,
        }
    }
    
    /// Set the child widget
    pub fn child(mut self, child: impl Widget + 'static) -> Self {
        self.child = Some(Box::new(child));
        self
    }
    
    /// Set padding
    pub fn with_padding(mut self, padding: EdgeInsets) -> Self {
        self.padding = padding;
        self
    }
    
    /// Set uniform padding
    pub fn with_padding_all(mut self, value: f32) -> Self {
        self.padding = EdgeInsets::all(value);
        self
    }
    
    /// Set margin
    pub fn with_margin(mut self, margin: EdgeInsets) -> Self {
        self.margin = margin;
        self
    }
    
    /// Set explicit width
    pub fn with_width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }
    
    /// Set explicit height
    pub fn with_height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }
    
    /// Set explicit size
    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }
    
    /// Set box decoration
    pub fn with_decoration(mut self, decoration: BoxDecoration) -> Self {
        self.decoration = Some(decoration);
        self
    }
    
    /// Set background color
    pub fn with_color(mut self, color: hoshimi_types::Color) -> Self {
        self.decoration = Some(
            self.decoration.take().unwrap_or_default().with_color(color)
        );
        self
    }
    
    /// Set child alignment
    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = Some(alignment);
        self
    }
    
    /// Set the widget key
    pub fn with_key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.key = Some(key.into());
        self
    }
}

impl Default for Container {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Container {
    fn widget_type(&self) -> TypeId {
        TypeId::of::<Self>()
    }
    
    fn key(&self) -> Option<WidgetKey> {
        self.key.clone()
    }
    
    fn children(&self) -> Vec<&dyn Widget> {
        self.child.as_ref().map(|c| vec![c.as_ref()]).unwrap_or_default()
    }
    
    fn create_render_object(&self) -> Box<dyn RenderObject> {
        let child_ro = self.child.as_ref().map(|c| c.create_render_object());
        
        Box::new(ContainerRenderObject::new(
            child_ro,
            self.padding,
            self.margin,
            self.width,
            self.height,
            self.decoration.clone(),
            self.alignment,
        ))
    }
    
    fn update_render_object(&self, render_object: &mut dyn RenderObject) {
        if let Some(container_ro) = render_object.as_any_mut().downcast_mut::<ContainerRenderObject>() {
            container_ro.padding = self.padding;
            container_ro.margin = self.margin;
            container_ro.explicit_width = self.width;
            container_ro.explicit_height = self.height;
            container_ro.decoration = self.decoration.clone();
            container_ro.alignment = self.alignment;
            container_ro.state.mark_needs_layout();
            container_ro.state.mark_needs_paint();
        }
    }
    
    fn should_update(&self, old: &dyn Widget) -> bool {
        if let Some(old_container) = old.as_any().downcast_ref::<Container>() {
            self.padding != old_container.padding ||
            self.margin != old_container.margin ||
            self.width != old_container.width ||
            self.height != old_container.height ||
            self.decoration != old_container.decoration ||
            self.alignment != old_container.alignment
        } else {
            true
        }
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_boxed(&self) -> Box<dyn Widget> {
        Box::new(Container {
            child: self.child.as_ref().map(|c| c.clone_boxed()),
            padding: self.padding,
            margin: self.margin,
            width: self.width,
            height: self.height,
            decoration: self.decoration.clone(),
            alignment: self.alignment,
            key: self.key.clone(),
        })
    }
}

/// Render object for Container widget
#[derive(Debug)]
pub struct ContainerRenderObject {
    state: RenderObjectState,
    child: Option<Box<dyn RenderObject>>,
    padding: EdgeInsets,
    margin: EdgeInsets,
    explicit_width: Option<f32>,
    explicit_height: Option<f32>,
    decoration: Option<BoxDecoration>,
    alignment: Option<Alignment>,
}

impl ContainerRenderObject {
    fn new(
        child: Option<Box<dyn RenderObject>>,
        padding: EdgeInsets,
        margin: EdgeInsets,
        explicit_width: Option<f32>,
        explicit_height: Option<f32>,
        decoration: Option<BoxDecoration>,
        alignment: Option<Alignment>,
    ) -> Self {
        Self {
            state: RenderObjectState::new(),
            child,
            padding,
            margin,
            explicit_width,
            explicit_height,
            decoration,
            alignment,
        }
    }
}

impl RenderObject for ContainerRenderObject {
    impl_render_object_common!(state);
    
    fn layout(&mut self, constraints: Constraints) -> Size {
        let margin_size = self.margin.total_size();
        let padding_size = self.padding.total_size();
        
        // Calculate inner constraints (accounting for margin and padding)
        let inner_constraints = constraints.deflate(self.margin).deflate(self.padding);
        
        // Layout child if present
        let child_size = if let Some(ref mut child) = self.child {
            child.layout(inner_constraints)
        } else {
            Size::zero()
        };
        
        // Calculate content size
        let content_width = self.explicit_width.unwrap_or(
            child_size.width + padding_size.width
        );
        let content_height = self.explicit_height.unwrap_or(
            child_size.height + padding_size.height
        );
        
        // Total size including margin
        let total_size = constraints.constrain(Size::new(
            content_width + margin_size.width,
            content_height + margin_size.height,
        ));
        
        // Position child within container
        if let Some(ref mut child) = self.child {
            let available_for_child = Size::new(
                total_size.width - margin_size.width - padding_size.width,
                total_size.height - margin_size.height - padding_size.height,
            );
            
            let child_offset = if let Some(alignment) = self.alignment {
                let align_offset = alignment.align_offset(available_for_child, child_size);
                Offset::new(
                    self.margin.left + self.padding.left + align_offset.x,
                    self.margin.top + self.padding.top + align_offset.y,
                )
            } else {
                Offset::new(
                    self.margin.left + self.padding.left,
                    self.margin.top + self.padding.top,
                )
            };
            
            child.set_offset(child_offset);
        }
        
        self.state.size = total_size;
        self.state.needs_layout = false;
        
        total_size
    }
    
    fn get_min_intrinsic_width(&self, height: f32) -> f32 {
        if let Some(width) = self.explicit_width {
            return width + self.margin.left + self.margin.right;
        }
        let horizontal_insets = self.margin.left + self.margin.right 
            + self.padding.left + self.padding.right;
        let child_height = (height - self.margin.top - self.margin.bottom 
            - self.padding.top - self.padding.bottom).max(0.0);
        let child_width = self.child.as_ref()
            .map(|c| c.get_min_intrinsic_width(child_height))
            .unwrap_or(0.0);
        child_width + horizontal_insets
    }
    
    fn get_max_intrinsic_width(&self, height: f32) -> f32 {
        if let Some(width) = self.explicit_width {
            return width + self.margin.left + self.margin.right;
        }
        let horizontal_insets = self.margin.left + self.margin.right 
            + self.padding.left + self.padding.right;
        let child_height = (height - self.margin.top - self.margin.bottom 
            - self.padding.top - self.padding.bottom).max(0.0);
        let child_width = self.child.as_ref()
            .map(|c| c.get_max_intrinsic_width(child_height))
            .unwrap_or(0.0);
        child_width + horizontal_insets
    }
    
    fn get_min_intrinsic_height(&self, width: f32) -> f32 {
        if let Some(height) = self.explicit_height {
            return height + self.margin.top + self.margin.bottom;
        }
        let vertical_insets = self.margin.top + self.margin.bottom 
            + self.padding.top + self.padding.bottom;
        let child_width = (width - self.margin.left - self.margin.right 
            - self.padding.left - self.padding.right).max(0.0);
        let child_height = self.child.as_ref()
            .map(|c| c.get_min_intrinsic_height(child_width))
            .unwrap_or(0.0);
        child_height + vertical_insets
    }
    
    fn get_max_intrinsic_height(&self, width: f32) -> f32 {
        if let Some(height) = self.explicit_height {
            return height + self.margin.top + self.margin.bottom;
        }
        let vertical_insets = self.margin.top + self.margin.bottom 
            + self.padding.top + self.padding.bottom;
        let child_width = (width - self.margin.left - self.margin.right 
            - self.padding.left - self.padding.right).max(0.0);
        let child_height = self.child.as_ref()
            .map(|c| c.get_max_intrinsic_height(child_width))
            .unwrap_or(0.0);
        child_height + vertical_insets
    }
    
    fn paint(&self, painter: &mut dyn Painter) {
        let rect = self.state.get_rect();
        
        // Inset by margin to get the container bounds
        let container_rect = rect.inset(self.margin);
        
        // Draw decoration
        if let Some(ref decoration) = self.decoration {
            painter.draw_box_decoration(container_rect, decoration);
        }
        
        // Paint child
        if let Some(ref child) = self.child {
            painter.save();
            painter.translate(self.state.offset);
            child.paint(painter);
            painter.restore();
        }
    }
    
    fn children(&self) -> Vec<&dyn RenderObject> {
        self.child.as_ref().map(|c| vec![c.as_ref()]).unwrap_or_default()
    }
    
    fn children_mut(&mut self) -> Vec<&mut dyn RenderObject> {
        self.child.as_mut().map(|c| vec![c.as_mut()]).unwrap_or_default()
    }
    
    fn add_child(&mut self, child: Box<dyn RenderObject>) {
        self.child = Some(child);
    }
    
    fn remove_child(&mut self, _index: usize) -> Option<Box<dyn RenderObject>> {
        self.child.take()
    }
}
