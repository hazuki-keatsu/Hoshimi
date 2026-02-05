//! # Hoshimi Renderer
//! 
//! Provides high-performance rendering capabilities based on Skia, 
//! encapsulating the underlying graphics API.
//! 
//! 
//! ## Structure
//! 
//! - `types` - Basic UI Elements
//! - `error` - Error Types
//! - `scene_renderer` - Scene Renderer Core
//! 
//! ## Example
//! 
//! ```ignore
//! use hoshimi_renderer::{SceneRenderer, UIColor, UIRect};
//! 
//! let mut renderer = SceneRenderer::new(1920, 1080)?;
//! 
//! renderer.begin_frame(Some(UIColor::black()))?;
//! renderer.set_color(UIColor::red())?;
//! renderer.draw_rect(UIRect::from_xywh(100.0, 100.0, 200.0, 150.0))?;
//! renderer.end_frame()?;
//! ```

pub mod types;
pub mod error;
pub mod scene_renderer;

// Re-exports for convenience
pub use types::*;
pub use error::{RendererError, RendererResult};
pub use scene_renderer::SceneRenderer;