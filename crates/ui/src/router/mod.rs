//! Router Module
//!
//! A complete routing system for page-based navigation with animated transitions.
//! 
//! # Overview
//! 
//! The router module provides:
//! - **Page Stack**: A navigation stack for managing page hierarchy
//! - **Named Routes**: Registration and lookup of pages by name
//! - **Transitions**: Animated transitions between pages
//! - **Page Lifecycle**: Full lifecycle management for pages
//! - **Snapshots**: Efficient page snapshots for smooth transitions
//! 
//! # Architecture
//! 
//! ```text
//! ┌──────────────────────────────────────────────────────────────┐
//! │                         Router                               │
//! │  ┌─────────────────────────────────────────────────────────┐ │
//! │  │                    Navigation Stack                     │ │
//! │  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐     │ │
//! │  │  │ Page 1  │  │ Page 2  │  │ Page 3  │  │ Page N  │     │ │
//! │  │  │(Paused) │  │(Paused) │  │(Paused) │  │(Active) │     │ │
//! │  │  └─────────┘  └─────────┘  └─────────┘  └─────────┘     │ │
//! │  └─────────────────────────────────────────────────────────┘ │
//! │  ┌─────────────────────────────────────────────────────────┐ │
//! │  │                Active Transition (optional)             │ │
//! │  │  from_snapshot ──────────────────────► to_snapshot      │ │
//! │  └─────────────────────────────────────────────────────────┘ │
//! └──────────────────────────────────────────────────────────────┘
//! ```
//! 
//! # Example
//! 
//! ```ignore
//! use hoshimi_ui::router::{Router, Page, PageParams, TransitionType};
//! use hoshimi_ui::widget::*;
//! 
//! // Define a page
//! #[derive(Debug)]
//! struct HomePage;
//! 
//! impl Page for HomePage {
//!     fn route_name(&self) -> &str { "home" }
//!     fn build(&self) -> Box<dyn Widget> {
//!         Box::new(Center::new(Text::new("Home Page")))
//!     }
//!     impl_page_common!();
//! }
//! 
//! // Create and use router
//! let mut router = Router::new();
//! router.push(HomePage);
//! 
//! // In your main loop:
//! router.tick(delta_time);
//! router.paint(&mut painter);
//! ```

mod page;
mod snapshot;
mod transition;
mod types;

pub use page::{Page, PageFactory, SimplePage};
pub use snapshot::{
    DynamicNodeRef, PageSnapshot, SnapshotBuilder, SnapshotOptions,
    SnapshotPainter, SnapshotPainterExt, SurfaceHandle, TextureHandle,
};
pub use transition::{
    ActiveTransition, PageTransform, TransitionBuilder, TransitionState,
    presets as transition_presets,
};
pub use types::{
    CustomTransitionBuilder, NavigationOptions, NavigationResult,
    PageParams, PageState, ScaleAnchor, SlideDirection, TransitionType,
};

use std::collections::HashMap;

use hoshimi_types::{Constraints, Offset, Rect, Size};
use tracing::{debug, warn};

use crate::events::{EventResult, InputEvent, UIMessage};
use crate::painter::Painter;
use crate::tree::UiTree;

// ============================================================================
// Page Entry
// ============================================================================

/// An entry in the router's navigation stack
struct PageEntry {
    /// The page instance
    page: Box<dyn Page>,
    /// The page's UI tree
    ui_tree: UiTree,
    /// Current page state
    state: PageState,
    /// Cached snapshot (created when transitioning)
    snapshot: Option<PageSnapshot>,
    /// Whether the UI needs to be rebuilt
    needs_rebuild: bool,
}

impl std::fmt::Debug for PageEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PageEntry")
            .field("route_name", &self.page.route_name())
            .field("state", &self.state)
            .field("has_snapshot", &self.snapshot.is_some())
            .finish()
    }
}

impl PageEntry {
    /// Create a new page entry
    fn new(page: Box<dyn Page>) -> Self {
        Self {
            page,
            ui_tree: UiTree::new(),
            state: PageState::Active,
            snapshot: None,
            needs_rebuild: true,
        }
    }
    
    /// Build or rebuild the UI tree from the page
    fn rebuild_ui(&mut self) {
        let widget = self.page.build();
        self.ui_tree.set_root_boxed(widget);
        self.needs_rebuild = false;
        self.page.mark_rebuilt();
    }
    
    /// Check if UI needs rebuild and rebuild if necessary
    fn rebuild_if_needed(&mut self) {
        if self.needs_rebuild || self.page.needs_rebuild() {
            self.rebuild_ui();
        }
    }
    
    /// Set constraints on the UI tree
    fn set_constraints(&mut self, constraints: Constraints) {
        self.ui_tree.set_constraints(constraints);
    }
}

// ============================================================================
// Router
// ============================================================================

/// The main router that manages page navigation
/// 
/// Router provides:
/// - A navigation stack for page management
/// - Named route registration and lookup
/// - Animated page transitions
/// - Full page lifecycle management
pub struct Router {
    /// Navigation stack of pages
    stack: Vec<PageEntry>,
    
    /// Named route factories
    named_routes: HashMap<String, PageFactory>,
    
    /// Currently active transition
    active_transition: Option<ActiveTransition>,
    
    /// Layout constraints for pages
    constraints: Constraints,
    
    /// Pending messages from event handling
    pending_messages: Vec<UIMessage>,
    
    /// Default transition for push operations
    default_push_transition: TransitionType,
    
    /// Default transition for pop operations
    default_pop_transition: TransitionType,
    
    /// Whether transitions are enabled
    transitions_enabled: bool,
}

impl std::fmt::Debug for Router {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Router")
            .field("stack_depth", &self.stack.len())
            .field("named_routes_count", &self.named_routes.len())
            .field("has_active_transition", &self.active_transition.is_some())
            .field("transitions_enabled", &self.transitions_enabled)
            .finish()
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

impl Router {
    /// Create a new empty router
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            named_routes: HashMap::new(),
            active_transition: None,
            constraints: Constraints::loose(Size::new(800.0, 600.0)),
            pending_messages: Vec::new(),
            default_push_transition: TransitionType::slide_left(),
            default_pop_transition: TransitionType::slide_right(),
            transitions_enabled: true,
        }
    }
    
    /// Create a router with an initial page
    pub fn with_initial_page(page: impl Page + 'static) -> Self {
        let mut router = Self::new();
        router.push_without_transition(page);
        router
    }
    
    // ========================================================================
    // Configuration
    // ========================================================================
    
    /// Set the layout constraints for all pages
    pub fn set_constraints(&mut self, constraints: Constraints) {
        self.constraints = constraints;
        
        // Update constraints on all page entries
        for entry in &mut self.stack {
            entry.set_constraints(constraints);
        }
    }
    
    /// Set size from width and height
    pub fn set_size(&mut self, width: f32, height: f32) {
        self.set_constraints(Constraints::loose(Size::new(width, height)));
    }
    
    /// Set the default push transition
    pub fn set_default_push_transition(&mut self, transition: TransitionType) {
        self.default_push_transition = transition;
    }
    
    /// Set the default pop transition
    pub fn set_default_pop_transition(&mut self, transition: TransitionType) {
        self.default_pop_transition = transition;
    }
    
    /// Enable or disable transitions
    pub fn set_transitions_enabled(&mut self, enabled: bool) {
        self.transitions_enabled = enabled;
    }
    
    // ========================================================================
    // Named Routes
    // ========================================================================
    
    /// Register a named route
    /// 
    /// # Example
    /// ```ignore
    /// router.register_route("settings", || Box::new(SettingsPage::new()));
    /// ```
    pub fn register_route<F>(&mut self, name: impl Into<String>, factory: F)
    where
        F: Fn() -> Box<dyn Page> + Send + Sync + 'static,
    {
        self.named_routes.insert(name.into(), Box::new(factory));
    }
    
    /// Unregister a named route
    pub fn unregister_route(&mut self, name: &str) -> bool {
        self.named_routes.remove(name).is_some()
    }
    
    /// Check if a named route exists
    pub fn has_route(&self, name: &str) -> bool {
        self.named_routes.contains_key(name)
    }
    
    // ========================================================================
    // Navigation Operations
    // ========================================================================
    
    /// Push a new page onto the stack
    /// 
    /// The current page (if any) will be paused and the new page will become active.
    /// An animated transition will play between the pages.
    pub fn push(&mut self, page: impl Page + 'static) -> NavigationResult {
        self.push_with_options(page, PageParams::new(), NavigationOptions::default())
    }
    
    /// Push a new page with parameters
    pub fn push_with_params(
        &mut self,
        page: impl Page + 'static,
        params: PageParams,
    ) -> NavigationResult {
        self.push_with_options(page, params, NavigationOptions::default())
    }
    
    /// Push a new page with full options
    pub fn push_with_options(
        &mut self,
        page: impl Page + 'static,
        params: PageParams,
        options: NavigationOptions,
    ) -> NavigationResult {
        // Check if transition is already in progress
        if self.active_transition.is_some() {
            warn!("Cannot push: transition already in progress");
            return NavigationResult::TransitionInProgress;
        }
        
        let boxed_page: Box<dyn Page> = Box::new(page);
        self.push_boxed_with_options(boxed_page, params, options)
    }
    
    /// Push a boxed page with options
    fn push_boxed_with_options(
        &mut self,
        mut page: Box<dyn Page>,
        params: PageParams,
        options: NavigationOptions,
    ) -> NavigationResult {
        debug!("Pushing page: {}", page.route_name());
        
        // Handle clear stack option
        if options.clear_stack {
            self.clear_stack();
        }
        
        // Handle replace option
        if options.replace && !self.stack.is_empty() {
            self.pop_without_transition();
        }
        
        // Initialize the page
        page.on_create(params);
        
        // Create page entry
        let mut entry = PageEntry::new(page);
        entry.set_constraints(self.constraints);
        entry.rebuild_ui();
        entry.page.on_resume();
        
        // Determine transition type
        let transition_type = options.transition
            .unwrap_or_else(|| entry.page.enter_transition());
        
        // Start transition if enabled and there's a current page
        if self.transitions_enabled && !self.stack.is_empty() && transition_type.duration() > 0.0 {
            // Pause current page
            if let Some(current) = self.stack.last_mut() {
                current.page.on_pause();
                current.state = PageState::TransitioningOut;
            }
            
            entry.state = PageState::TransitioningIn;
            
            // Create snapshots and start transition
            // Note: In a real implementation, we'd create proper snapshots here
            let size = self.constraints.biggest();
            let from_snapshot = PageSnapshot::empty(size);
            let to_snapshot = PageSnapshot::empty(size);
            
            let transition = ActiveTransition::new(
                from_snapshot,
                to_snapshot,
                transition_type,
                size,
                false,
            );
            
            self.active_transition = Some(transition);
        }
        
        self.stack.push(entry);
        NavigationResult::Success
    }
    
    /// Push a page without any transition animation
    pub fn push_without_transition(&mut self, page: impl Page + 'static) {
        let mut page = Box::new(page) as Box<dyn Page>;
        page.on_create(PageParams::new());
        
        // Pause current page
        if let Some(current) = self.stack.last_mut() {
            current.page.on_pause();
            current.state = PageState::Paused;
        }
        
        // Create and add new entry
        let mut entry = PageEntry::new(page);
        entry.set_constraints(self.constraints);
        entry.rebuild_ui();
        entry.page.on_resume();
        entry.state = PageState::Active;
        
        self.stack.push(entry);
    }
    
    /// Push a named route
    pub fn push_named(&mut self, name: &str) -> NavigationResult {
        self.push_named_with_params(name, PageParams::new())
    }
    
    /// Push a named route with parameters
    pub fn push_named_with_params(&mut self, name: &str, params: PageParams) -> NavigationResult {
        if let Some(factory) = self.named_routes.get(name) {
            let page = factory();
            self.push_with_params_boxed(page, params)
        } else {
            warn!("Route not found: {}", name);
            NavigationResult::RouteNotFound
        }
    }
    
    /// Internal helper for pushing boxed pages with params
    fn push_with_params_boxed(
        &mut self,
        page: Box<dyn Page>,
        params: PageParams,
    ) -> NavigationResult {
        self.push_boxed_with_options(page, params, NavigationOptions::default())
    }
    
    /// Pop the current page from the stack
    /// 
    /// Returns to the previous page with an animated transition.
    pub fn pop(&mut self) -> NavigationResult {
        self.pop_with_options(NavigationOptions::default())
    }
    
    /// Pop with custom options
    pub fn pop_with_options(&mut self, options: NavigationOptions) -> NavigationResult {
        // Check if transition is already in progress
        if self.active_transition.is_some() {
            warn!("Cannot pop: transition already in progress");
            return NavigationResult::TransitionInProgress;
        }
        
        // Check if stack has pages to pop
        if self.stack.len() <= 1 {
            warn!("Cannot pop: stack is empty or has only one page");
            return NavigationResult::StackEmpty;
        }
        
        debug!("Popping page");
        
        // Get the transition type from the current page
        let transition_type = options.transition
            .unwrap_or_else(|| {
                self.stack.last()
                    .map(|e| e.page.exit_transition())
                    .unwrap_or(self.default_pop_transition.clone())
            });
        
        // Start transition if enabled
        if self.transitions_enabled && transition_type.duration() > 0.0 {
            // Mark current page as transitioning out
            if let Some(current) = self.stack.last_mut() {
                current.state = PageState::TransitioningOut;
            }
            
            // Mark previous page as transitioning in
            let stack_len = self.stack.len();
            if stack_len >= 2 {
                self.stack[stack_len - 2].state = PageState::TransitioningIn;
            }
            
            // Create snapshots and start transition
            let size = self.constraints.biggest();
            let from_snapshot = PageSnapshot::empty(size);
            let to_snapshot = PageSnapshot::empty(size);
            
            let transition = ActiveTransition::new(
                from_snapshot,
                to_snapshot,
                transition_type,
                size,
                true, // is_reverse for pop
            );
            
            self.active_transition = Some(transition);
        } else {
            // Immediate pop
            self.complete_pop();
        }
        
        NavigationResult::Success
    }
    
    /// Pop without any transition animation
    pub fn pop_without_transition(&mut self) -> bool {
        if self.stack.len() <= 1 {
            return false;
        }
        
        self.complete_pop();
        true
    }
    
    /// Complete a pop operation (after transition or immediately)
    fn complete_pop(&mut self) {
        if let Some(mut entry) = self.stack.pop() {
            // Lifecycle: destroy the page
            entry.page.on_pause();
            entry.page.on_destroy();
        }
        
        // Resume the now-current page
        if let Some(current) = self.stack.last_mut() {
            current.page.on_resume();
            current.state = PageState::Active;
        }
    }
    
    /// Pop until a predicate is satisfied
    /// 
    /// Keeps popping pages until the predicate returns true for the current page,
    /// or until only one page remains.
    pub fn pop_until<F>(&mut self, predicate: F) -> NavigationResult
    where
        F: Fn(&dyn Page) -> bool,
    {
        while self.stack.len() > 1 {
            if let Some(current) = self.stack.last() {
                if predicate(current.page.as_ref()) {
                    break;
                }
            }
            
            if !self.pop_without_transition() {
                break;
            }
        }
        
        NavigationResult::Success
    }
    
    /// Pop until a named route is reached
    pub fn pop_until_named(&mut self, route_name: &str) -> NavigationResult {
        self.pop_until(|page| page.route_name() == route_name)
    }
    
    /// Clear the entire stack (except optionally keeping the bottom page)
    pub fn clear_stack(&mut self) {
        while self.stack.len() > 0 {
            if let Some(mut entry) = self.stack.pop() {
                entry.page.on_pause();
                entry.page.on_destroy();
            }
        }
    }
    
    /// Replace the current page with a new one
    pub fn replace(&mut self, page: impl Page + 'static) -> NavigationResult {
        self.push_with_options(
            page,
            PageParams::new(),
            NavigationOptions::default().replace(),
        )
    }
    
    /// Replace with parameters
    pub fn replace_with_params(
        &mut self,
        page: impl Page + 'static,
        params: PageParams,
    ) -> NavigationResult {
        self.push_with_options(
            page,
            params,
            NavigationOptions::default().replace(),
        )
    }
    
    // ========================================================================
    // Stack Inspection
    // ========================================================================
    
    /// Get the current stack depth
    pub fn stack_depth(&self) -> usize {
        self.stack.len()
    }
    
    /// Check if the stack is empty
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }
    
    /// Check if a transition is currently active
    pub fn is_transitioning(&self) -> bool {
        self.active_transition.is_some()
    }
    
    /// Get the current page (if any)
    pub fn current_page(&self) -> Option<&dyn Page> {
        self.stack.last().map(|e| e.page.as_ref())
    }
    
    /// Get mutable access to the current page
    pub fn current_page_mut(&mut self) -> Option<&mut dyn Page> {
        self.stack.last_mut().map(|e| e.page.as_mut())
    }
    
    /// Check if the current page can be popped
    pub fn can_pop(&self) -> bool {
        self.stack.len() > 1 && self.active_transition.is_none()
    }
    
    // ========================================================================
    // Update & Rendering
    // ========================================================================
    
    /// Update animations and transitions
    /// 
    /// Call this every frame before painting.
    /// Returns true if animations are still running.
    pub fn tick(&mut self, delta: f32) -> bool {
        let mut animating = false;
        
        // Check and rebuild UI if page state changed
        if self.active_transition.is_none() {
            if let Some(entry) = self.stack.last_mut() {
                entry.rebuild_if_needed();
            }
        }
        
        // Update active transition
        if let Some(ref mut transition) = self.active_transition {
            if transition.tick(delta) {
                animating = true;
            } else if transition.is_complete() {
                // Transition completed
                self.complete_transition();
            }
        }
        
        // Update current page's UI tree if not transitioning
        if self.active_transition.is_none() {
            if let Some(entry) = self.stack.last_mut() {
                if entry.ui_tree.tick(delta) {
                    animating = true;
                }
            }
        } else {
            // During transition, tick both pages
            let stack_len = self.stack.len();
            if stack_len >= 1 {
                if self.stack[stack_len - 1].ui_tree.tick(delta) {
                    animating = true;
                }
            }
            if stack_len >= 2 {
                if self.stack[stack_len - 2].ui_tree.tick(delta) {
                    animating = true;
                }
            }
        }
        
        animating
    }
    
    /// Complete a transition
    fn complete_transition(&mut self) {
        let transition = self.active_transition.take();
        
        if let Some(transition) = transition {
            if transition.is_complete() {
                if transition.is_reverse() {
                    // Pop transition completed — remove the leaving page
                    self.complete_pop();
                } else {
                    // Push transition completed — update page states
                    for entry in &mut self.stack {
                        match entry.state {
                            PageState::TransitioningOut => {
                                entry.state = PageState::Paused;
                            }
                            PageState::TransitioningIn => {
                                entry.state = PageState::Active;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
    
    /// Paint the current state to the painter
    pub fn paint(&mut self, painter: &mut dyn Painter) {
        // If there's an active transition, paint both pages with transforms
        if let Some(ref transition) = self.active_transition {
            let stack_len = self.stack.len();
            if stack_len < 2 {
                // Fallback: just paint current page
                if let Some(entry) = self.stack.last_mut() {
                    entry.ui_tree.paint(painter);
                }
                return;
            }

            let (from_xform, to_xform) = transition.get_live_transforms();
            let screen_rect = Rect::from_size(transition.size());

            // For push: from = stack[len-2] (old page), to = stack[len-1] (new page)
            // For pop:  from = stack[len-1] (leaving page), to = stack[len-2] (returning page)
            let (from_idx, to_idx) = if transition.is_reverse() {
                (stack_len - 1, stack_len - 2)
            } else {
                (stack_len - 2, stack_len - 1)
            };

            // Determine paint order for scale transitions:
            // The page with smaller scale should be painted on top (last)
            // so we can see it shrinking away
            let from_scale = from_xform.scale.map(|(sx, _, _)| sx).unwrap_or(1.0);
            let to_scale = to_xform.scale.map(|(sx, _, _)| sx).unwrap_or(1.0);
            let paint_from_on_top = from_scale < to_scale;

            // Helper to paint a page with transform
            let paint_page = |painter: &mut dyn Painter,
                              xform: &PageTransform,
                              idx: usize,
                              stack: &mut [PageEntry]| {
                painter.save();
                painter.clip_rect(screen_rect);
                if let Some((sx, sy, anchor)) = xform.scale {
                    painter.translate(anchor);
                    painter.scale(sx, sy);
                    painter.translate(Offset::new(-anchor.x, -anchor.y));
                }
                painter.translate(xform.offset);
                stack[idx].ui_tree.paint(painter);
                painter.restore();
            };

            if paint_from_on_top {
                // Paint "to" first (bottom), then "from" on top
                paint_page(painter, &to_xform, to_idx, &mut self.stack);
                paint_page(painter, &from_xform, from_idx, &mut self.stack);
            } else {
                // Paint "from" first (bottom), then "to" on top
                paint_page(painter, &from_xform, from_idx, &mut self.stack);
                paint_page(painter, &to_xform, to_idx, &mut self.stack);
            }
        } else {
            // Paint current page
            if let Some(entry) = self.stack.last_mut() {
                entry.ui_tree.paint(painter);
            }
        }
    }
    
    // ========================================================================
    // Event Handling
    // ========================================================================
    
    /// Push an input event to the event queue (with gesture detection)
    /// 
    /// Events are processed through the gesture detector, which may generate
    /// additional high-level gesture events (Tap, LongPress, etc.)
    /// 
    /// Call `process_events()` after pushing events to dispatch them.
    pub fn push_event(&mut self, event: InputEvent) {
        // Don't accept events during transitions
        if self.active_transition.is_some() {
            return;
        }
        
        if let Some(entry) = self.stack.last_mut() {
            entry.ui_tree.push_event(event);
        }
    }
    
    /// Process all queued events and dispatch them to the UI tree
    /// 
    /// Returns the number of events processed.
    pub fn process_events(&mut self) -> usize {
        // Don't process events during transitions
        if self.active_transition.is_some() {
            return 0;
        }
        
        if let Some(entry) = self.stack.last_mut() {
            entry.ui_tree.process_events()
        } else {
            0
        }
    }
    
    /// Handle an input event directly (bypasses gesture detection)
    /// 
    /// For most cases, prefer using `push_event()` and `process_events()` instead,
    /// which provides automatic gesture detection.
    pub fn handle_event(&mut self, event: &InputEvent) -> EventResult {
        // Don't handle events during transitions
        if self.active_transition.is_some() {
            return EventResult::Ignored;
        }
        
        // Forward to current page
        if let Some(entry) = self.stack.last_mut() {
            entry.ui_tree.handle_event(event)
        } else {
            EventResult::Ignored
        }
    }
    
    /// Take any pending messages from event handling
    pub fn take_messages(&mut self) -> Vec<UIMessage> {
        let mut messages = std::mem::take(&mut self.pending_messages);
        
        // Also collect from current page's UI tree
        if let Some(entry) = self.stack.last_mut() {
            messages.extend(entry.ui_tree.take_messages());
        }
        
        messages
    }
    
    // ========================================================================
    // Page Rebuild
    // ========================================================================
    
    /// Request a rebuild of the current page's UI
    pub fn rebuild_current_page(&mut self) {
        if let Some(entry) = self.stack.last_mut() {
            entry.rebuild_ui();
        }
    }
}

// ============================================================================
// Extension trait for UiTree
// ============================================================================

/// Extension to UiTree for boxed widget support
trait UiTreeExt {
    fn set_root_boxed(&mut self, widget: Box<dyn crate::widget::Widget>);
}

impl UiTreeExt for UiTree {
    fn set_root_boxed(&mut self, widget: Box<dyn crate::widget::Widget>) {
        // Use the existing update mechanism
        // This is a workaround since set_root requires a concrete type
        // In a full implementation, we'd add proper boxed widget support to UiTree
        self.update_root(widget.as_ref());
    }
}
