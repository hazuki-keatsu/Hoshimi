//! The definition of the error types

use std::fmt;

/// Renderer error types
#[derive(Debug)]
pub enum RendererError {
    /// Fail to Initialize
    InitializationFailed(String),
    /// Fail to create Surface
    SurfaceCreationFailed(String),
    /// Fail to load resource
    ResourceLoadFailed(String),
    /// Fail to load font
    FontLoadFailed(String),
    /// Fail to load image
    ImageLoadFailed(String),
    /// Invalid parameter
    InvalidParameter(String),
    /// Fail to render
    RenderFailed(String),
    /// IO error
    IoError(std::io::Error),
}

impl fmt::Display for RendererError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RendererError::InitializationFailed(msg) => write!(f, "Initialization failed: {}", msg),
            RendererError::SurfaceCreationFailed(msg) => write!(f, "Surface creation failed: {}", msg),
            RendererError::ResourceLoadFailed(msg) => write!(f, "Resource load failed: {}", msg),
            RendererError::FontLoadFailed(msg) => write!(f, "Font load failed: {}", msg),
            RendererError::ImageLoadFailed(msg) => write!(f, "Image load failed: {}", msg),
            RendererError::InvalidParameter(msg) => write!(f, "Invalid parameter: {}", msg),
            RendererError::RenderFailed(msg) => write!(f, "Render failed: {}", msg),
            RendererError::IoError(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for RendererError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RendererError::IoError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for RendererError {
    fn from(err: std::io::Error) -> Self {
        RendererError::IoError(err)
    }
}

/// Alias for RenderResult
pub type RendererResult<T> = Result<T, RendererError>;
