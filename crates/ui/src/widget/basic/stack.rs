//! Stack Widget
//!
//! Positions children on top of each other.

use std::any::{Any, TypeId};

use hoshimi_shared::{Alignment, Constraints, Offset, Rect, Size};

use crate::events::{EventResult, HitTestResult, InputEvent};
use crate::key::WidgetKey;
use crate::painter::Painter;
use crate::render::{RenderObject, RenderObjectState};
use crate::widget::Widget;
use crate::impl_render_object_common;

/// Stack fit behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StackFit {
    /// Let children be as small as they want
    #[default]
    Loose,
    /// Force children to fill the stack
    Expand,
    /// Pass through parent constraints
    Passthrough,
}

/// Stack widget that positions children on top of each other
#[derive(Debug)]
pub struct Stack {
    /// Child widgets
    pub children: Vec<Box<dyn Widget>>,
    
    /// Alignment for non-positioned children
    pub alignment: Alignment,
    
    /// How to size non-positioned children
    pub fit: StackFit,
    
    /// Clip children that overflow
    pub clip_behavior: bool,
    
    /// Optional widget key
    pub key: Option<WidgetKey>,
}

impl Stack {
    /// Create a new empty stack
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            alignment: Alignment::TOP_LEFT,
            fit: StackFit::Loose,
            clip_behavior: true,
            key: None,
        }
    }
    
    /// Create a stack with children
    pub fn with_children(children: Vec<Box<dyn Widget>>) -> Self {
        Self {
            children,
            alignment: Alignment::TOP_LEFT,
            fit: StackFit::Loose,
            clip_behavior: true,
            key: None,
        }
    }
    
    /// Add a child widget
    pub fn child(mut self, child: impl Widget + 'static) -> Self {
        self.children.push(Box::new(child));
        self
    }
    
    /// Set alignment
    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }
    
    /// Set fit behavior
    pub fn with_fit(mut self, fit: StackFit) -> Self {
        self.fit = fit;
        self
    }
    
    /// Set clip behavior
    pub fn with_clip(mut self, clip: bool) -> Self {
        self.clip_behavior = clip;
        self
    }
    
    /// Set the widget key
    pub fn with_key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.key = Some(key.into());
        self
    }
}

impl Default for Stack {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Stack {
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
        
        Box::new(StackRenderObject::new(
            child_ros,
            self.alignment,
            self.fit,
            self.clip_behavior,
        ))
    }
    
    fn update_render_object(&self, render_object: &mut dyn RenderObject) {
        if let Some(stack_ro) = render_object.as_any_mut().downcast_mut::<StackRenderObject>() {
            stack_ro.alignment = self.alignment;
            stack_ro.fit = self.fit;
            stack_ro.clip_behavior = self.clip_behavior;
            stack_ro.state.mark_needs_layout();
        }
    }
    
    fn should_update(&self, old: &dyn Widget) -> bool {
        if let Some(old_stack) = old.as_any().downcast_ref::<Stack>() {
            self.alignment != old_stack.alignment ||
            self.fit != old_stack.fit ||
            self.clip_behavior != old_stack.clip_behavior
        } else {
            true
        }
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Render object for Stack widget
#[derive(Debug)]
pub struct StackRenderObject {
    state: RenderObjectState,
    children: Vec<Box<dyn RenderObject>>,
    alignment: Alignment,
    fit: StackFit,
    clip_behavior: bool,
}

impl StackRenderObject {
    fn new(
        children: Vec<Box<dyn RenderObject>>,
        alignment: Alignment,
        fit: StackFit,
        clip_behavior: bool,
    ) -> Self {
        Self {
            state: RenderObjectState::new(),
            children,
            alignment,
            fit,
            clip_behavior,
        }
    }
}

impl RenderObject for StackRenderObject {
    impl_render_object_common!(state);
    
    fn layout(&mut self, constraints: Constraints) -> Size {
        let non_positioned_constraints = match self.fit {
            StackFit::Loose => constraints.loosen(),
            StackFit::Expand => Constraints::tight(Size::new(
                constraints.max_width,
                constraints.max_height,
            )),
            StackFit::Passthrough => constraints,
        };
        
        // Layout all children and find the maximum size
        let mut max_width = constraints.min_width;
        let mut max_height = constraints.min_height;
        
        let mut child_sizes: Vec<Size> = Vec::with_capacity(self.children.len());
        
        for child in &mut self.children {
            let size = child.layout(non_positioned_constraints);
            child_sizes.push(size);
            max_width = max_width.max(size.width);
            max_height = max_height.max(size.height);
        }
        
        let size = constraints.constrain(Size::new(max_width, max_height));
        
        // Position children according to alignment
        for (i, child) in self.children.iter_mut().enumerate() {
            let child_size = child_sizes[i];
            let offset = self.alignment.align(child_size, size);
            child.set_offset(offset);
        }
        
        self.state.size = size;
        self.state.needs_layout = false;
        
        size
    }
    
    fn get_min_intrinsic_width(&self, height: f32) -> f32 {
        // Stack: max of children's min widths
        let mut max_width: f32 = 0.0;
        for child in &self.children {
            max_width = max_width.max(child.get_min_intrinsic_width(height));
        }
        max_width
    }
    
    fn get_max_intrinsic_width(&self, height: f32) -> f32 {
        // Stack: max of children's max widths
        let mut max_width: f32 = 0.0;
        for child in &self.children {
            max_width = max_width.max(child.get_max_intrinsic_width(height));
        }
        max_width
    }
    
    fn get_min_intrinsic_height(&self, width: f32) -> f32 {
        // Stack: max of children's min heights
        let mut max_height: f32 = 0.0;
        for child in &self.children {
            max_height = max_height.max(child.get_min_intrinsic_height(width));
        }
        max_height
    }
    
    fn get_max_intrinsic_height(&self, width: f32) -> f32 {
        // Stack: max of children's max heights
        let mut max_height: f32 = 0.0;
        for child in &self.children {
            max_height = max_height.max(child.get_max_intrinsic_height(width));
        }
        max_height
    }
    
    fn paint(&self, painter: &mut dyn Painter) {
        painter.save();
        painter.translate(self.state.offset);
        
        if self.clip_behavior {
            painter.clip_rect(Rect::from_size(self.state.size));
        }
        
        // Paint children in order (first child at bottom)
        for child in &self.children {
            child.paint(painter);
        }
        
        painter.restore();
    }
    
    fn hit_test(&self, position: Offset) -> HitTestResult {
        // Check if position is within bounds
        if position.x < 0.0 || position.y < 0.0 ||
           position.x > self.state.size.width || position.y > self.state.size.height {
            return HitTestResult::Miss;
        }
        
        // Hit test children in reverse order (topmost first)
        for child in self.children.iter().rev() {
            let child_offset = child.get_offset();
            let local_pos = Offset::new(
                position.x - child_offset.x,
                position.y - child_offset.y,
            );
            let result = child.hit_test(local_pos);
            if result.is_hit() {
                return result;
            }
        }
        
        HitTestResult::HitTransparent
    }
    
    fn handle_event(&mut self, event: &InputEvent) -> EventResult {
        // Handle events in reverse order (topmost first)
        for child in self.children.iter_mut().rev() {
            let result = child.handle_event(event);
            if result == EventResult::Consumed {
                return EventResult::Consumed;
            }
        }
        
        EventResult::Ignored
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
