//! Row Widget
//!
//! Arranges children horizontally.

use std::any::{Any, TypeId};

use hoshimi_types::{
    Constraints, CrossAxisAlignment, MainAxisAlignment, MainAxisSize, Offset, Rect, Size,
};

use crate::events::{EventResult, InputEvent};
use crate::key::WidgetKey;
use crate::painter::Painter;
use crate::render_object::{
    EventHandlable, Layoutable, Lifecycle, Paintable, Parent, RenderObject, RenderObjectState,
};
use crate::widget::Widget;

/// Row widget that arranges children horizontally
#[derive(Debug)]
pub struct Row {
    /// Child widgets
    pub children: Vec<Box<dyn Widget>>,
    
    /// Main axis (horizontal) alignment
    pub main_axis_alignment: MainAxisAlignment,
    
    /// Cross axis (vertical) alignment
    pub cross_axis_alignment: CrossAxisAlignment,
    
    /// Main axis size behavior
    pub main_axis_size: MainAxisSize,
    
    /// Spacing between children
    pub spacing: f32,
    
    /// Optional widget key
    pub key: Option<WidgetKey>,
}

impl Row {
    /// Create a new empty row
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
    
    /// Create a row with children
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

impl Default for Row {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Row {
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
        
        Box::new(RowRenderObject::new(
            child_ros,
            self.main_axis_alignment,
            self.cross_axis_alignment,
            self.main_axis_size,
            self.spacing,
        ))
    }
    
    fn update_render_object(&self, render_object: &mut dyn RenderObject) {
        if let Some(row_ro) = render_object.as_any_mut().downcast_mut::<RowRenderObject>() {
            row_ro.main_axis_alignment = self.main_axis_alignment;
            row_ro.cross_axis_alignment = self.cross_axis_alignment;
            row_ro.main_axis_size = self.main_axis_size;
            row_ro.spacing = self.spacing;
            row_ro.state.mark_needs_layout();
        }
    }
    
    fn should_update(&self, old: &dyn Widget) -> bool {
        if let Some(old_row) = old.as_any().downcast_ref::<Row>() {
            self.main_axis_alignment != old_row.main_axis_alignment ||
            self.cross_axis_alignment != old_row.cross_axis_alignment ||
            self.main_axis_size != old_row.main_axis_size ||
            (self.spacing - old_row.spacing).abs() > f32::EPSILON
        } else {
            true
        }
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_boxed(&self) -> Box<dyn Widget> {
        Box::new(Row {
            children: self.children.iter().map(|c| c.clone_boxed()).collect(),
            main_axis_alignment: self.main_axis_alignment,
            cross_axis_alignment: self.cross_axis_alignment,
            main_axis_size: self.main_axis_size,
            spacing: self.spacing,
            key: self.key.clone(),
        })
    }
}

/// Render object for Row widget
#[derive(Debug)]
pub struct RowRenderObject {
    state: RenderObjectState,
    children: Vec<Box<dyn RenderObject>>,
    main_axis_alignment: MainAxisAlignment,
    cross_axis_alignment: CrossAxisAlignment,
    main_axis_size: MainAxisSize,
    spacing: f32,
}

impl RowRenderObject {
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

impl Layoutable for RowRenderObject {
    fn layout(&mut self, constraints: Constraints) -> Size {
        if self.children.is_empty() {
            let size = match self.main_axis_size {
                MainAxisSize::Min => constraints.smallest(),
                MainAxisSize::Max => Size::new(constraints.max_width, constraints.min_height),
            };
            self.state.size = constraints.constrain(size);
            self.state.needs_layout = false;
            return self.state.size;
        }

        // ====================================================================
        // Phase 1: Layout non-flex children and collect flex children info
        // ====================================================================

        // Constraints for non-flex children (unbounded on main axis)
        let non_flex_constraints = Constraints::new(
            0.0,
            f32::INFINITY,
            constraints.min_height,
            constraints.max_height,
        );

        // Track child sizes, flex info, and totals
        let mut child_sizes: Vec<Option<Size>> = vec![None; self.children.len()];
        let mut flex_children: Vec<(usize, u32, bool)> = Vec::new(); // (index, flex, is_tight)
        let mut total_flex: u32 = 0;
        let mut allocated_width = 0.0;
        let mut max_height: f32 = 0.0;

        for (i, child) in self.children.iter_mut().enumerate() {
            if let Some((flex, is_tight)) = child.get_flex_data() {
                // This is a flex child, defer layout
                flex_children.push((i, flex, is_tight));
                total_flex += flex;
            } else {
                // Non-flex child: layout immediately
                let size = child.layout(non_flex_constraints);
                child_sizes[i] = Some(size);
                allocated_width += size.width;
                max_height = max_height.max(size.height);
            }
        }

        // Add spacing to allocated width
        if !self.children.is_empty() {
            allocated_width += self.spacing * (self.children.len() - 1) as f32;
        }

        // ====================================================================
        // Phase 2: Distribute remaining space to flex children
        // ====================================================================

        let available_width = constraints.max_width;
        let remaining_width = (available_width - allocated_width).max(0.0);
        let space_per_flex = if total_flex > 0 {
            remaining_width / total_flex as f32
        } else {
            0.0
        };

        for (index, flex, is_tight) in &flex_children {
            let child = &mut self.children[*index];
            let child_main_size = space_per_flex * (*flex as f32);

            // Create constraints based on fit type
            let child_constraints = if *is_tight {
                // Tight: child must fill allocated space
                Constraints::new(
                    child_main_size,
                    child_main_size,
                    constraints.min_height,
                    constraints.max_height,
                )
            } else {
                // Loose: child can be smaller than allocated space
                Constraints::new(
                    0.0,
                    child_main_size,
                    constraints.min_height,
                    constraints.max_height,
                )
            };

            let size = child.layout(child_constraints);
            child_sizes[*index] = Some(size);
            max_height = max_height.max(size.height);
        }

        // ====================================================================
        // Phase 3: Calculate final size and position children
        // ====================================================================

        // Calculate total width from all children
        let total_children_width: f32 = child_sizes.iter()
            .filter_map(|s| s.as_ref())
            .map(|s| s.width)
            .sum();
        let total_width = total_children_width + self.spacing * (self.children.len() - 1).max(0) as f32;

        // Determine final size
        let final_height = match self.cross_axis_alignment {
            CrossAxisAlignment::Stretch => constraints.max_height,
            _ => max_height,
        };

        let final_width = match self.main_axis_size {
            MainAxisSize::Min => total_width,
            MainAxisSize::Max => constraints.max_width,
        };

        let size = constraints.constrain(Size::new(final_width, final_height));

        // Calculate spacing for main axis alignment
        let extra_space = (size.width - total_width).max(0.0);
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

        // Position children
        let mut x = start_offset;
        for (i, child) in self.children.iter_mut().enumerate() {
            let child_size = child_sizes[i].unwrap_or(Size::zero());

            let y = match self.cross_axis_alignment {
                CrossAxisAlignment::Start => 0.0,
                CrossAxisAlignment::End => size.height - child_size.height,
                CrossAxisAlignment::Center => (size.height - child_size.height) / 2.0,
                CrossAxisAlignment::Stretch => 0.0,
            };

            child.set_offset(Offset::new(x, y));
            x += child_size.width + self.spacing + between_space;
        }

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
        // Row: min width is sum of children's min widths + spacing
        let mut total_width: f32 = 0.0;
        for child in &self.children {
            total_width += child.get_min_intrinsic_width(height);
        }
        if !self.children.is_empty() {
            total_width += self.spacing * (self.children.len() - 1) as f32;
        }
        total_width
    }

    fn get_max_intrinsic_width(&self, height: f32) -> f32 {
        // Row: max width is sum of children's max widths + spacing
        let mut total_width: f32 = 0.0;
        for child in &self.children {
            total_width += child.get_max_intrinsic_width(height);
        }
        if !self.children.is_empty() {
            total_width += self.spacing * (self.children.len() - 1) as f32;
        }
        total_width
    }

    fn get_min_intrinsic_height(&self, _width: f32) -> f32 {
        // Row: min height is max of children's min heights
        let mut max_height: f32 = 0.0;
        for child in &self.children {
            max_height = max_height.max(child.get_min_intrinsic_height(f32::INFINITY));
        }
        max_height
    }

    fn get_max_intrinsic_height(&self, _width: f32) -> f32 {
        // Row: max height is max of children's max heights
        let mut max_height: f32 = 0.0;
        for child in &self.children {
            max_height = max_height.max(child.get_max_intrinsic_height(f32::INFINITY));
        }
        max_height
    }
}

impl Paintable for RowRenderObject {
    fn paint(&self, painter: &mut dyn Painter) {
        painter.save();
        painter.translate(self.state.offset);

        for child in &self.children {
            child.paint(painter);
        }

        painter.restore();
    }

    fn needs_paint(&self) -> bool {
        self.state.needs_paint
    }

    fn mark_needs_paint(&mut self) {
        self.state.needs_paint = true;
    }
}

impl EventHandlable for RowRenderObject {
    fn handle_event(&mut self, event: &InputEvent) -> EventResult {
        for child in &mut self.children {
            let result = child.handle_event(event);
            if result != EventResult::Ignored {
                return result;
            }
        }
        EventResult::Ignored
    }
}

impl Lifecycle for RowRenderObject {
    fn on_mount(&mut self) {
        for child in &mut self.children {
            child.on_mount();
        }
    }

    fn on_unmount(&mut self) {
        for child in &mut self.children {
            child.on_unmount();
        }
    }
}

impl Parent for RowRenderObject {
    fn children(&self) -> Vec<&dyn RenderObject> {
        self.children.iter().map(|c| c.as_ref()).collect()
    }

    fn children_mut(&mut self) -> Vec<&mut dyn RenderObject> {
        self.children.iter_mut().map(|c| c.as_mut()).collect()
    }

    fn add_child(&mut self, child: Box<dyn RenderObject>) {
        self.children.push(child);
        self.state.needs_layout = true;
    }

    fn remove_child(&mut self, index: usize) -> Option<Box<dyn RenderObject>> {
        if index < self.children.len() {
            self.state.needs_layout = true;
            Some(self.children.remove(index))
        } else {
            None
        }
    }

    fn insert_child(&mut self, index: usize, child: Box<dyn RenderObject>) {
        self.children.insert(index.min(self.children.len()), child);
        self.state.needs_layout = true;
    }
}

impl RenderObject for RowRenderObject {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
