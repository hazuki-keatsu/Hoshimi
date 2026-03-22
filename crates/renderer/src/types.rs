//! Renderer-specific types
//! 
//! This module provides renderer-specific types that are not shared with UI.
//! Common types like Color, Offset, Size, Rect are re-exported from hoshimi_types.

// Re-export shared types for convenience
pub use hoshimi_types::{
    Color, Offset, Size, Rect, BorderRadius, TextAlign,
    // Skia conversion utilities
    rect_to_rrect, rect_to_rrect_uniform,
};

/// Font configuration for text rendering
#[derive(Debug, Clone)]
pub struct UIFont {
    pub path: String,
    pub size: f32,
}

impl UIFont {
    pub fn new(path: impl Into<String>, size: f32) -> Self {
        Self {
            path: path.into(),
            size,
        }
    }
}

/// Paint style for rendering operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PaintStyle {
    #[default]
    Fill,
    Stroke,
    StrokeAndFill,
}

impl From<PaintStyle> for skia_safe::paint::Style {
    fn from(style: PaintStyle) -> Self {
        match style {
            PaintStyle::Fill => skia_safe::paint::Style::Fill,
            PaintStyle::Stroke => skia_safe::paint::Style::Stroke,
            PaintStyle::StrokeAndFill => skia_safe::paint::Style::StrokeAndFill,
        }
    }
}
