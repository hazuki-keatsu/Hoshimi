//! Column Widget
//!
//! Arranges children vertically.

use std::any::{Any, TypeId};

use hoshimi_shared::{
    Constraints, CrossAxisAlignment, MainAxisAlignment, MainAxisSize, Offset, Size,
};

use crate::key::WidgetKey;
use crate::painter::Painter;
use crate::render::{RenderObject, RenderObjectState};
use crate::widget::Widget;
use crate::impl_render_object_common;

/// Column widget that arranges children vertically
#[derive(Debug)]
pub struct Column {
    /// Child widgets
    pub children: Vec<Box<dyn Widget>>,
    
    /// Main axis (vertical) alignment
    pub main_axis_alignment: MainAxisAlignment,
    
    /// Cross axis (horizontal) alignment
    pub cross_axis_alignment: CrossAxisAlignment,
    
    /// Main axis size behavior
    pub main_axis_size: MainAxisSize,
    
    /// Spacing between children
    pub spacing: f32,
    
    /// Optional widget key
    pub key: Option<WidgetKey>,
}

impl Column {
    /// Create a new empty column
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Start,
            main_axis_size: MainAxisSize::Max,
            spacing: 0.0,
            key: None,
        }
    }
    
    /// Create a column with children
    pub fn with_children(children: Vec<Box<dyn Widget>>) -> Self {
        Self {
            children,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Start,
            main_axis_size: MainAxisSize::Max,
            spacing: 0.0,
            key: None,
        }
    }
    
    /// Add a child widget
    pub fn child(mut self, child: impl Widget + 'static) -> Self {
        self.children.push(Box::new(child));
        self
    }
    
    /// Set main axis alignment
    pub fn with_main_axis_alignment(mut self, alignment: MainAxisAlignment) -> Self {
        self.main_axis_alignment = alignment;
        self
    }
    
    /// Set cross axis alignment
    pub fn with_cross_axis_alignment(mut self, alignment: CrossAxisAlignment) -> Self {
        self.cross_axis_alignment = alignment;
        self
    }
    
    /// Set main axis size
    pub fn with_main_axis_size(mut self, size: MainAxisSize) -> Self {
        self.main_axis_size = size;
        self
    }
    
    /// Set spacing between children
    pub fn with_spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }
    
    /// Set the widget key
    pub fn with_key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.key = Some(key.into());
        self
    }
}

impl Default for Column {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Column {
    fn widget_type(&self) -> TypeId {
        TypeId::of::<Self>()
    }
    
    fn key(&self) -> Option<WidgetKey> {
        self.key.clone()
    }
    
    fn children(&self) -> Vec<&dyn Widget> {
        self.children.iter().map(|c| c.as_ref()).collect()
    }
    
    fn create_render_object(&self) -> Box<dyn RenderObject> {
        let child_ros: Vec<Box<dyn RenderObject>> = self.children
            .iter()
            .map(|c| c.create_render_object())
            .collect();
        
        Box::new(ColumnRenderObject::new(
            child_ros,
            self.main_axis_alignment,
            self.cross_axis_alignment,
            self.main_axis_size,
            self.spacing,
        ))
    }
    
    fn update_render_object(&self, render_object: &mut dyn RenderObject) {
        if let Some(column_ro) = render_object.as_any_mut().downcast_mut::<ColumnRenderObject>() {
            column_ro.main_axis_alignment = self.main_axis_alignment;
            column_ro.cross_axis_alignment = self.cross_axis_alignment;
            column_ro.main_axis_size = self.main_axis_size;
            column_ro.spacing = self.spacing;
            column_ro.state.mark_needs_layout();
        }
    }
    
    fn should_update(&self, old: &dyn Widget) -> bool {
        if let Some(old_column) = old.as_any().downcast_ref::<Column>() {
            self.main_axis_alignment != old_column.main_axis_alignment ||
            self.cross_axis_alignment != old_column.cross_axis_alignment ||
            self.main_axis_size != old_column.main_axis_size ||
            (self.spacing - old_column.spacing).abs() > f32::EPSILON
        } else {
            true
        }
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Render object for Column widget
#[derive(Debug)]
pub struct ColumnRenderObject {
    state: RenderObjectState,
    children: Vec<Box<dyn RenderObject>>,
    main_axis_alignment: MainAxisAlignment,
    cross_axis_alignment: CrossAxisAlignment,
    main_axis_size: MainAxisSize,
    spacing: f32,
}

impl ColumnRenderObject {
    fn new(
        children: Vec<Box<dyn RenderObject>>,
        main_axis_alignment: MainAxisAlignment,
        cross_axis_alignment: CrossAxisAlignment,
        main_axis_size: MainAxisSize,
        spacing: f32,
    ) -> Self {
        Self {
            state: RenderObjectState::new(),
            children,
            main_axis_alignment,
            cross_axis_alignment,
            main_axis_size,
            spacing,
        }
    }
}

impl RenderObject for ColumnRenderObject {
    impl_render_object_common!(state);
    
    fn layout(&mut self, constraints: Constraints) -> Size {
        if self.children.is_empty() {
            let size = match self.main_axis_size {
                MainAxisSize::Min => constraints.smallest(),
                MainAxisSize::Max => Size::new(constraints.min_width, constraints.max_height),
            };
            self.state.size = constraints.constrain(size);
            self.state.needs_layout = false;
            return self.state.size;
        }
        
        // First pass: layout children with unbounded height
        let child_constraints = Constraints::new(
            constraints.min_width,
            constraints.max_width,
            0.0,
            f32::INFINITY,
        );
        
        let mut child_sizes: Vec<Size> = Vec::with_capacity(self.children.len());
        let mut total_height = 0.0;
        let mut max_width: f32 = 0.0;
        
        for child in &mut self.children {
            let size = child.layout(child_constraints);
            child_sizes.push(size);
            total_height += size.height;
            max_width = max_width.max(size.width);
        }
        
        // Add spacing
        if !self.children.is_empty() {
            total_height += self.spacing * (self.children.len() - 1) as f32;
        }
        
        // Determine final size
        let final_width = match self.cross_axis_alignment {
            CrossAxisAlignment::Stretch => constraints.max_width,
            _ => max_width,
        };
        
        let final_height = match self.main_axis_size {
            MainAxisSize::Min => total_height,
            MainAxisSize::Max => constraints.max_height,
        };
        
        let size = constraints.constrain(Size::new(final_width, final_height));
        
        // Second pass: position children
        let extra_space = (size.height - total_height).max(0.0);
        let (start_offset, between_space) = match self.main_axis_alignment {
            MainAxisAlignment::Start => (0.0, 0.0),
            MainAxisAlignment::End => (extra_space, 0.0),
            MainAxisAlignment::Center => (extra_space / 2.0, 0.0),
            MainAxisAlignment::SpaceBetween => {
                if self.children.len() > 1 {
                    (0.0, extra_space / (self.children.len() - 1) as f32)
                } else {
                    (0.0, 0.0)
                }
            }
            MainAxisAlignment::SpaceAround => {
                let space = extra_space / self.children.len() as f32;
                (space / 2.0, space)
            }
            MainAxisAlignment::SpaceEvenly => {
                let space = extra_space / (self.children.len() + 1) as f32;
                (space, space)
            }
        };
        
        let mut y = start_offset;
        for (i, child) in self.children.iter_mut().enumerate() {
            let child_size = child_sizes[i];
            
            let x = match self.cross_axis_alignment {
                CrossAxisAlignment::Start => 0.0,
                CrossAxisAlignment::End => size.width - child_size.width,
                CrossAxisAlignment::Center => (size.width - child_size.width) / 2.0,
                CrossAxisAlignment::Stretch => 0.0,
            };
            
            child.set_offset(Offset::new(x, y));
            y += child_size.height + self.spacing + between_space;
        }
        
        self.state.size = size;
        self.state.needs_layout = false;
        
        size
    }
    
    fn paint(&self, painter: &mut dyn Painter) {
        painter.save();
        painter.translate(self.state.offset);
        
        for child in &self.children {
            child.paint(painter);
        }
        
        painter.restore();
    }
    
    fn children(&self) -> Vec<&dyn RenderObject> {
        self.children.iter().map(|c| c.as_ref()).collect()
    }
    
    fn children_mut(&mut self) -> Vec<&mut dyn RenderObject> {
        self.children.iter_mut().map(|c| c.as_mut()).collect()
    }
    
    fn add_child(&mut self, child: Box<dyn RenderObject>) {
        self.children.push(child);
        self.state.mark_needs_layout();
    }
    
    fn remove_child(&mut self, index: usize) -> Option<Box<dyn RenderObject>> {
        if index < self.children.len() {
            self.state.mark_needs_layout();
            Some(self.children.remove(index))
        } else {
            None
        }
    }
    
    fn insert_child(&mut self, index: usize, child: Box<dyn RenderObject>) {
        self.children.insert(index.min(self.children.len()), child);
        self.state.mark_needs_layout();
    }
}
