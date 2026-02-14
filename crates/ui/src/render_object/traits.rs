//! RenderObject Trait Definition
//!
//! RenderObjects are the mutable counterparts to Widgets.
//! They handle layout, painting, and event handling.
//!
//! # Architecture
//!
//! The RenderObject system is split into multiple traits for better separation of concerns:
//! - `Layoutable` - Layout computation and geometry
//! - `Paintable` - Rendering and painting
//! - `EventHandlable` - Input event handling
//! - `RenderObject` - Composite trait combining all capabilities

use std::any::Any;
use std::fmt::Debug;

use hoshimi_types::{Constraints, Offset, Rect, Size};

use crate::events::{EventResult, HitTestResult, InputEvent};
use crate::painter::Painter;

// ============================================================================
// Layout Trait
// ============================================================================

/// Trait for objects that can perform layout computations.
///
/// Layoutable objects are responsible for:
/// - Computing their size given constraints
/// - Positioning their children
/// - Providing intrinsic size information
pub trait Layoutable: Debug {
    /// Perform layout and return the computed size.
    ///
    /// This method should:
    /// 1. Use the constraints to determine the appropriate size
    /// 2. Layout any children (passing appropriate child constraints)
    /// 3. Position children using `set_offset()`
    /// 4. Return the final size
    fn layout(&mut self, constraints: Constraints) -> Size;

    /// Get the computed rect (position + size).
    fn get_rect(&self) -> Rect;

    /// Set the position offset (called by parent during layout).
    fn set_offset(&mut self, offset: Offset);

    /// Get the current offset.
    fn get_offset(&self) -> Offset;

    /// Get the computed size.
    fn get_size(&self) -> Size;

    /// Check if layout is needed.
    fn needs_layout(&self) -> bool;

    /// Mark that layout is needed.
    fn mark_needs_layout(&mut self);

    /// Get flex data for this render object (used by Row/Column).
    fn get_flex_data(&self) -> Option<(u32, bool)> {
        None
    }

    /// Get the minimum intrinsic width given a height constraint.
    fn get_min_intrinsic_width(&self, _height: f32) -> f32 {
        0.0
    }

    /// Get the maximum intrinsic width given a height constraint.
    fn get_max_intrinsic_width(&self, _height: f32) -> f32 {
        0.0
    }

    /// Get the minimum intrinsic height given a width constraint.
    fn get_min_intrinsic_height(&self, _width: f32) -> f32 {
        0.0
    }

    /// Get the maximum intrinsic height given a width constraint.
    fn get_max_intrinsic_height(&self, _width: f32) -> f32 {
        0.0
    }

    /// Check if this render object is a relayout boundary.
    fn is_relayout_boundary(&self) -> bool {
        false
    }

    /// Perform layout only if needed, skipping if already laid out.
    fn layout_if_needed(&mut self, constraints: Constraints) -> Size {
        if self.needs_layout() {
            self.layout(constraints)
        } else {
            self.get_size()
        }
    }
}

// ============================================================================
// Paint Trait
// ============================================================================

/// Trait for objects that can paint themselves.
///
/// Paintable objects are responsible for:
/// - Rendering their visual content
/// - Managing paint dirty flags
pub trait Paintable: Debug {
    /// Paint this render object.
    ///
    /// The painter's coordinate system is relative to this object's offset.
    fn paint(&self, painter: &mut dyn Painter);

    /// Check if paint is needed.
    fn needs_paint(&self) -> bool;

    /// Mark that paint is needed.
    fn mark_needs_paint(&mut self);

    /// Check if this render object can be cached as a texture layer.
    fn supports_layer_cache(&self) -> bool {
        true
    }
}

// ============================================================================
// Event Handling Trait
// ============================================================================

/// Trait for objects that can handle input events.
///
/// EventHandlable objects are responsible for:
/// - Hit testing to determine if a point is within bounds
/// - Processing input events and producing results
///
/// This trait requires `Layoutable` because event handling needs geometry information.
pub trait EventHandlable: Layoutable {
    /// Test if a point hits this render object.
    ///
    /// Position is in local coordinates (relative to this object's offset).
    fn hit_test(&self, position: Offset) -> HitTestResult {
        let rect = self.get_rect();

        if rect.contains(position) {
            HitTestResult::HitTransparent
        } else {
            HitTestResult::Miss
        }
    }

    /// Handle an input event.
    ///
    /// Returns how the event was handled.
    fn handle_event(&mut self, _event: &InputEvent) -> EventResult {
        EventResult::Ignored
    }
}

// ============================================================================
// Lifecycle Trait
// ============================================================================

/// Trait for objects with a lifecycle in the render tree.
pub trait Lifecycle: Debug {
    /// Called when this render object is mounted to the tree.
    fn on_mount(&mut self) {}

    /// Called when this render object is unmounted from the tree.
    fn on_unmount(&mut self) {}

    /// Called when the widget configuration is updated.
    fn on_update(&mut self) {}
}

// ============================================================================
// Animation Trait
// ============================================================================

/// Trait for render objects that can be animated.
pub trait Animatable: Debug {
    /// Update animations with the given delta time (in seconds).
    fn update(&mut self, delta: f32);

    /// Check if any animation is currently running.
    fn is_animating(&self) -> bool;
}

// ============================================================================
// Children Management Trait
// ============================================================================

/// Trait for objects that can have children.
pub trait Parent: Debug {
    /// Get mutable references to child render objects.
    fn children_mut(&mut self) -> Vec<&mut dyn RenderObject> {
        Vec::new()
    }

    /// Get references to child render objects.
    fn children(&self) -> Vec<&dyn RenderObject> {
        Vec::new()
    }

    /// Add a child render object.
    fn add_child(&mut self, _child: Box<dyn RenderObject>) {}

    /// Remove a child at the given index.
    fn remove_child(&mut self, _index: usize) -> Option<Box<dyn RenderObject>> {
        None
    }

    /// Insert a child at the given index.
    fn insert_child(&mut self, _index: usize, _child: Box<dyn RenderObject>) {}
}

// ============================================================================
// Composite RenderObject Trait
// ============================================================================

/// The core RenderObject trait.
///
/// RenderObjects are mutable objects that:
/// - Store layout state (position, size)
/// - Store rendering state (animations, caches)
/// - Perform layout calculations
/// - Paint to the screen
/// - Handle input events
///
/// # Lifecycle
///
/// 1. `on_mount()` - Called when first attached to the render tree
/// 2. `layout()` - Called to compute size given constraints
/// 3. `paint()` - Called to render to the screen
/// 4. `on_unmount()` - Called when removed from the render tree
///
/// # Design Note
///
/// This trait combines multiple smaller traits for convenience.
/// For stateful widgets, the RenderObject should NOT manage interaction state.
/// Interaction state should be managed by `WidgetState` implementations.
pub trait RenderObject:
    Layoutable
    + Paintable
    + EventHandlable
    + Lifecycle
    + Parent
    + Any
{
    /// Update animations with the given delta time (in seconds).
    ///
    /// This method is called every frame to advance any running animations.
    /// Returns `true` if any animation is still in progress (needs more frames).
    fn tick(&mut self, delta: f32) -> bool {
        let mut animating = false;
        for child in self.children_mut() {
            if child.tick(delta) {
                animating = true;
            }
        }
        animating
    }

    /// Check if this render object is considered "dynamic".
    ///
    /// Dynamic objects are those that:
    /// - Have running animations
    /// - Are interactive (gesture detectors)
    /// - Have real-time content (video, live data)
    fn is_dynamic(&self) -> bool {
        for child in self.children() {
            if child.is_dynamic() {
                return true;
            }
        }
        false
    }

    /// Convert to Any for downcasting.
    fn as_any(&self) -> &dyn Any;

    /// Convert to mutable Any for downcasting.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

// ============================================================================
// RenderObjectState
// ============================================================================

/// Base state for render objects.
///
/// Most render objects can include this struct to handle common state.
/// This struct only contains rendering-related state (position, size, dirty flags),
/// NOT interaction state (pressed, hovered, etc.).
#[derive(Debug, Clone, Default)]
pub struct RenderObjectState {
    /// The position offset
    pub offset: Offset,

    /// The computed size
    pub size: Size,

    /// Whether layout needs to be recomputed
    pub needs_layout: bool,

    /// Whether painting needs to be redone
    pub needs_paint: bool,

    /// Whether this object is mounted
    pub is_mounted: bool,
}

impl RenderObjectState {
    /// Create a new RenderObjectState with default value.
    pub fn new() -> Self {
        Self {
            offset: Offset::zero(),
            size: Size::zero(),
            needs_layout: true,
            needs_paint: true,
            is_mounted: false,
        }
    }

    /// Get the RenderObject's bounds.
    pub fn get_rect(&self) -> Rect {
        Rect::from_offset_size(self.offset, self.size)
    }

    /// Mark this object as needing layout.
    pub fn mark_needs_layout(&mut self) {
        self.needs_layout = true;
    }

    /// Mark this object as needing paint.
    pub fn mark_needs_paint(&mut self) {
        self.needs_paint = true;
    }
}

// ============================================================================
// Helper Macros
// ============================================================================

/// Helper macro for implementing Layoutable trait for a struct with a state field.
/// Use this macro inside an `impl Layoutable for YourStruct` block.
#[macro_export]
macro_rules! impl_layoutable_body {
    ($state_field:ident) => {
        fn get_rect(&self) -> hoshimi_types::Rect {
            self.$state_field.get_rect()
        }

        fn set_offset(&mut self, offset: hoshimi_types::Offset) {
            self.$state_field.offset = offset;
        }

        fn get_offset(&self) -> hoshimi_types::Offset {
            self.$state_field.offset
        }

        fn get_size(&self) -> hoshimi_types::Size {
            self.$state_field.size
        }

        fn needs_layout(&self) -> bool {
            self.$state_field.needs_layout
        }

        fn mark_needs_layout(&mut self) {
            self.$state_field.needs_layout = true;
        }
    };
}

/// Helper macro for implementing Paintable trait for a struct with a state field.
/// Use this macro inside an `impl Paintable for YourStruct` block.
#[macro_export]
macro_rules! impl_paintable_body {
    ($state_field:ident) => {
        fn needs_paint(&self) -> bool {
            self.$state_field.needs_paint
        }

        fn mark_needs_paint(&mut self) {
            self.$state_field.needs_paint = true;
        }
    };
}

/// Helper macro for implementing EventHandlable trait for a struct with a single child.
/// Use this macro inside an `impl EventHandlable for YourStruct` block.
#[macro_export]
macro_rules! impl_event_handlable_single_body {
    ($child_field:ident) => {
        fn handle_event(&mut self, event: &$crate::events::InputEvent) -> $crate::events::EventResult {
            self.$child_field.handle_event(event)
        }
    };
}

/// Helper macro for implementing Lifecycle trait for a struct with a single child.
/// Use this macro inside an `impl Lifecycle for YourStruct` block.
#[macro_export]
macro_rules! impl_lifecycle_single_body {
    ($child_field:ident) => {
        fn on_mount(&mut self) {
            self.$child_field.on_mount();
        }

        fn on_unmount(&mut self) {
            self.$child_field.on_unmount();
        }
    };
}

/// Helper macro for implementing Parent trait for a struct with a single child.
/// Use this macro inside an `impl Parent for YourStruct` block.
#[macro_export]
macro_rules! impl_parent_single_body {
    ($child_field:ident) => {
        fn children(&self) -> Vec<&dyn $crate::render_object::RenderObject> {
            vec![self.$child_field.as_ref()]
        }

        fn children_mut(&mut self) -> Vec<&mut dyn $crate::render_object::RenderObject> {
            vec![self.$child_field.as_mut()]
        }
    };
}

/// Helper macro for implementing single-child layout.
/// Use this macro inside an `impl Layoutable for YourStruct` block.
#[macro_export]
macro_rules! impl_single_child_layout_body {
    ($state_field:ident, $child_field:ident) => {
        fn layout(&mut self, constraints: hoshimi_types::Constraints) -> hoshimi_types::Size {
            let child_size = self.$child_field.layout(constraints);
            self.$child_field.set_offset(hoshimi_types::Offset::ZERO);
            self.$state_field.size = child_size;
            child_size
        }

        $crate::impl_layoutable_body!($state_field);
    };
}

/// Helper macro for implementing RenderObject::tick for animated widgets.
/// Use this macro inside an `impl RenderObject for YourStruct` block.
#[macro_export]
macro_rules! impl_animated_tick_body {
    ($state_field:ident, $child_field:ident) => {
        fn tick(&mut self, delta: f32) -> bool {
            $crate::render_object::Animatable::update(self, delta);
            let self_animating = $crate::render_object::Animatable::is_animating(self);

            let child_animating = self.$child_field.tick(delta);

            if self_animating {
                self.$state_field.needs_paint = true;
            }

            self_animating || child_animating
        }
    };
}

/// Helper macro for implementing RenderObject for simple widgets.
/// Use this macro inside an `impl RenderObject for YourStruct` block.
#[macro_export]
macro_rules! impl_render_object_body {
    ($state_field:ident) => {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    };
}

// ============================================================================
// Complete Trait Implementation Macros
// ============================================================================

/// Macro to implement all traits for a single-child render object with standard layout.
/// This generates impl blocks for Layoutable, Paintable, EventHandlable, Lifecycle, Parent, and RenderObject.
#[macro_export]
macro_rules! impl_render_object_single_child {
    ($type_name:ident, $state_field:ident, $child_field:ident) => {
        impl $crate::render_object::Layoutable for $type_name {
            $crate::impl_single_child_layout_body!($state_field, $child_field);
        }

        impl $crate::render_object::Paintable for $type_name {
            $crate::impl_paintable_body!($state_field);
        }

        impl $crate::render_object::EventHandlable for $type_name {
            $crate::impl_event_handlable_single_body!($child_field);
        }

        impl $crate::render_object::Lifecycle for $type_name {
            $crate::impl_lifecycle_single_body!($child_field);
        }

        impl $crate::render_object::Parent for $type_name {
            $crate::impl_parent_single_body!($child_field);
        }

        impl $crate::render_object::RenderObject for $type_name {
            $crate::impl_render_object_body!($state_field);
        }
    };
}

/// Empty RenderObject for placeholder purposes.
#[derive(Debug)]
pub struct EmptyRenderObject;

impl Layoutable for EmptyRenderObject {
    fn layout(&mut self, _constraints: Constraints) -> Size {
        Size::zero()
    }

    fn get_rect(&self) -> Rect {
        Rect::zero()
    }

    fn set_offset(&mut self, _offset: Offset) {}

    fn get_offset(&self) -> Offset {
        Offset::zero()
    }

    fn get_size(&self) -> Size {
        Size::zero()
    }

    fn needs_layout(&self) -> bool {
        false
    }

    fn mark_needs_layout(&mut self) {}
}

impl Paintable for EmptyRenderObject {
    fn paint(&self, _painter: &mut dyn Painter) {}

    fn needs_paint(&self) -> bool {
        false
    }

    fn mark_needs_paint(&mut self) {}
}

impl EventHandlable for EmptyRenderObject {
    fn handle_event(&mut self, _event: &InputEvent) -> EventResult {
        EventResult::Ignored
    }
}

impl Lifecycle for EmptyRenderObject {}

impl Parent for EmptyRenderObject {}
