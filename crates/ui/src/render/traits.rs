//! RenderObject Trait Definition
//!
//! RenderObjects are the mutable counterparts to Widgets.
//! They handle layout, painting, and event handling.

use std::any::Any;
use std::fmt::Debug;

use hoshimi_shared::{Constraints, Offset, Rect, Size};

use crate::events::{EventResult, HitTestResult, InputEvent};
use crate::painter::Painter;

/// The core RenderObject trait
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
pub trait RenderObject: Debug + Any {
    // ========================================================================
    // Layout
    // ========================================================================
    
    /// Perform layout and return the computed size
    /// 
    /// This method should:
    /// 1. Use the constraints to determine the appropriate size
    /// 2. Layout any children (passing appropriate child constraints)
    /// 3. Position children using `set_offset()`
    /// 4. Return the final size
    fn layout(&mut self, constraints: Constraints) -> Size;
    
    /// Get the computed rect (position + size)
    fn get_rect(&self) -> Rect;
    
    /// Set the position offset (called by parent during layout)
    fn set_offset(&mut self, offset: Offset);
    
    /// Get the current offset
    fn get_offset(&self) -> Offset;
    
    /// Get the computed size
    fn get_size(&self) -> Size;
    
    // ========================================================================
    // Painting
    // ========================================================================
    
    /// Paint this render object
    /// 
    /// The painter's coordinate system is relative to this object's offset.
    /// Child painting should be done through `paint_child()`.
    fn paint(&self, painter: &mut dyn Painter);
    
    // ========================================================================
    // Hit Testing
    // ========================================================================
    
    /// Test if a point hits this render object
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
    
    // ========================================================================
    // Event Handling
    // ========================================================================
    
    /// Handle an input event
    /// 
    /// Returns how the event was handled.
    fn handle_event(&mut self, _event: &InputEvent) -> EventResult {
        EventResult::Ignored
    }
    
    // ========================================================================
    // Lifecycle
    // ========================================================================
    
    /// Called when this render object is mounted to the tree
    /// 
    /// Use this for:
    /// - Resource loading
    /// - Starting animations
    /// - Setting up event listeners
    fn on_mount(&mut self) {}
    
    /// Called when this render object is unmounted from the tree
    /// 
    /// Use this for:
    /// - Resource cleanup
    /// - Stopping animations
    /// - Removing event listeners
    fn on_unmount(&mut self) {}
    
    /// Called when the widget configuration is updated
    /// 
    /// The Widget's `update_render_object` method has already been called
    /// to apply the new configuration. This hook allows for additional
    /// processing after the update.
    fn on_update(&mut self) {}
    
    // ========================================================================
    // Animation
    // ========================================================================
    
    /// Update animations with the given delta time (in seconds)
    /// 
    /// This method is called every frame to advance any running animations.
    /// Returns `true` if any animation is still in progress (needs more frames).
    /// 
    /// The default implementation recursively ticks all children.
    fn tick(&mut self, delta: f32) -> bool {
        let mut animating = false;
        for child in self.children_mut() {
            if child.tick(delta) {
                animating = true;
            }
        }
        animating
    }
    
    // ========================================================================
    // Dynamic/Static Classification (for Snapshots)
    // ========================================================================
    
    /// Check if this render object is considered "dynamic"
    /// 
    /// Dynamic objects are those that:
    /// - Have running animations
    /// - Are interactive (gesture detectors)
    /// - Have real-time content (video, live data)
    /// 
    /// During page transitions, dynamic objects are kept alive and continue
    /// to receive tick updates, while static objects are rendered to a texture.
    /// 
    /// Default implementation checks if any animation is running.
    fn is_dynamic(&self) -> bool {
        // By default, consider dynamic if any child is animating
        for child in self.children() {
            if child.is_dynamic() {
                return true;
            }
        }
        false
    }
    
    /// Check if this render object can be cached as a texture layer
    /// 
    /// Some objects (like video players) may not support being cached.
    /// Override and return false to prevent texture caching.
    fn supports_layer_cache(&self) -> bool {
        true
    }
    
    // ========================================================================
    // Children
    // ========================================================================
    
    /// Get mutable references to child render objects
    fn children_mut(&mut self) -> Vec<&mut dyn RenderObject> {
        Vec::new()
    }
    
    /// Get references to child render objects
    fn children(&self) -> Vec<&dyn RenderObject> {
        Vec::new()
    }
    
    /// Add a child render object
    fn add_child(&mut self, _child: Box<dyn RenderObject>) {
        // Default: no children supported
    }
    
    /// Remove a child at the given index
    fn remove_child(&mut self, _index: usize) -> Option<Box<dyn RenderObject>> {
        None
    }
    
    /// Insert a child at the given index
    fn insert_child(&mut self, _index: usize, _child: Box<dyn RenderObject>) {
        // Default: no children supported
    }
    
    // ========================================================================
    // Dirty Flags
    // ========================================================================
    
    /// Check if layout is needed
    fn needs_layout(&self) -> bool;
    
    /// Mark that layout is needed
    fn mark_needs_layout(&mut self);
    
    /// Check if paint is needed
    fn needs_paint(&self) -> bool;
    
    /// Mark that paint is needed
    fn mark_needs_paint(&mut self);
    
    // ========================================================================
    // Type Operations
    // ========================================================================
    
    /// Convert to Any for downcasting
    fn as_any(&self) -> &dyn Any;
    
    /// Convert to mutable Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Base state for render objects
/// 
/// Most render objects can include this struct to handle common state.
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
    /// Create a new RenderObjectState with default value
    pub fn new() -> Self {
        Self {
            offset: Offset::zero(),
            size: Size::zero(),
            needs_layout: true,
            needs_paint: true,
            is_mounted: false,
        }
    }
    
    /// Get the RenderObject's bounds
    pub fn get_rect(&self) -> Rect {
        Rect::from_offset_size(self.offset, self.size)
    }
    
    /// Mark this object as needing layout
    pub fn mark_needs_layout(&mut self) {
        self.needs_layout = true;
    }
    
    /// Mark this object as needing paint
    pub fn mark_needs_paint(&mut self) {
        self.needs_paint = true;
    }
}

/// Helper macro for implementing common RenderObject methods
#[macro_export]
macro_rules! impl_render_object_common {
    ($state_field:ident) => {
        fn get_rect(&self) -> hoshimi_shared::Rect {
            self.$state_field.get_rect()
        }
        
        fn set_offset(&mut self, offset: hoshimi_shared::Offset) {
            self.$state_field.offset = offset;
        }
        
        fn get_offset(&self) -> hoshimi_shared::Offset {
            self.$state_field.offset
        }
        
        fn get_size(&self) -> hoshimi_shared::Size {
            self.$state_field.size
        }
        
        fn needs_layout(&self) -> bool {
            self.$state_field.needs_layout
        }
        
        fn mark_needs_layout(&mut self) {
            self.$state_field.needs_layout = true;
        }
        
        fn needs_paint(&self) -> bool {
            self.$state_field.needs_paint
        }
        
        fn mark_needs_paint(&mut self) {
            self.$state_field.needs_paint = true;
        }
        
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    };
}

/// Trait for render objects that can be animated
/// 
/// Implement this trait to enable automatic animation ticking.
pub trait Animatable {
    /// Update the animation state with delta time (in seconds)
    fn update(&mut self, delta: f32);
    
    /// Check if any animation is currently running
    fn is_animating(&self) -> bool;
}

/// Helper macro for implementing common single-child RenderObject methods
/// 
/// This macro generates implementations for:
/// - `children()` / `children_mut()`
/// - `handle_event()` - delegates to child
/// - `on_mount()` / `on_unmount()` - delegates to child
/// 
/// # Usage
/// ```ignore
/// impl RenderObject for MyRenderObject {
///     impl_single_child_render_object!(child);
///     // ... other methods
/// }
/// ```
#[macro_export]
macro_rules! impl_single_child_render_object {
    ($child_field:ident) => {
        fn children(&self) -> Vec<&dyn $crate::render::RenderObject> {
            vec![self.$child_field.as_ref()]
        }
        
        fn children_mut(&mut self) -> Vec<&mut dyn $crate::render::RenderObject> {
            vec![self.$child_field.as_mut()]
        }
        
        fn handle_event(&mut self, event: &$crate::events::InputEvent) -> $crate::events::EventResult {
            self.$child_field.handle_event(event)
        }
        
        fn on_mount(&mut self) {
            self.$child_field.on_mount();
        }
        
        fn on_unmount(&mut self) {
            self.$child_field.on_unmount();
        }
    };
}

/// Helper macro for implementing animated RenderObject tick method
/// 
/// Requires the struct to implement `Animatable` trait and have:
/// - A `state` field of type `RenderObjectState`
/// - A `child` field that is a `Box<dyn RenderObject>`
/// 
/// # Usage
/// ```ignore
/// impl RenderObject for MyAnimatedRenderObject {
///     impl_animated_tick!(state, child);
///     // ... other methods
/// }
/// ```
#[macro_export]
macro_rules! impl_animated_tick {
    ($state_field:ident, $child_field:ident) => {
        fn tick(&mut self, delta: f32) -> bool {
            // Update this object's animation
            $crate::render::Animatable::update(self, delta);
            let self_animating = $crate::render::Animatable::is_animating(self);
            
            // Recursively tick children
            let child_animating = self.$child_field.tick(delta);
            
            // Mark for repaint if animating
            if self_animating {
                self.$state_field.mark_needs_paint();
            }
            
            self_animating || child_animating
        }
    };
}

/// Helper macro for implementing single-child layout
/// 
/// # Usage
/// ```ignore
/// impl RenderObject for MyRenderObject {
///     impl_single_child_layout!(state, child);
///     // ... other methods
/// }
/// ```
#[macro_export]
macro_rules! impl_single_child_layout {
    ($state_field:ident, $child_field:ident) => {
        fn layout(&mut self, constraints: hoshimi_shared::Constraints) -> hoshimi_shared::Size {
            let child_size = self.$child_field.layout(constraints);
            self.$child_field.set_offset(hoshimi_shared::Offset::ZERO);
            self.$state_field.size = child_size;
            child_size
        }
    };
}
