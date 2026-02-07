//! Dialog Box Widget
//!
//! Widget for displaying visual novel dialog text with character names.

use std::any::{Any, TypeId};

use hoshimi_shared::{
    Alignment, BorderRadius, BoxDecoration, Color, Constraints, EdgeInsets, Offset, Rect, Size,
    TextStyle,
};

use crate::events::{EventResult, HitTestResult, InputEvent, UIMessage};
use crate::key::WidgetKey;
use crate::painter::Painter;
use crate::render::{RenderObject, RenderObjectState};
use crate::widget::Widget;
use crate::impl_render_object_common;

/// Dialog display style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DialogStyle {
    /// Standard ADV-style dialog box at bottom
    #[default]
    Adv,
    /// NVL-style full screen text
    Nvl,
    /// Message box style (centered)
    MessageBox,
}

/// Text reveal animation
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TextReveal {
    /// Instant reveal
    #[default]
    Instant,
    /// Character by character
    Typewriter {
        /// Characters per second
        chars_per_second: f32,
    },
    /// Fade in by character
    Fade {
        /// Characters per second
        chars_per_second: f32,
        /// Fade duration per character
        fade_duration: f32,
    },
}

/// Dialog box widget
#[derive(Debug)]
pub struct DialogBox {
    /// Character name (if any)
    pub speaker: Option<String>,
    
    /// Dialog text content
    pub text: String,
    
    /// Dialog style
    pub style: DialogStyle,
    
    /// Text reveal animation
    pub text_reveal: TextReveal,
    
    /// Text style
    pub text_style: TextStyle,
    
    /// Speaker name style
    pub speaker_style: Option<TextStyle>,
    
    /// Box decoration
    pub decoration: Option<BoxDecoration>,
    
    /// Content padding
    pub padding: EdgeInsets,
    
    /// Box position alignment
    pub alignment: Alignment,
    
    /// Box margins
    pub margin: EdgeInsets,
    
    /// Show continue indicator
    pub show_continue: bool,
    
    /// Optional widget key
    pub key: Option<WidgetKey>,
}

impl DialogBox {
    /// Create a new dialog box
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            speaker: None,
            text: text.into(),
            style: DialogStyle::Adv,
            text_reveal: TextReveal::Instant,
            text_style: TextStyle {
                font_size: 24.0,
                color: Color::WHITE,
                ..Default::default()
            },
            speaker_style: None,
            decoration: Some(BoxDecoration {
                color: Some(Color::from_rgba8(0, 0, 0, 200)),
                border_radius: Some(BorderRadius::all(8.0)),
                ..Default::default()
            }),
            padding: EdgeInsets::all(20.0),
            alignment: Alignment::BOTTOM_CENTER,
            margin: EdgeInsets::new(20.0, 40.0, 20.0, 40.0),
            show_continue: true,
            key: None,
        }
    }
    
    /// Set speaker name
    pub fn with_speaker(mut self, speaker: impl Into<String>) -> Self {
        self.speaker = Some(speaker.into());
        self
    }
    
    /// Set dialog style
    pub fn with_style(mut self, style: DialogStyle) -> Self {
        self.style = style;
        self
    }
    
    /// Set text reveal animation
    pub fn with_text_reveal(mut self, reveal: TextReveal) -> Self {
        self.text_reveal = reveal;
        self
    }
    
    /// Set text style
    pub fn with_text_style(mut self, style: TextStyle) -> Self {
        self.text_style = style;
        self
    }
    
    /// Set speaker name style
    pub fn with_speaker_style(mut self, style: TextStyle) -> Self {
        self.speaker_style = Some(style);
        self
    }
    
    /// Set decoration
    pub fn with_decoration(mut self, decoration: BoxDecoration) -> Self {
        self.decoration = Some(decoration);
        self
    }
    
    /// Set padding
    pub fn with_padding(mut self, padding: EdgeInsets) -> Self {
        self.padding = padding;
        self
    }
    
    /// Set alignment
    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }
    
    /// Set margin
    pub fn with_margin(mut self, margin: EdgeInsets) -> Self {
        self.margin = margin;
        self
    }
    
    /// Set continue indicator visibility
    pub fn with_continue_indicator(mut self, show: bool) -> Self {
        self.show_continue = show;
        self
    }
    
    /// Set the widget key
    pub fn with_key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.key = Some(key.into());
        self
    }
}

impl Widget for DialogBox {
    fn widget_type(&self) -> TypeId {
        TypeId::of::<Self>()
    }
    
    fn key(&self) -> Option<WidgetKey> {
        self.key.clone()
    }
    
    fn children(&self) -> Vec<&dyn Widget> {
        vec![]
    }
    
    fn create_render_object(&self) -> Box<dyn RenderObject> {
        Box::new(DialogBoxRenderObject::new(
            self.speaker.clone(),
            self.text.clone(),
            self.style,
            self.text_reveal,
            self.text_style.clone(),
            self.speaker_style.clone(),
            self.decoration.clone(),
            self.padding,
            self.alignment,
            self.margin,
            self.show_continue,
        ))
    }
    
    fn update_render_object(&self, render_object: &mut dyn RenderObject) {
        if let Some(dialog_ro) = render_object.as_any_mut().downcast_mut::<DialogBoxRenderObject>() {
            // Check if text changed to reset reveal animation
            if dialog_ro.text != self.text {
                dialog_ro.text = self.text.clone();
                dialog_ro.revealed_chars = 0;
                dialog_ro.char_reveal_timer = 0.0;
            }
            
            dialog_ro.speaker = self.speaker.clone();
            dialog_ro.style = self.style;
            dialog_ro.text_reveal = self.text_reveal;
            dialog_ro.text_style = self.text_style.clone();
            dialog_ro.speaker_style = self.speaker_style.clone();
            dialog_ro.decoration = self.decoration.clone();
            dialog_ro.padding = self.padding;
            dialog_ro.alignment = self.alignment;
            dialog_ro.margin = self.margin;
            dialog_ro.show_continue = self.show_continue;
            dialog_ro.state.mark_needs_layout();
        }
    }
    
    fn should_update(&self, old: &dyn Widget) -> bool {
        if let Some(old_dialog) = old.as_any().downcast_ref::<DialogBox>() {
            self.speaker != old_dialog.speaker ||
            self.text != old_dialog.text ||
            self.style != old_dialog.style ||
            self.text_reveal != old_dialog.text_reveal ||
            self.show_continue != old_dialog.show_continue
        } else {
            true
        }
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_boxed(&self) -> Box<dyn Widget> {
        Box::new(DialogBox {
            speaker: self.speaker.clone(),
            text: self.text.clone(),
            style: self.style,
            text_reveal: self.text_reveal,
            text_style: self.text_style.clone(),
            speaker_style: self.speaker_style.clone(),
            decoration: self.decoration.clone(),
            padding: self.padding,
            alignment: self.alignment,
            margin: self.margin,
            show_continue: self.show_continue,
            key: self.key.clone(),
        })
    }
}

/// Render object for DialogBox widget
#[derive(Debug)]
pub struct DialogBoxRenderObject {
    state: RenderObjectState,
    speaker: Option<String>,
    text: String,
    style: DialogStyle,
    text_reveal: TextReveal,
    text_style: TextStyle,
    speaker_style: Option<TextStyle>,
    decoration: Option<BoxDecoration>,
    padding: EdgeInsets,
    alignment: Alignment,
    margin: EdgeInsets,
    show_continue: bool,
    
    // Animation state
    revealed_chars: usize,
    char_reveal_timer: f32,
    continue_blink_timer: f32,
    
    // Layout cache
    box_rect: Rect,
    text_rect: Rect,
    speaker_rect: Option<Rect>,
}

impl DialogBoxRenderObject {
    fn new(
        speaker: Option<String>,
        text: String,
        style: DialogStyle,
        text_reveal: TextReveal,
        text_style: TextStyle,
        speaker_style: Option<TextStyle>,
        decoration: Option<BoxDecoration>,
        padding: EdgeInsets,
        alignment: Alignment,
        margin: EdgeInsets,
        show_continue: bool,
    ) -> Self {
        let initial_chars = match text_reveal {
            TextReveal::Instant => text.chars().count(),
            _ => 0,
        };
        
        Self {
            state: RenderObjectState::new(),
            speaker,
            text,
            style,
            text_reveal,
            text_style,
            speaker_style,
            decoration,
            padding,
            alignment,
            margin,
            show_continue,
            revealed_chars: initial_chars,
            char_reveal_timer: 0.0,
            continue_blink_timer: 0.0,
            box_rect: Rect::ZERO,
            text_rect: Rect::ZERO,
            speaker_rect: None,
        }
    }
    
    /// Update text reveal animation
    pub fn update_animation(&mut self, delta_time: f32) -> bool {
        let mut needs_update = false;
        
        // Update text reveal
        let total_chars = self.text.chars().count();
        if self.revealed_chars < total_chars {
            let chars_per_second = match self.text_reveal {
                TextReveal::Typewriter { chars_per_second } => chars_per_second,
                TextReveal::Fade { chars_per_second, .. } => chars_per_second,
                TextReveal::Instant => {
                    self.revealed_chars = total_chars;
                    return false;
                }
            };
            
            self.char_reveal_timer += delta_time;
            let chars_to_reveal = (self.char_reveal_timer * chars_per_second) as usize;
            if chars_to_reveal > self.revealed_chars {
                self.revealed_chars = chars_to_reveal.min(total_chars);
                self.state.mark_needs_paint();
                needs_update = true;
            }
        }
        
        // Update continue indicator blink
        if self.show_continue && self.is_fully_revealed() {
            self.continue_blink_timer += delta_time;
            if self.continue_blink_timer >= 0.5 {
                self.continue_blink_timer = 0.0;
                self.state.mark_needs_paint();
                needs_update = true;
            }
        }
        
        needs_update
    }
    
    /// Check if text is fully revealed
    pub fn is_fully_revealed(&self) -> bool {
        self.revealed_chars >= self.text.chars().count()
    }
    
    /// Skip to fully revealed
    pub fn skip_reveal(&mut self) {
        self.revealed_chars = self.text.chars().count();
        self.state.mark_needs_paint();
    }
    
    /// Get revealed text
    pub fn revealed_text(&self) -> String {
        self.text.chars().take(self.revealed_chars).collect()
    }
}

impl RenderObject for DialogBoxRenderObject {
    impl_render_object_common!(state);
    
    fn layout(&mut self, constraints: Constraints) -> Size {
        let parent_size = Size::new(constraints.max_width, constraints.max_height);
        
        // Calculate available size after margins
        let available_width = parent_size.width - self.margin.left - self.margin.right;
        let available_height = parent_size.height - self.margin.top - self.margin.bottom;
        
        // Calculate box size based on style
        let (box_width, box_height) = match self.style {
            DialogStyle::Adv => {
                (available_width, available_height.min(200.0))
            }
            DialogStyle::Nvl => {
                (available_width * 0.8, available_height * 0.8)
            }
            DialogStyle::MessageBox => {
                (available_width * 0.6, available_height * 0.3)
            }
        };
        
        // Calculate box position based on alignment
        let box_offset = self.alignment.align(
            Size::new(box_width, box_height),
            Size::new(available_width, available_height),
        );
        
        self.box_rect = Rect::new(
            self.margin.left + box_offset.x,
            self.margin.top + box_offset.y,
            box_width,
            box_height,
        );
        
        // Calculate text area
        let content_x = self.box_rect.x + self.padding.left;
        let content_y = self.box_rect.y + self.padding.top;
        let content_width = self.box_rect.width - self.padding.left - self.padding.right;
        let content_height = self.box_rect.height - self.padding.top - self.padding.bottom;
        
        // Speaker name area
        if self.speaker.is_some() {
            let speaker_height = self.speaker_style
                .as_ref()
                .map(|s| s.font_size)
                .unwrap_or(self.text_style.font_size) + 8.0;
            
            self.speaker_rect = Some(Rect::new(
                content_x,
                content_y,
                content_width,
                speaker_height,
            ));
            
            self.text_rect = Rect::new(
                content_x,
                content_y + speaker_height,
                content_width,
                content_height - speaker_height,
            );
        } else {
            self.speaker_rect = None;
            self.text_rect = Rect::new(content_x, content_y, content_width, content_height);
        }
        
        self.state.size = parent_size;
        self.state.needs_layout = false;
        
        parent_size
    }
    
    fn paint(&self, painter: &mut dyn Painter) {
        painter.save();
        painter.translate(self.state.offset);
        
        // Draw box decoration
        if let Some(ref decoration) = self.decoration {
            if let Some(color) = decoration.color {
                painter.draw_rounded_rect(
                    self.box_rect,
                    decoration.border_radius.unwrap_or(BorderRadius::ZERO),
                    color,
                );
            }
            
            // Draw border if present
            if let Some(ref border) = decoration.border {
                painter.stroke_rounded_rect(
                    self.box_rect,
                    decoration.border_radius.unwrap_or(BorderRadius::ZERO),
                    border.color,
                    border.width,
                );
            }
        }
        
        // Draw speaker name
        if let (Some(speaker), Some(speaker_rect)) = (&self.speaker, self.speaker_rect) {
            let style = self.speaker_style.as_ref().unwrap_or(&self.text_style);
            painter.draw_text(
                speaker,
                Offset::new(speaker_rect.x, speaker_rect.y),
                style,
            );
        }
        
        // Draw revealed text
        let revealed = self.revealed_text();
        if !revealed.is_empty() {
            painter.draw_text(
                &revealed,
                Offset::new(self.text_rect.x, self.text_rect.y),
                &self.text_style,
            );
        }
        
        // Draw continue indicator
        if self.show_continue && self.is_fully_revealed() {
            let blink_visible = (self.continue_blink_timer * 2.0) as i32 % 2 == 0;
            if blink_visible {
                let indicator_size = 12.0;
                let indicator_x = self.box_rect.x + self.box_rect.width - self.padding.right - indicator_size;
                let indicator_y = self.box_rect.y + self.box_rect.height - self.padding.bottom - indicator_size;
                
                // Draw a simple triangle indicator
                let indicator_color = self.text_style.color;
                painter.fill_rect(
                    Rect::new(indicator_x, indicator_y, indicator_size, indicator_size),
                    indicator_color,
                );
            }
        }
        
        painter.restore();
    }
    
    fn hit_test(&self, position: Offset) -> HitTestResult {
        if self.box_rect.contains(position) {
            HitTestResult::Hit
        } else {
            HitTestResult::Miss
        }
    }
    
    fn handle_event(&mut self, event: &InputEvent) -> EventResult {
        match event {
            InputEvent::Tap { position } | InputEvent::MouseDown { position, .. } => {
                if self.box_rect.contains(*position) {
                    if !self.is_fully_revealed() {
                        // Skip text reveal
                        self.skip_reveal();
                        EventResult::Consumed
                    } else {
                        // Confirm dialog
                        EventResult::Message(UIMessage::DialogConfirm)
                    }
                } else {
                    EventResult::Ignored
                }
            }
            _ => EventResult::Ignored,
        }
    }
}