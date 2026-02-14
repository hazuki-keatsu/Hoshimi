//! Widget Module
//!
//! Contains all widget definitions and traits.

mod traits;
pub mod animated;
pub mod basic;

pub use traits::Widget;
pub use animated::*;
pub use basic::*;

pub use crate::state::{StatefulWidget, StatelessWidget, WidgetState};
