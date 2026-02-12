//! Text Widget
//!
//! Displays text with configurable styling.

use std::any::{Any, TypeId};

use hoshimi_types::{Constraints, Size, TextAlign, TextOverflow, TextStyle};

use crate::key::WidgetKey;
use crate::painter::Painter;
use crate::render::{RenderObject, RenderObjectState};
use crate::widget::Widget;
use crate::impl_render_object_common;

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
}

impl RenderObject for TextRenderObject {
    impl_render_object_common!(state);
    
    fn layout(&mut self, constraints: Constraints) -> Size {
        // Calculate text width with better heuristics for different character types
        // CJK characters are approximately square (width ≈ font_size)
        // Latin characters are narrower (width ≈ font_size * 0.5)
        let mut approx_width = 0.0;
        for ch in self.content.chars() {
            if is_cjk_char(ch) {
                // CJK characters are roughly square
                approx_width += self.style.font_size;
            } else if ch.is_ascii() {
                // ASCII characters are narrower
                approx_width += self.style.font_size * 0.5;
            } else {
                // Other Unicode characters (emoji, etc.) - assume wider
                approx_width += self.style.font_size * 0.8;
            }
        }
        
        let line_height = self.style.line_height.unwrap_or(self.style.font_size * 1.2);
        
        let size = constraints.constrain(Size::new(approx_width, line_height));
        self.state.size = size;
        self.state.needs_layout = false;
        self.cached_size = Some(size);
        
        size
    }
    
    fn paint(&self, painter: &mut dyn Painter) {
        let rect = self.state.get_rect();
        painter.draw_text_aligned(&self.content, rect, &self.style, self.align);
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
        // Fullwidth Latin characters
        || (0xFF00..=0xFFEF).contains(&code)
        // CJK Symbols and Punctuation
        || (0x3000..=0x303F).contains(&code)
}
