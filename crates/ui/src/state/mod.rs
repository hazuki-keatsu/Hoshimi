//! State Management Module
//!
//! This module provides state management traits for stateful and stateless widgets.
//!
//! # Architecture
//!
//! The state management system follows a three-layer architecture:
//! - **Widget**: Immutable configuration describing the UI structure
//! - **State**: Mutable object managing component state and business logic
//! - **RenderObject**: Mutable object responsible for layout and painting
//!
//! # Stateful vs Stateless
//!
//! - **StatelessWidget**: Widgets that don't have internal state (e.g., Container, Text)
//! - **StatefulWidget**: Widgets that have internal state (e.g., Button, Checkbox)

pub mod traits;

pub use traits::*;
