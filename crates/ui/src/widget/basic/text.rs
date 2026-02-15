//! Text Widget
//!
//! Displays text with configurable styling.

use std::any::{Any, TypeId};

use hoshimi_types::{Constraints, Offset, Rect, Size, TextAlign, TextOverflow, TextStyle};


use crate::key::WidgetKey;
use crate::painter::{Painter, TextMeasurer};
use crate::render_object::{
    EventHandlable, Layoutable, Lifecycle, Paintable, Parent, RenderObject, RenderObjectState,
};
use crate::widget::Widget;

/// Text widget for displaying styled text
#[derive(Debug, Clone)]
pub struct Text {
    /// The text content to display
    pub content: String,
    
    /// Text styling
    pub style: TextStyle,
    
    /// Text alignment
    pub align: TextAlign,
    
    /// Maximum number of lines (None for unlimited)
    pub max_lines: Option<u32>,
    
    /// Overflow behavior
    pub overflow: TextOverflow,
    
    /// Optional widget key
    pub key: Option<WidgetKey>,
}

impl Text {
    /// Create a new text widget
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            style: TextStyle::default(),
            align: TextAlign::Center,
            max_lines: None,
            overflow: TextOverflow::Clip,
            key: None,
        }
    }
    
    /// Set the text style
    pub fn with_style(mut self, style: TextStyle) -> Self {
        self.style = style;
        self
    }
    
    /// Set the font size
    pub fn with_size(mut self, size: f32) -> Self {
        self.style.font_size = size;
        self
    }
    
    /// Set the text color
    pub fn with_color(mut self, color: hoshimi_types::Color) -> Self {
        self.style.color = color;
        self
    }
    
    /// Set text alignment
    pub fn with_align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }
    
    /// Set maximum lines
    pub fn with_max_lines(mut self, max_lines: u32) -> Self {
        self.max_lines = Some(max_lines);
        self
    }
    
    /// Set overflow behavior
    pub fn with_overflow(mut self, overflow: TextOverflow) -> Self {
        self.overflow = overflow;
        self
    }
    
    /// Set the widget key
    pub fn with_key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.key = Some(key.into());
        self
    }
}

impl Widget for Text {
    fn widget_type(&self) -> TypeId {
        TypeId::of::<Self>()
    }
    
    fn key(&self) -> Option<WidgetKey> {
        self.key.clone()
    }
    
    fn create_render_object(&self) -> Box<dyn RenderObject> {
        Box::new(TextRenderObject::new(
            self.content.clone(),
            self.style.clone(),
            self.align,
            self.max_lines,
            self.overflow,
        ))
    }
    
    fn update_render_object(&self, render_object: &mut dyn RenderObject) {
        if let Some(text_ro) = render_object.as_any_mut().downcast_mut::<TextRenderObject>() {
            text_ro.content = self.content.clone();
            text_ro.style = self.style.clone();
            text_ro.align = self.align;
            text_ro.max_lines = self.max_lines;
            text_ro.overflow = self.overflow;
            text_ro.state.mark_needs_layout();
            text_ro.state.mark_needs_paint();
        }
    }
    
    fn should_update(&self, old: &dyn Widget) -> bool {
        if let Some(old_text) = old.as_any().downcast_ref::<Text>() {
            self.content != old_text.content ||
            self.style != old_text.style ||
            self.align != old_text.align ||
            self.max_lines != old_text.max_lines ||
            self.overflow != old_text.overflow
        } else {
            true
        }
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_boxed(&self) -> Box<dyn Widget> {
        Box::new(self.clone())
    }
}

/// Render object for Text widget
#[derive(Debug)]
pub struct TextRenderObject {
    state: RenderObjectState,
    content: String,
    style: TextStyle,
    align: TextAlign,
    max_lines: Option<u32>,
    overflow: TextOverflow,
    
    /// Cached text size
    cached_size: Option<Size>,
}

impl TextRenderObject {
    fn new(
        content: String,
        style: TextStyle,
        align: TextAlign,
        max_lines: Option<u32>,
        overflow: TextOverflow,
    ) -> Self {
        Self {
            state: RenderObjectState::new(),
            content,
            style,
            align,
            max_lines,
            overflow,
            cached_size: None,
        }
    }
    
    /// Truncate text to fit within a given width using binary search
    fn truncate_text_to_width(
        &self,
        text: &str,
        max_width: f32,
        painter: &dyn Painter,
    ) -> String {
        let chars: Vec<char> = text.chars().collect();
        if chars.is_empty() {
            return String::new();
        }
        
        // Binary search for the maximum number of characters that fit
        let mut left = 0;
        let mut right = chars.len();
        
        while left < right {
            let mid = left + (right - left + 1) / 2;
            let substring: String = chars[..mid].iter().collect();
            let size = painter.measure_text(&substring, &self.style);
            
            if size.width <= max_width {
                left = mid;
            } else {
                right = mid - 1;
            }
        }
        
        if left == 0 {
            String::new()
        } else {
            chars[..left].iter().collect()
        }
    }
}

impl Layoutable for TextRenderObject {
    fn layout(&mut self, constraints: Constraints, text_measurer: &dyn TextMeasurer) -> Size {
        // Use actual text measurement instead of approximation
        let text_size = text_measurer.measure_text(&self.content, &self.style);
        let line_height = self.style.line_height.unwrap_or(self.style.font_size * 1.2);

        let size = constraints.constrain(Size::new(text_size.width, line_height));
        self.state.size = size;
        self.state.needs_layout = false;
        self.cached_size = Some(size);

        size
    }

    fn get_rect(&self) -> Rect {
        self.state.get_rect()
    }

    fn set_offset(&mut self, offset: Offset) {
        self.state.offset = offset;
    }

    fn get_offset(&self) -> Offset {
        self.state.offset
    }

    fn get_size(&self) -> Size {
        self.state.size
    }

    fn needs_layout(&self) -> bool {
        self.state.needs_layout
    }

    fn mark_needs_layout(&mut self) {
        self.state.needs_layout = true;
    }
}

impl Paintable for TextRenderObject {
    fn paint(&self, painter: &mut dyn Painter) {
        let rect = self.state.get_rect();
        let text_size = painter.measure_text(&self.content, &self.style);
        let available_width = rect.width;
        
        // Handle overflow based on the overflow mode
        match self.overflow {
            TextOverflow::Clip => {
                // Clip mode: save state, clip to rect, draw text, restore
                if text_size.width <= available_width {
                    painter.draw_text_aligned(&self.content, rect, &self.style, self.align);
                } else {
                    painter.save();
                    painter.clip_rect(rect);
                    painter.draw_text_aligned(&self.content, rect, &self.style, TextAlign::Left);
                    painter.restore();
                }                
            }
            TextOverflow::Ellipsis => {
                // Ellipsis mode: truncate text with "..." if it overflows
                if text_size.width <= available_width {
                    // Text fits, use user-specified alignment
                    painter.draw_text_aligned(&self.content, rect, &self.style, self.align);
                } else {
                    // Text overflows, force left alignment and truncate with ellipsis
                    let ellipsis = "...";
                    let ellipsis_size = painter.measure_text(ellipsis, &self.style);
                    let target_width = available_width - ellipsis_size.width;
                    
                    if target_width > 0.0 {
                        // Binary search for the truncation point
                        let truncated = self.truncate_text_to_width(
                            &self.content,
                            target_width,
                            painter,
                        );
                        let final_text = format!("{}{}", truncated, ellipsis);
                        // Force left alignment for overflow
                        painter.draw_text_aligned(&final_text, rect, &self.style, TextAlign::Left);
                    } else {
                        // Even ellipsis doesn't fit, just draw ellipsis
                        painter.draw_text_aligned(ellipsis, rect, &self.style, TextAlign::Left);
                    }
                }
            }
            TextOverflow::Fade => {
                // Fade mode: text fades out at the right edge when overflowing                
                if text_size.width <= available_width {
                    // Text fits, use user-specified alignment
                    painter.draw_text_aligned(&self.content, rect, &self.style, self.align);
                } else {
                    // Text overflows, force left alignment and apply fade effect
                    // Calculate fade region (last 20% of the container width, or at least 2 characters width)
                    let fade_width = (available_width * 0.2).max(self.style.font_size * 1.5).min(available_width * 0.5);
                    
                    // Use layer approach for proper fade effect:
                    // 1. Save a layer to isolate the text
                    // 2. Draw text in the layer (forced left alignment)
                    // 3. Draw a gradient on top using DstIn to modulate the layer's alpha
                    // 4. Restore the layer
                    
                    painter.save();
                    painter.clip_rect(rect);
                    
                    // Create a layer to isolate the text
                    painter.save_layer_alpha(rect, 1.0);
                    
                    // Draw the text with forced left alignment
                    painter.draw_text_aligned(&self.content, rect, &self.style, TextAlign::Left);
                    
                    // Draw gradient on top to modulate alpha
                    // DstIn: result = dst * src_alpha
                    // We want: left side (src_alpha=1) keeps text, right side (src_alpha=0) fades out
                    let fade_rect = Rect::new(rect.x + available_width - fade_width, rect.y, fade_width, rect.height);
                    painter.apply_gradient_alpha_mask(fade_rect, 1.0, 0.0);
                    
                    // Restore the layer (text with modulated alpha is drawn to screen)
                    painter.restore();
                    
                    painter.restore();
                }
            }
        }
    }
    
    fn needs_paint(&self) -> bool {
        self.state.needs_paint
    }

    fn mark_needs_paint(&mut self) {
        self.state.needs_paint = true;
    }
}

impl EventHandlable for TextRenderObject {}

impl Lifecycle for TextRenderObject {}

impl Parent for TextRenderObject {}

impl RenderObject for TextRenderObject {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Check if a character is a CJK (Chinese, Japanese, Korean) character
/// These characters are typically rendered as full-width/square glyphs
fn is_cjk_char(ch: char) -> bool {
    let code = ch as u32;
    // CJK Unified Ideographs
    (0x4E00..=0x9FFF).contains(&code)
        // CJK Unified Ideographs Extension A
        || (0x3400..=0x4DBF).contains(&code)
        // CJK Unified Ideographs Extension B
        || (0x20000..=0x2A6DF).contains(&code)
        // CJK Compatibility Ideographs
        || (0xF900..=0xFAFF).contains(&code)
        // Hiragana
        || (0x3040..=0x309F).contains(&code)
        // Katakana
        || (0x30A0..=0x30FF).contains(&code)
        // Hangul Syllables
        || (0xAC00..=0xD7AF).contains(&code)
        // Full-width Latin characters
        || (0xFF00..=0xFFEF).contains(&code)
        // CJK Symbols and Punctuation
        || (0x3000..=0x303F).contains(&code)
}
