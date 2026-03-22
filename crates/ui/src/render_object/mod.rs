//! Render Module
//!
//! Contains RenderObject trait and related types.
//!
//! # Architecture
//!
//! The RenderObject system is split into multiple traits for better separation of concerns:
//! - [`Layoutable`] - Layout computation and geometry
//! - [`Paintable`] - Rendering and painting
//! - [`EventHandlable`] - Input event handling
//! - [`Lifecycle`] - Mount/unmount lifecycle
//! - [`Parent`] - Children management
//! - [`Animatable`] - Animation support
//! - [`RenderObject`] - Composite trait combining all capabilities

mod traits;

pub use traits::{
    Animatable, EmptyRenderObject, EventHandlable, Layoutable, Lifecycle, Paintable, Parent,
    RenderObject, RenderObjectState,
};
