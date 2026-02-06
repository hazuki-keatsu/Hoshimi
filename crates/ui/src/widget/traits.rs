//! Widget Trait Definition
//!
//! Widgets are immutable configuration objects that describe the UI structure.
//! They are the "what" of the UI, not the "how".

use std::any::{Any, TypeId};
use std::fmt::Debug;

use crate::key::WidgetKey;
use crate::render::RenderObject;

/// The core Widget trait
/// 
/// Widgets are immutable descriptions of UI configuration.
/// They describe what the UI should look like, not how to render it.
/// 
/// # Design Philosophy
/// 
/// - Widgets are **immutable** - create new widgets instead of mutating
/// - Widgets are **cheap to create** - they're just configuration data
/// - Widgets **don't hold state** - state lives in RenderObjects
/// - Widgets support **cloning** for diff comparisons
pub trait Widget: Debug + Any {
    /// Get the type ID of this widget
    /// 
    /// Used by the diff algorithm to detect widget type changes.
    fn widget_type(&self) -> TypeId {
        TypeId::of::<Self>()
    }
    
    /// Get the optional key for this widget
    /// 
    /// Keys are used to match widgets across rebuilds in lists.
    /// Without a key, widgets are matched by index.
    fn key(&self) -> Option<WidgetKey> {
        None
    }
    
    /// Get the children of this widget
    /// 
    /// Returns empty slice for leaf widgets.
    fn children(&self) -> Vec<&dyn Widget> {
        Vec::new()
    }
    
    /// Create a new RenderObject for this widget
    /// 
    /// Called when this widget is first mounted to the tree.
    /// The returned RenderObject will handle layout and painting.
    fn create_render_object(&self) -> Box<dyn RenderObject>;
    
    /// Update an existing RenderObject with new configuration
    /// 
    /// Called when this widget replaces a widget of the same type.
    /// Should update the RenderObject's configuration without recreating it.
    fn update_render_object(&self, render_object: &mut dyn RenderObject);
    
    /// Determine if the RenderObject needs to be updated
    /// 
    /// Returns true if this widget's configuration differs from the old widget
    /// in a way that requires updating the RenderObject.
    fn should_update(&self, _old: &dyn Widget) -> bool {
        // Default implementation: always update
        // Widgets should override this for better performance
        true
    }
    
    /// Convert to Any for down-casting
    fn as_any(&self) -> &dyn Any;
}

// /// Extension trait for Widget type operations
// pub trait WidgetExt: Widget {
//     /// Downcast to a concrete widget type
//     fn downcast_ref<T: Widget>(&self) -> Option<&T> {
//         self.as_any().downcast_ref::<T>()
//     }
    
//     /// Check if this is a specific widget type
//     fn is<T: Widget>(&self) -> bool {
//         self.as_any().is::<T>()
//     }
// }

// impl<W: Widget> WidgetExt for W {}

// /// A boxed widget for dynamic dispatch
// pub type BoxedWidget = Box<dyn Widget>;

// /// Trait for widgets that can be cloned
// pub trait CloneWidget: Widget {
//     fn clone_widget(&self) -> BoxedWidget;
// }
