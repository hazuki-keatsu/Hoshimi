//! Painter Module
//!
//! Provides the Painter trait and implementations for rendering.

mod traits;
mod scene_renderer_painter;

pub use traits::{Painter, TextMeasurer};
pub use scene_renderer_painter::SceneRendererPainter;
