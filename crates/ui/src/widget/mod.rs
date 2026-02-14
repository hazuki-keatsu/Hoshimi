//! Widget Module
//!
//! Contains all widget definitions and traits.

mod traits;
pub mod animated;
pub mod basic;
pub mod stateful_widget;
pub mod stateless_widget;

pub use traits::Widget;
pub use animated::*;
pub use basic::*;
