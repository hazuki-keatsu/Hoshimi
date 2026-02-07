//! Page Trait
//!
//! Defines the Page trait for managing page lifecycle and UI construction.
//! Pages are the fundamental unit of navigation in the router system.

use std::any::Any;
use std::fmt::Debug;

use super::types::{PageParams, TransitionType};
use crate::widget::Widget;

/// The core Page trait for router-managed pages
/// 
/// Pages represent individual screens or views in the application.
/// They manage their own lifecycle and construct their UI tree on demand.
/// 
/// # Lifecycle
/// 
/// A page goes through the following lifecycle states:
/// 
/// 1. **Create** (`on_create`) - Page is created with optional parameters
/// 2. **Resume** (`on_resume`) - Page becomes visible and active
/// 3. **Pause** (`on_pause`) - Page is covered by another page
/// 4. **Destroy** (`on_destroy`) - Page is removed from the stack
/// 
/// ```text
/// ┌─────────┐     ┌──────────┐     ┌─────────┐
/// │ Create  │ ──► │  Resume  │ ◄── │  Pause  │
/// └─────────┘     └──────────┘     └─────────┘
///                      │                │
///                      ▼                │
///                 ┌─────────┐           │
///                 │ Active  │ ◄─────────┘
///                 └─────────┘
///                      │
///                      ▼
///                 ┌─────────┐
///                 │ Destroy │
///                 └─────────┘
/// ```
/// 
/// # Example
/// 
/// ```ignore
/// use hoshimi_ui::router::{Page, PageParams, TransitionType};
/// use hoshimi_ui::widget::*;
/// 
/// #[derive(Debug)]
/// struct HomePage {
///     welcome_message: String,
/// }
/// 
/// impl Page for HomePage {
///     fn route_name(&self) -> &str {
///         "home"
///     }
///     
///     fn on_create(&mut self, params: PageParams) {
///         if let Some(name) = params.get_string("user_name") {
///             self.welcome_message = format!("Welcome, {}!", name);
///         }
///     }
///     
///     fn build(&self) -> Box<dyn Widget> {
///         Box::new(
///             Center::new(
///                 Text::new(&self.welcome_message)
///             )
///         )
///     }
/// }
/// ```
pub trait Page: Debug + Any {
    // ========================================================================
    // Route Identity
    // ========================================================================
    
    /// Get the route name for this page
    /// 
    /// This should be a unique identifier used for named routing.
    fn route_name(&self) -> &str;
    
    // ========================================================================
    // Lifecycle Methods
    // ========================================================================
    
    /// Called when the page is created
    /// 
    /// Use this to:
    /// - Initialize page state from parameters
    /// - Load required resources
    /// - Set up initial data
    fn on_create(&mut self, _params: PageParams) {}
    
    /// Called when the page becomes visible and active
    /// 
    /// Use this to:
    /// - Start animations
    /// - Resume paused operations
    /// - Refresh data if needed
    fn on_resume(&mut self) {}
    
    /// Called when the page is covered by another page
    /// 
    /// Use this to:
    /// - Pause animations
    /// - Save state
    /// - Release temporary resources
    fn on_pause(&mut self) {}
    
    /// Called when the page is about to be destroyed
    /// 
    /// Use this to:
    /// - Clean up resources
    /// - Save persistent state
    /// - Cancel pending operations
    fn on_destroy(&mut self) {}
    
    // ========================================================================
    // UI Construction
    // ========================================================================
    
    /// Build the UI tree for this page
    /// 
    /// Called whenever the page needs to render its UI.
    /// Returns a Widget that represents the entire page content.
    /// 
    /// This method may be called multiple times during the page's lifecycle,
    /// typically when the UI needs to be updated due to state changes.
    fn build(&self) -> Box<dyn Widget>;
    
    /// Check if the page needs to rebuild its UI
    /// 
    /// Return true if the page's state has changed and UI needs to be updated.
    /// The router will automatically call `build()` again when this returns true.
    /// 
    /// Default implementation returns false. Override to implement state management.
    fn needs_rebuild(&self) -> bool {
        false
    }
    
    /// Mark that the rebuild has been completed
    /// 
    /// Called by the router after successfully rebuilding the UI.
    /// Override this if you're tracking rebuild state manually.
    fn mark_rebuilt(&mut self) {}
    
    // ========================================================================
    // Transition Preferences
    // ========================================================================
    
    /// Get the preferred enter transition for this page
    /// 
    /// Override to customize how this page appears when pushed onto the stack.
    fn enter_transition(&self) -> TransitionType {
        TransitionType::slide_left()
    }
    
    /// Get the preferred exit transition for this page
    /// 
    /// Override to customize how this page disappears when popped from the stack.
    fn exit_transition(&self) -> TransitionType {
        TransitionType::slide_right()
    }
    
    // ========================================================================
    // Type Operations
    // ========================================================================
    
    /// Convert to Any for downcasting
    fn as_any(&self) -> &dyn Any;
    
    /// Convert to mutable Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Factory function type for creating pages from named routes
pub type PageFactory = Box<dyn Fn() -> Box<dyn Page> + Send + Sync>;

/// Helper macro for implementing common Page methods
/// 
/// # Usage
/// ```ignore
/// #[derive(Debug)]
/// struct MyPage;
/// 
/// impl Page for MyPage {
///     impl_page_common!();
///     
///     fn route_name(&self) -> &str { "my_page" }
///     fn build(&self) -> Box<dyn Widget> { /* ... */ }
/// }
/// ```
#[macro_export]
macro_rules! impl_page_common {
    () => {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    };
}

/// A simple page wrapper that wraps a widget-building function
/// 
/// Useful for simple pages that don't need complex lifecycle management.
/// 
/// # Example
/// 
/// ```ignore
/// use hoshimi_ui::router::SimplePage;
/// use hoshimi_ui::widget::*;
/// 
/// let page = SimplePage::new("home", || {
///     Box::new(Center::new(Text::new("Hello, World!")))
/// });
/// ```
pub struct SimplePage<F>
where
    F: Fn() -> Box<dyn Widget> + 'static,
{
    name: String,
    builder: F,
    enter_transition: TransitionType,
    exit_transition: TransitionType,
}

impl<F> Debug for SimplePage<F>
where
    F: Fn() -> Box<dyn Widget> + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimplePage")
            .field("name", &self.name)
            .field("enter_transition", &self.enter_transition)
            .field("exit_transition", &self.exit_transition)
            .finish()
    }
}

impl<F> SimplePage<F>
where
    F: Fn() -> Box<dyn Widget> + 'static,
{
    /// Create a new simple page
    pub fn new(name: impl Into<String>, builder: F) -> Self {
        Self {
            name: name.into(),
            builder,
            enter_transition: TransitionType::slide_left(),
            exit_transition: TransitionType::slide_right(),
        }
    }
    
    /// Set the enter transition
    pub fn with_enter_transition(mut self, transition: TransitionType) -> Self {
        self.enter_transition = transition;
        self
    }
    
    /// Set the exit transition
    pub fn with_exit_transition(mut self, transition: TransitionType) -> Self {
        self.exit_transition = transition;
        self
    }
}

impl<F> Page for SimplePage<F>
where
    F: Fn() -> Box<dyn Widget> + 'static,
{
    fn route_name(&self) -> &str {
        &self.name
    }
    
    fn build(&self) -> Box<dyn Widget> {
        (self.builder)()
    }
    
    fn enter_transition(&self) -> TransitionType {
        self.enter_transition.clone()
    }
    
    fn exit_transition(&self) -> TransitionType {
        self.exit_transition.clone()
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
