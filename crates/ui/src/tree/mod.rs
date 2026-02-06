//! Tree Module
//!
//! Contains UI tree management structures including:
//! - [`UiTree`] - The main UI tree manager
//! - [`diff`] - Widget diffing algorithm for incremental updates
//! - [`reconciler`] - Applies diff results to the RenderObject tree

mod ui_tree;
pub mod diff;
pub mod reconciler;

pub use ui_tree::UiTree;
pub use diff::{DiffOperation, DiffResult, WidgetDiffer, WidgetIdentity};
pub use reconciler::{Reconciler, ReconcileResult};
