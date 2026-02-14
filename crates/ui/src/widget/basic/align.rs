//! Align Widget
//!
//! Aligns its child within itself.

use std::any::{Any, TypeId};

use hoshimi_types::{Alignment, Constraints, Offset, Rect, Size};

use crate::events::{EventResult, InputEvent};
use crate::key::WidgetKey;
use crate::painter::Painter;
use crate::render_object::{
    EventHandlable, Layoutable, Lifecycle, Paintable, Parent, RenderObject, RenderObjectState,
};
use crate::widget::Widget;

/// Align widget that aligns its child within itself
#[derive(Debug)]
pub struct Align {
    /// Child widget
    pub child: Box<dyn Widget>,
    
    /// Alignment
    pub alignment: Alignment,
    
    /// Width factor (0.0-1.0) - multiplies available width
    pub width_factor: Option<f32>,
    
    /// Height factor (0.0-1.0) - multiplies available height
    pub height_factor: Option<f32>,
    
    /// Optional widget key
    pub key: Option<WidgetKey>,
}

impl Align {
    /// Create a new align widget
    pub fn new(alignment: Alignment, child: impl Widget + 'static) -> Self {
        Self {
            child: Box::new(child),
            alignment,
            width_factor: None,
            height_factor: None,
            key: None,
        }
    }
    
    /// Align to top left
    pub fn top_left(child: impl Widget + 'static) -> Self {
        Self::new(Alignment::TOP_LEFT, child)
    }
    
    /// Align to top center
    pub fn top_center(child: impl Widget + 'static) -> Self {
        Self::new(Alignment::TOP_CENTER, child)
    }
    
    /// Align to top right
    pub fn top_right(child: impl Widget + 'static) -> Self {
        Self::new(Alignment::TOP_RIGHT, child)
    }
    
    /// Align to center left
    pub fn center_left(child: impl Widget + 'static) -> Self {
        Self::new(Alignment::CENTER_LEFT, child)
    }
    
    /// Align to center
    pub fn center(child: impl Widget + 'static) -> Self {
        Self::new(Alignment::CENTER, child)
    }
    
    /// Align to center right
    pub fn center_right(child: impl Widget + 'static) -> Self {
        Self::new(Alignment::CENTER_RIGHT, child)
    }
    
    /// Align to bottom left
    pub fn bottom_left(child: impl Widget + 'static) -> Self {
        Self::new(Alignment::BOTTOM_LEFT, child)
    }
    
    /// Align to bottom center
    pub fn bottom_center(child: impl Widget + 'static) -> Self {
        Self::new(Alignment::BOTTOM_CENTER, child)
    }
    
    /// Align to bottom right
    pub fn bottom_right(child: impl Widget + 'static) -> Self {
        Self::new(Alignment::BOTTOM_RIGHT, child)
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

impl Widget for Align {
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
        Box::new(AlignRenderObject::new(
            self.child.create_render_object(),
            self.alignment,
            self.width_factor,
            self.height_factor,
        ))
    }
    
    fn update_render_object(&self, render_object: &mut dyn RenderObject) {
        if let Some(align_ro) = render_object.as_any_mut().downcast_mut::<AlignRenderObject>() {
            align_ro.alignment = self.alignment;
            align_ro.width_factor = self.width_factor;
            align_ro.height_factor = self.height_factor;
            align_ro.state.mark_needs_layout();
        }
    }
    
    fn should_update(&self, old: &dyn Widget) -> bool {
        if let Some(old_align) = old.as_any().downcast_ref::<Align>() {
            self.alignment != old_align.alignment ||
            self.width_factor != old_align.width_factor ||
            self.height_factor != old_align.height_factor
        } else {
            true
        }
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_boxed(&self) -> Box<dyn Widget> {
        Box::new(Align {
            child: self.child.clone_boxed(),
            alignment: self.alignment,
            width_factor: self.width_factor,
            height_factor: self.height_factor,
            key: self.key.clone(),
        })
    }
}

/// Render object for Align widget
#[derive(Debug)]
pub struct AlignRenderObject {
    state: RenderObjectState,
    child: Box<dyn RenderObject>,
    alignment: Alignment,
    width_factor: Option<f32>,
    height_factor: Option<f32>,
}

impl AlignRenderObject {
    fn new(
        child: Box<dyn RenderObject>,
        alignment: Alignment,
        width_factor: Option<f32>,
        height_factor: Option<f32>,
    ) -> Self {
        Self {
            state: RenderObjectState::new(),
            child,
            alignment,
            width_factor,
            height_factor,
        }
    }
}

impl Layoutable for AlignRenderObject {
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

        // Align the child
        let offset = self.alignment.align(child_size, size);
        self.child.set_offset(offset);

        self.state.size = size;
        self.state.needs_layout = false;

        size
    }

    fn get_rect(&self) -> Rect {
        self.state.get_rect()
    }

    fn set_offset(&mut self, offset: Offset) {
        self.state.offset = offset;
    }

    fn get_offset(&self) -> Offset {
        self.state.offset
    }

    fn get_size(&self) -> Size {
        self.state.size
    }

    fn needs_layout(&self) -> bool {
        self.state.needs_layout
    }

    fn mark_needs_layout(&mut self) {
        self.state.needs_layout = true;
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
}

impl Paintable for AlignRenderObject {
    fn paint(&self, painter: &mut dyn Painter) {
        painter.save();
        painter.translate(self.state.offset);
        self.child.paint(painter);
        painter.restore();
    }

    fn needs_paint(&self) -> bool {
        self.state.needs_paint
    }

    fn mark_needs_paint(&mut self) {
        self.state.needs_paint = true;
    }
}

impl EventHandlable for AlignRenderObject {
    fn handle_event(&mut self, event: &InputEvent) -> EventResult {
        self.child.handle_event(event)
    }
}

impl Lifecycle for AlignRenderObject {
    fn on_mount(&mut self) {
        self.child.on_mount();
    }

    fn on_unmount(&mut self) {
        self.child.on_unmount();
    }
}

impl Parent for AlignRenderObject {
    fn children(&self) -> Vec<&dyn RenderObject> {
        vec![self.child.as_ref()]
    }

    fn children_mut(&mut self) -> Vec<&mut dyn RenderObject> {
        vec![self.child.as_mut()]
    }
}

impl RenderObject for AlignRenderObject {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
