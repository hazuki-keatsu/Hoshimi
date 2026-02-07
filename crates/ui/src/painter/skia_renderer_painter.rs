//! SkiaRenderer Adapter
//!
//! Implements the Painter trait using the Hoshimi SkiaRenderer.

use hoshimi_renderer::skia_renderer::SkiaRenderer;
use hoshimi_shared::{
    BorderRadius, Color, EdgeInsets, Offset, Rect, Size, TextStyle,
};

use crate::painter::Painter;

/// Adapter that wraps SkiaRenderer to implement the Painter trait
pub struct SkiaRendererPainter<'a> {
    renderer: &'a mut SkiaRenderer,
}

impl<'a> SkiaRendererPainter<'a> {
    /// Create a new painter wrapping the given SkiaRenderer
    pub fn new(renderer: &'a mut SkiaRenderer) -> Self {
        Self { renderer }
    }
}

impl<'a> Painter for SkiaRendererPainter<'a> {
    // ========================================================================
    // State Management
    // ========================================================================
    
    fn save(&mut self) {
        self.renderer.save();
    }
    
    fn restore(&mut self) {
        self.renderer.restore();
    }
    
    fn clip_rect(&mut self, rect: Rect) {
        self.renderer.clip_rect(rect);
    }
    
    fn clip_rounded_rect(&mut self, rect: Rect, radius: BorderRadius) {
        // Use uniform radius if all corners are the same, otherwise use first corner
        let r = radius.uniform_radius().unwrap_or(radius.top_left);
        self.renderer.clip_rounded_rect(rect, r);
    }
    
    // ========================================================================
    // Transforms
    // ========================================================================
    
    fn translate(&mut self, offset: Offset) {
        self.renderer.translate(offset.x, offset.y);
    }
    
    fn scale(&mut self, sx: f32, sy: f32) {
        self.renderer.scale(sx, sy);
    }
    
    fn rotate(&mut self, radians: f32) {
        // SkiaRenderer uses degrees
        self.renderer.rotate(radians.to_degrees());
    }
    
    fn rotate_around(&mut self, radians: f32, center: Offset) {
        self.renderer.rotate_around(radians.to_degrees(), center);
    }
    
    // ========================================================================
    // Basic Shapes
    // ========================================================================
    
    fn fill_rect(&mut self, rect: Rect, color: Color) {
        let _ = self.renderer.fill_rect(rect, color);
    }
    
    fn stroke_rect(&mut self, rect: Rect, color: Color, stroke_width: f32) {
        let _ = self.renderer.stroke_rect(rect, color, stroke_width);
    }
    
    fn fill_rounded_rect(&mut self, rect: Rect, radius: BorderRadius, color: Color) {
        let r = radius.uniform_radius().unwrap_or(radius.top_left);
        let _ = self.renderer.fill_rounded_rect(rect, r, color);
    }
    
    fn stroke_rounded_rect(&mut self, rect: Rect, radius: BorderRadius, color: Color, stroke_width: f32) {
        let r = radius.uniform_radius().unwrap_or(radius.top_left);
        let _ = self.renderer.stroke_rounded_rect(rect, r, color, stroke_width);
    }
    
    fn fill_circle(&mut self, center: Offset, radius: f32, color: Color) {
        let _ = self.renderer.fill_circle(center, radius, color);
    }
    
    fn stroke_circle(&mut self, center: Offset, radius: f32, color: Color, stroke_width: f32) {
        let _ = self.renderer.stroke_circle(center, radius, color, stroke_width);
    }
    
    fn draw_line(&mut self, start: Offset, end: Offset, color: Color, stroke_width: f32) {
        let _ = self.renderer.draw_line(start, end, color, stroke_width);
    }
    
    // ========================================================================
    // Text
    // ========================================================================
    
    fn draw_text(&mut self, text: &str, position: Offset, style: &TextStyle) {
        // Use font key if specified, otherwise use default font
        if let Some(ref font_family) = style.font_family {
            let _ = self.renderer.draw_text_with_font_key(
                text,
                position,
                font_family,
                style.font_size,
                style.color,
            );
        } else {
            let _ = self.renderer.draw_text(
                text,
                position,
                style.font_size,
                style.color,
            );
        }
    }
    
    fn draw_text_centered(&mut self, text: &str, rect: Rect, style: &TextStyle) {
        if let Some(ref font_family) = style.font_family {
            let _ = self.renderer.draw_text_centered_with_font_key(
                text,
                rect,
                font_family,
                style.font_size,
                style.color,
            );
        } else {
            let _ = self.renderer.draw_text_centered(
                text,
                rect,
                style.font_size,
                style.color,
            );
        }
    }
    
    // ========================================================================
    // Images
    // ========================================================================
    
    fn draw_image(&mut self, image_key: &str, position: Offset) {
        let _ = self.renderer.draw_image_by_key(image_key, position);
    }
    
    fn draw_image_with_alpha(&mut self, image_key: &str, position: Offset, alpha: f32) {
        let _ = self.renderer.draw_image_by_key_with_alpha(image_key, position, alpha);
    }
    
    fn draw_image_rect(&mut self, image_key: &str, dest_rect: Rect) {
        let _ = self.renderer.draw_image_by_key_rect(image_key, dest_rect);
    }
    
    fn draw_image_rect_src(&mut self, image_key: &str, src_rect: Rect, dest_rect: Rect) {
        let _ = self.renderer.draw_image_by_key_rect_src(image_key, src_rect, dest_rect);
    }
    
    fn draw_nine_patch(&mut self, image_key: &str, dest_rect: Rect, _insets: EdgeInsets) {
        // Nine-patch implementation
        // This draws the image in 9 sections
        // For now, we just draw the full image scaled
        // TODO: Implement proper nine-patch drawing
        let _ = self.renderer.draw_image_by_key_rect(image_key, dest_rect);
    }
    
    fn get_image_size(&self, image_key: &str) -> Option<Size> {
        self.renderer
            .get_image_size(image_key)
            .map(|(w, h)| Size::new(w as f32, h as f32))
    }
    
    // ========================================================================
    // Measurement
    // ========================================================================
    
    fn measure_text(&self, text: &str, style: &TextStyle) -> Size {
        if let Some(ref font_family) = style.font_family {
            if let Ok(size) = self.renderer.measure_text_with_font_key(text, font_family, style.font_size) {
                return size;
            }
        }
        
        self.renderer.measure_text(text, style.font_size)
    }
    
    fn line_height(&self, style: &TextStyle) -> f32 {
        // Approximate line height as 1.2x font size
        // TODO: Get accurate line height from font metrics
        style.line_height.unwrap_or(style.font_size * 1.2)
    }
    
    fn measure_char_positions(&self, text: &str, style: &TextStyle) -> Vec<f32> {
        // Calculate character positions
        // This is a simple implementation that assumes monospace-like behavior
        // TODO: Implement proper character position measurement
        let mut positions = Vec::with_capacity(text.chars().count() + 1);
        let mut current_x = 0.0;
        positions.push(current_x);
        
        for ch in text.chars() {
            let ch_str = ch.to_string();
            let size = self.measure_text(&ch_str, style);
            current_x += size.width;
            positions.push(current_x);
        }
        
        positions
    }
    
    // ========================================================================
    // Utility
    // ========================================================================
    
    fn canvas_size(&self) -> Size {
        self.renderer.size()
    }
}
