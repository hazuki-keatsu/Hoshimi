//! Widget Module
//!
//! Contains all widget definitions and traits.

mod traits;
pub mod animated;
pub mod basic;
pub mod novel;

pub use traits::Widget;
pub use animated::*;
pub use basic::*;
pub use novel::*;
