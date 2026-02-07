//! # Hoshimi Renderer
//! 
//! Provides high-performance rendering capabilities based on Skia, 
//! encapsulating the underlying graphics API.
//! 
//! 
//! ## Structure
//! 
//! - `types` - Re-exported shared types and renderer-specific types
//! - `error` - Error Types
//! - `skia_renderer` - Skia Renderer Core
//! 
//! ## Example
//! 
//! ```ignore
//! use hoshimi_renderer::{SkiaRenderer, Color, Rect};
//! 
//! let mut renderer = SkiaRenderer::new(1920, 1080)?;
//! 
//! renderer.begin_frame(Some(Color::black()))?;
//! renderer.fill_rect(Rect::new(100.0, 100.0, 200.0, 150.0), Color::red())?;
//! renderer.end_frame()?;
//! ```

pub mod types;
pub mod error;
pub mod skia_renderer;

// Re-exports for convenience
pub use types::*;
pub use error::{RendererError, RendererResult};
pub use skia_renderer::SkiaRenderer;