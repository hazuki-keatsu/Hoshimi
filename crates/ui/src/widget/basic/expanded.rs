//! Expanded Widget
//!
//! A widget that expands to fill available space in a flex container.

use std::any::{Any, TypeId};

use hoshimi_types::{Constraints, Offset, Size};

use crate::key::WidgetKey;
use crate::painter::Painter;
use crate::render_object::{RenderObject, RenderObjectState};
use crate::widget::Widget;
use crate::impl_render_object_common;

/// Expanded widget that expands to fill remaining space
#[derive(Debug)]
pub struct Expanded {
    /// Child widget
    pub child: Box<dyn Widget>,
    
    /// Flex factor (weight for space distribution)
    pub flex: u32,
    
    /// Optional widget key
    pub key: Option<WidgetKey>,
}

impl Expanded {
    /// Create a new expanded widget with flex factor of 1
    pub fn new(child: impl Widget + 'static) -> Self {
        Self {
            child: Box::new(child),
            flex: 1,
            key: None,
        }
    }
    
    /// Set flex factor
    pub fn with_flex(mut self, flex: u32) -> Self {
        self.flex = flex;
        self
    }
    
    /// Set the widget key
    pub fn with_key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.key = Some(key.into());
        self
    }
}

impl Widget for Expanded {
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
        Box::new(ExpandedRenderObject::new(
            self.child.create_render_object(),
            self.flex,
        ))
    }
    
    fn update_render_object(&self, render_object: &mut dyn RenderObject) {
        if let Some(expanded_ro) = render_object.as_any_mut().downcast_mut::<ExpandedRenderObject>() {
            expanded_ro.flex = self.flex;
            expanded_ro.state.mark_needs_layout();
        }
    }
    
    fn should_update(&self, old: &dyn Widget) -> bool {
        if let Some(old_expanded) = old.as_any().downcast_ref::<Expanded>() {
            self.flex != old_expanded.flex
        } else {
            true
        }
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_boxed(&self) -> Box<dyn Widget> {
        Box::new(Expanded {
            child: self.child.clone_boxed(),
            flex: self.flex,
            key: self.key.clone(),
        })
    }
}

/// Render object for Expanded widget
#[derive(Debug)]
pub struct ExpandedRenderObject {
    state: RenderObjectState,
    child: Box<dyn RenderObject>,
    flex: u32,
}

impl ExpandedRenderObject {
    fn new(child: Box<dyn RenderObject>, flex: u32) -> Self {
        Self {
            state: RenderObjectState::new(),
            child,
            flex,
        }
    }
    
    /// Get the flex factor
    pub fn flex(&self) -> u32 {
        self.flex
    }
}

impl RenderObject for ExpandedRenderObject {
    impl_render_object_common!(state);
    
    fn get_flex_data(&self) -> Option<(u32, bool)> {
        // Expanded always uses tight fit (must fill allocated space)
        Some((self.flex, true))
    }
    
    fn layout(&mut self, constraints: Constraints) -> Size {
        // Expanded fills all available space
        let size = Size::new(constraints.max_width, constraints.max_height);
        
        // Child fills the expanded widget
        let child_constraints = Constraints::tight(size);
        self.child.layout(child_constraints);
        self.child.set_offset(Offset::ZERO);
        
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

/// Flexible widget with configurable fit behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FlexFit {
    /// Child can be smaller than allocated space
    #[default]
    Loose,
    /// Child must fill allocated space
    Tight,
}

/// Flexible widget that takes a portion of remaining space
#[derive(Debug)]
pub struct Flexible {
    /// Child widget
    pub child: Box<dyn Widget>,
    
    /// Flex factor
    pub flex: u32,
    
    /// Fit behavior
    pub fit: FlexFit,
    
    /// Optional widget key
    pub key: Option<WidgetKey>,
}

impl Flexible {
    /// Create a new flexible widget
    pub fn new(child: impl Widget + 'static) -> Self {
        Self {
            child: Box::new(child),
            flex: 1,
            fit: FlexFit::Loose,
            key: None,
        }
    }
    
    /// Set flex factor
    pub fn with_flex(mut self, flex: u32) -> Self {
        self.flex = flex;
        self
    }
    
    /// Set fit behavior
    pub fn with_fit(mut self, fit: FlexFit) -> Self {
        self.fit = fit;
        self
    }
    
    /// Set the widget key
    pub fn with_key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.key = Some(key.into());
        self
    }
}

impl Widget for Flexible {
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
        Box::new(FlexibleRenderObject::new(
            self.child.create_render_object(),
            self.flex,
            self.fit,
        ))
    }
    
    fn update_render_object(&self, render_object: &mut dyn RenderObject) {
        if let Some(flexible_ro) = render_object.as_any_mut().downcast_mut::<FlexibleRenderObject>() {
            flexible_ro.flex = self.flex;
            flexible_ro.fit = self.fit;
            flexible_ro.state.mark_needs_layout();
        }
    }
    
    fn should_update(&self, old: &dyn Widget) -> bool {
        if let Some(old_flexible) = old.as_any().downcast_ref::<Flexible>() {
            self.flex != old_flexible.flex || self.fit != old_flexible.fit
        } else {
            true
        }
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_boxed(&self) -> Box<dyn Widget> {
        Box::new(Flexible {
            child: self.child.clone_boxed(),
            flex: self.flex,
            fit: self.fit,
            key: self.key.clone(),
        })
    }
}

/// Render object for Flexible widget
#[derive(Debug)]
pub struct FlexibleRenderObject {
    state: RenderObjectState,
    child: Box<dyn RenderObject>,
    flex: u32,
    fit: FlexFit,
}

impl FlexibleRenderObject {
    fn new(child: Box<dyn RenderObject>, flex: u32, fit: FlexFit) -> Self {
        Self {
            state: RenderObjectState::new(),
            child,
            flex,
            fit,
        }
    }
    
    /// Get the flex factor
    pub fn flex(&self) -> u32 {
        self.flex
    }
    
    /// Get the fit behavior
    pub fn fit(&self) -> FlexFit {
        self.fit
    }
}

impl RenderObject for FlexibleRenderObject {
    impl_render_object_common!(state);
    
    fn get_flex_data(&self) -> Option<(u32, bool)> {
        // Return flex factor and whether it's tight fit
        Some((self.flex, self.fit == FlexFit::Tight))
    }
    
    fn layout(&mut self, constraints: Constraints) -> Size {
        let child_constraints = match self.fit {
            FlexFit::Tight => Constraints::tight(Size::new(
                constraints.max_width,
                constraints.max_height,
            )),
            FlexFit::Loose => constraints.loosen(),
        };
        
        let child_size = self.child.layout(child_constraints);
        self.child.set_offset(Offset::ZERO);
        
        let size = match self.fit {
            FlexFit::Tight => Size::new(constraints.max_width, constraints.max_height),
            FlexFit::Loose => child_size,
        };
        
        self.state.size = constraints.constrain(size);
        self.state.needs_layout = false;
        
        self.state.size
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
