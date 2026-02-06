//! Widget Module
//!
//! Contains all widget definitions and traits.

mod traits;
pub mod basic;
pub mod novel;

pub use traits::Widget;
pub use basic::*;
pub use novel::*;
