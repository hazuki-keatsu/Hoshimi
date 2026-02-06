//! Painter Trait Definition
//!
//! The Painter trait provides an abstraction over rendering backends,
//! allowing the UI system to be decoupled from specific graphics APIs.

use hoshimi_shared::{
    BorderRadius, BoxDecoration, Color, EdgeInsets, Offset, Rect, Size, TextAlign, TextStyle,
};

/// The Painter trait for rendering operations
/// 
/// Painter provides a high-level drawing API that can be implemented
/// by different rendering backends (Skia, SDL3, WebGPU, etc.)
/// 
/// # Coordinate System
/// 
/// - Origin is at top-left
/// - X increases to the right
/// - Y increases downward
/// - All coordinates are in logical pixels
/// 
/// # State Management
/// 
/// The painter maintains a state stack for:
/// - Transformation matrix
/// - Clipping region
/// 
/// Use `save()` and `restore()` to manage state.
pub trait Painter {
    // ========================================================================
    // State Management
    // ========================================================================
    
    /// Save the current drawing state (transforms, clips)
    fn save(&mut self);
    
    /// Restore the previously saved drawing state
    fn restore(&mut self);
    
    /// Set a rectangular clipping region
    fn clip_rect(&mut self, rect: Rect);
    
    /// Set a rounded rectangle clipping region
    fn clip_rounded_rect(&mut self, rect: Rect, radius: BorderRadius);
    
    // ========================================================================
    // Transforms
    // ========================================================================
    
    /// Translate the coordinate system
    fn translate(&mut self, offset: Offset);
    
    /// Scale the coordinate system
    fn scale(&mut self, sx: f32, sy: f32);
    
    /// Rotate the coordinate system (in radians)
    fn rotate(&mut self, radians: f32);
    
    /// Rotate around a specific point
    fn rotate_around(&mut self, radians: f32, center: Offset);
    
    // ========================================================================
    // Basic Shapes
    // ========================================================================
    
    /// Fill a rectangle with a color
    fn fill_rect(&mut self, rect: Rect, color: Color);
    
    /// Draw a rectangle (alias for fill_rect)
    fn draw_rect(&mut self, rect: Rect, color: Color) {
        self.fill_rect(rect, color);
    }
    
    /// Stroke a rectangle outline
    fn stroke_rect(&mut self, rect: Rect, color: Color, stroke_width: f32);
    
    /// Fill a rounded rectangle
    fn fill_rounded_rect(&mut self, rect: Rect, radius: BorderRadius, color: Color);
    
    /// Draw a rounded rectangle (alias for fill_rounded_rect)
    fn draw_rounded_rect(&mut self, rect: Rect, radius: BorderRadius, color: Color) {
        self.fill_rounded_rect(rect, radius, color);
    }
    
    /// Stroke a rounded rectangle outline
    fn stroke_rounded_rect(&mut self, rect: Rect, radius: BorderRadius, color: Color, stroke_width: f32);
    
    /// Fill a circle
    fn fill_circle(&mut self, center: Offset, radius: f32, color: Color);
    
    /// Stroke a circle outline
    fn stroke_circle(&mut self, center: Offset, radius: f32, color: Color, stroke_width: f32);
    
    /// Draw a line between two points
    fn draw_line(&mut self, start: Offset, end: Offset, color: Color, stroke_width: f32);
    
    // ========================================================================
    // Box Decoration
    // ========================================================================
    
    /// Draw a decorated box (background, border, shadow)
    fn draw_box_decoration(&mut self, rect: Rect, decoration: &BoxDecoration) {
        // Draw shadow first (behind everything)
        if let Some(shadow) = &decoration.box_shadow {
            let shadow_rect = rect.translate(shadow.offset);
            let shadow_color = shadow.color;
            
            if let Some(radius) = &decoration.border_radius {
                self.fill_rounded_rect(shadow_rect, *radius, shadow_color);
            } else {
                self.fill_rect(shadow_rect, shadow_color);
            }
        }
        
        // Draw background
        if let Some(color) = decoration.color {
            if let Some(radius) = &decoration.border_radius {
                self.fill_rounded_rect(rect, *radius, color);
            } else {
                self.fill_rect(rect, color);
            }
        }
        
        // Draw border
        if let Some(border) = &decoration.border {
            if let Some(radius) = &decoration.border_radius {
                self.stroke_rounded_rect(rect, *radius, border.color, border.width);
            } else {
                self.stroke_rect(rect, border.color, border.width);
            }
        }
    }
    
    // ========================================================================
    // Text
    // ========================================================================
    
    /// Draw text at the specified position
    /// 
    /// Position is the top-left corner of the text bounding box.
    fn draw_text(&mut self, text: &str, position: Offset, style: &TextStyle);
    
    /// Draw text centered within a rectangle
    fn draw_text_centered(&mut self, text: &str, rect: Rect, style: &TextStyle);
    
    /// Draw text aligned within a rectangle
    fn draw_text_aligned(&mut self, text: &str, rect: Rect, style: &TextStyle, align: TextAlign) {
        let size = self.measure_text(text, style);
        
        let x = match align {
            TextAlign::Left => rect.x,
            TextAlign::Center => rect.x + (rect.width - size.width) / 2.0,
            TextAlign::Right => rect.x + rect.width - size.width,
            TextAlign::Justify => rect.x, // For single line, same as left
        };
        
        // Vertically center
        let y = rect.y + (rect.height - size.height) / 2.0;
        
        self.draw_text(text, Offset::new(x, y), style);
    }
    
    // ========================================================================
    // Images
    // ========================================================================
    
    /// Draw an image at the specified position
    fn draw_image(&mut self, image_key: &str, position: Offset);
    
    /// Draw an image to a rectangle with optional opacity
    fn draw_image_to_rect(&mut self, image_key: &str, rect: Rect, opacity: f32) {
        // Default implementation: just draw at position
        // Implementations should override for proper scaling and opacity
        if opacity > 0.0 {
            self.draw_image(image_key, rect.origin());
        }
    }
    
    /// Draw an image with opacity
    fn draw_image_with_alpha(&mut self, image_key: &str, position: Offset, alpha: f32);
    
    /// Draw an image scaled to fit a rectangle
    fn draw_image_rect(&mut self, image_key: &str, dest_rect: Rect);
    
    /// Draw a portion of an image (source rect) to a destination rect
    fn draw_image_rect_src(&mut self, image_key: &str, src_rect: Rect, dest_rect: Rect);

    /// Get the size of a cached image
    /// 
    /// Returns None if the image is not loaded.
    fn get_image_size(&self, image_key: &str) -> Option<Size>;
    
    /// Draw a nine-patch image
    /// 
    /// Nine-patch divides the image into 9 regions:
    /// - 4 corners (not stretched)
    /// - 4 edges (stretched in one direction)
    /// - 1 center (stretched in both directions)
    fn draw_nine_patch(&mut self, image_key: &str, dest_rect: Rect, insets: EdgeInsets);
    
    // ========================================================================
    // Measurement
    // ========================================================================
    
    /// Measure the size of text
    fn measure_text(&self, text: &str, style: &TextStyle) -> Size;
    
    /// Get the line height for a text style
    fn line_height(&self, style: &TextStyle) -> f32;
    
    /// Measure character positions within text
    /// 
    /// Returns the X offset of each character boundary.
    fn measure_char_positions(&self, text: &str, style: &TextStyle) -> Vec<f32>;
    
    // ========================================================================
    // Utility
    // ========================================================================
    
    /// Get the canvas size
    fn canvas_size(&self) -> Size;
}

/// Text measurement utility trait
/// 
/// Separated from Painter for cases where only measurement is needed.
pub trait TextMeasurer {
    /// Measure the size of text
    fn measure_text(&self, text: &str, style: &TextStyle) -> Size;
    
    /// Get the line height for a text style
    fn line_height(&self, style: &TextStyle) -> f32;
    
    /// Measure character positions within text
    fn measure_char_positions(&self, text: &str, style: &TextStyle) -> Vec<f32>;
}
