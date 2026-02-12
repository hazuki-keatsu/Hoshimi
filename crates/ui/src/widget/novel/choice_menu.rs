//! Choice Menu Widget
//!
//! Widget for displaying visual novel choice menus.

use std::any::{Any, TypeId};

use hoshimi_types::{
    Alignment, BorderRadius, BoxDecoration, Color, Constraints, EdgeInsets, Offset, Rect, Size,
    TextStyle,
};

use crate::events::{EventResult, HitTestResult, InputEvent, UIMessage};
use crate::key::WidgetKey;
use crate::painter::Painter;
use crate::render::{RenderObject, RenderObjectState};
use crate::widget::Widget;
use crate::impl_render_object_common;

/// A single choice option
#[derive(Debug, Clone)]
pub struct ChoiceOption {
    /// Option identifier
    pub id: String,
    
    /// Display text
    pub text: String,
    
    /// Whether this option is enabled
    pub enabled: bool,
    
    /// Optional condition hint text
    pub hint: Option<String>,
}

impl ChoiceOption {
    /// Create a new choice option
    pub fn new(id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            enabled: true,
            hint: None,
        }
    }
    
    /// Set enabled state
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    
    /// Set hint text
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }
}

/// Choice menu layout style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChoiceLayout {
    /// Vertical list
    #[default]
    Vertical,
    /// Horizontal row
    Horizontal,
    /// Grid layout
    Grid {
        /// The number of columns
        columns: u32,
    },
}

/// Choice menu widget
#[derive(Debug)]
pub struct ChoiceMenu {
    /// Choice options
    pub options: Vec<ChoiceOption>,
    
    /// Menu title (optional)
    pub title: Option<String>,
    
    /// Layout style
    pub layout: ChoiceLayout,
    
    /// Text style for options
    pub text_style: TextStyle,
    
    /// Text style for disabled options
    pub disabled_style: Option<TextStyle>,
    
    /// Text style for title
    pub title_style: Option<TextStyle>,
    
    /// Option button decoration
    pub button_decoration: Option<BoxDecoration>,
    
    /// Hovered button decoration
    pub hover_decoration: Option<BoxDecoration>,
    
    /// Selected button decoration
    pub selected_decoration: Option<BoxDecoration>,
    
    /// Button padding
    pub button_padding: EdgeInsets,
    
    /// Spacing between options
    pub spacing: f32,
    
    /// Menu alignment
    pub alignment: Alignment,
    
    /// Menu margins
    pub margin: EdgeInsets,
    
    /// Optional widget key
    pub key: Option<WidgetKey>,
}

impl ChoiceMenu {
    /// Create a new choice menu
    pub fn new(options: Vec<ChoiceOption>) -> Self {
        Self {
            options,
            title: None,
            layout: ChoiceLayout::Vertical,
            text_style: TextStyle {
                font_size: 20.0,
                color: Color::WHITE,
                ..Default::default()
            },
            disabled_style: Some(TextStyle {
                font_size: 20.0,
                color: Color::from_rgba8(128, 128, 128, 255),
                ..Default::default()
            }),
            title_style: None,
            button_decoration: Some(BoxDecoration {
                color: Some(Color::from_rgba8(40, 40, 60, 220)),
                border_radius: Some(BorderRadius::all(8.0)),
                ..Default::default()
            }),
            hover_decoration: Some(BoxDecoration {
                color: Some(Color::from_rgba8(60, 60, 100, 240)),
                border_radius: Some(BorderRadius::all(8.0)),
                ..Default::default()
            }),
            selected_decoration: Some(BoxDecoration {
                color: Some(Color::from_rgba8(80, 80, 140, 255)),
                border_radius: Some(BorderRadius::all(8.0)),
                ..Default::default()
            }),
            button_padding: EdgeInsets::symmetric(20.0, 12.0),
            spacing: 12.0,
            alignment: Alignment::CENTER,
            margin: EdgeInsets::all(40.0),
            key: None,
        }
    }
    
    /// Set title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }
    
    /// Set layout
    pub fn with_layout(mut self, layout: ChoiceLayout) -> Self {
        self.layout = layout;
        self
    }
    
    /// Set text style
    pub fn with_text_style(mut self, style: TextStyle) -> Self {
        self.text_style = style;
        self
    }
    
    /// Set button decoration
    pub fn with_button_decoration(mut self, decoration: BoxDecoration) -> Self {
        self.button_decoration = Some(decoration);
        self
    }
    
    /// Set hover decoration
    pub fn with_hover_decoration(mut self, decoration: BoxDecoration) -> Self {
        self.hover_decoration = Some(decoration);
        self
    }
    
    /// Set spacing
    pub fn with_spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }
    
    /// Set alignment
    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }
    
    /// Set the widget key
    pub fn with_key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.key = Some(key.into());
        self
    }
}

impl Widget for ChoiceMenu {
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
        Box::new(ChoiceMenuRenderObject::new(
            self.options.clone(),
            self.title.clone(),
            self.layout,
            self.text_style.clone(),
            self.disabled_style.clone(),
            self.title_style.clone(),
            self.button_decoration.clone(),
            self.hover_decoration.clone(),
            self.selected_decoration.clone(),
            self.button_padding,
            self.spacing,
            self.alignment,
            self.margin,
        ))
    }
    
    fn update_render_object(&self, render_object: &mut dyn RenderObject) {
        if let Some(menu_ro) = render_object.as_any_mut().downcast_mut::<ChoiceMenuRenderObject>() {
            menu_ro.options = self.options.clone();
            menu_ro.title = self.title.clone();
            menu_ro.layout = self.layout;
            menu_ro.text_style = self.text_style.clone();
            menu_ro.disabled_style = self.disabled_style.clone();
            menu_ro.title_style = self.title_style.clone();
            menu_ro.button_decoration = self.button_decoration.clone();
            menu_ro.hover_decoration = self.hover_decoration.clone();
            menu_ro.selected_decoration = self.selected_decoration.clone();
            menu_ro.button_padding = self.button_padding;
            menu_ro.spacing = self.spacing;
            menu_ro.alignment = self.alignment;
            menu_ro.margin = self.margin;
            menu_ro.state.mark_needs_layout();
        }
    }
    
    fn should_update(&self, old: &dyn Widget) -> bool {
        if let Some(old_menu) = old.as_any().downcast_ref::<ChoiceMenu>() {
            self.options.len() != old_menu.options.len() ||
            self.title != old_menu.title ||
            self.layout != old_menu.layout
        } else {
            true
        }
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_boxed(&self) -> Box<dyn Widget> {
        Box::new(ChoiceMenu {
            options: self.options.clone(),
            title: self.title.clone(),
            layout: self.layout,
            text_style: self.text_style.clone(),
            disabled_style: self.disabled_style.clone(),
            title_style: self.title_style.clone(),
            button_decoration: self.button_decoration.clone(),
            hover_decoration: self.hover_decoration.clone(),
            selected_decoration: self.selected_decoration.clone(),
            button_padding: self.button_padding,
            spacing: self.spacing,
            alignment: self.alignment,
            margin: self.margin,
            key: self.key.clone(),
        })
    }
}

/// Render object for ChoiceMenu widget
#[derive(Debug)]
pub struct ChoiceMenuRenderObject {
    state: RenderObjectState,
    options: Vec<ChoiceOption>,
    title: Option<String>,
    layout: ChoiceLayout,
    text_style: TextStyle,
    disabled_style: Option<TextStyle>,
    title_style: Option<TextStyle>,
    button_decoration: Option<BoxDecoration>,
    hover_decoration: Option<BoxDecoration>,
    selected_decoration: Option<BoxDecoration>,
    button_padding: EdgeInsets,
    spacing: f32,
    alignment: Alignment,
    margin: EdgeInsets,
    
    // Interaction state
    hovered_index: Option<usize>,
    selected_index: Option<usize>,
    
    // Layout cache
    button_rects: Vec<Rect>,
    title_rect: Option<Rect>,
    menu_rect: Rect,
}

impl ChoiceMenuRenderObject {
    #[allow(clippy::too_many_arguments)]
    fn new(
        options: Vec<ChoiceOption>,
        title: Option<String>,
        layout: ChoiceLayout,
        text_style: TextStyle,
        disabled_style: Option<TextStyle>,
        title_style: Option<TextStyle>,
        button_decoration: Option<BoxDecoration>,
        hover_decoration: Option<BoxDecoration>,
        selected_decoration: Option<BoxDecoration>,
        button_padding: EdgeInsets,
        spacing: f32,
        alignment: Alignment,
        margin: EdgeInsets,
    ) -> Self {
        Self {
            state: RenderObjectState::new(),
            options,
            title,
            layout,
            text_style,
            disabled_style,
            title_style,
            button_decoration,
            hover_decoration,
            selected_decoration,
            button_padding,
            spacing,
            alignment,
            margin,
            hovered_index: None,
            selected_index: None,
            button_rects: Vec::new(),
            title_rect: None,
            menu_rect: Rect::ZERO,
        }
    }
    
    /// Get the selected option ID (if any)
    pub fn selected_option_id(&self) -> Option<&str> {
        self.selected_index.and_then(|i| self.options.get(i).map(|o| o.id.as_str()))
    }
}

impl RenderObject for ChoiceMenuRenderObject {
    impl_render_object_common!(state);
    
    fn layout(&mut self, constraints: Constraints) -> Size {
        let parent_size = Size::new(constraints.max_width, constraints.max_height);
        
        // Calculate available area
        let available_width = parent_size.width - self.margin.left - self.margin.right;
        let available_height = parent_size.height - self.margin.top - self.margin.bottom;
        
        // Estimate button sizes
        let button_height = self.text_style.font_size + self.button_padding.top + self.button_padding.bottom;
        let button_width = available_width.min(400.0); // Max button width
        
        // Calculate total menu size
        let (menu_width, menu_height) = match self.layout {
            ChoiceLayout::Vertical => {
                let height = button_height * self.options.len() as f32 +
                    self.spacing * (self.options.len().saturating_sub(1)) as f32;
                (button_width, height)
            }
            ChoiceLayout::Horizontal => {
                let width = button_width * self.options.len() as f32 +
                    self.spacing * (self.options.len().saturating_sub(1)) as f32;
                (width.min(available_width), button_height)
            }
            ChoiceLayout::Grid { columns } => {
                let rows = (self.options.len() as f32 / columns as f32).ceil() as usize;
                let width = button_width * columns as f32 +
                    self.spacing * (columns.saturating_sub(1)) as f32;
                let height = button_height * rows as f32 +
                    self.spacing * (rows.saturating_sub(1)) as f32;
                (width.min(available_width), height)
            }
        };
        
        // Add title height if present
        let title_height = if self.title.is_some() {
            self.title_style.as_ref()
                .map(|s| s.font_size)
                .unwrap_or(self.text_style.font_size + 4.0) + self.spacing
        } else {
            0.0
        };
        
        let total_height = menu_height + title_height;
        
        // Calculate menu position
        let menu_offset = self.alignment.align(
            Size::new(menu_width, total_height),
            Size::new(available_width, available_height),
        );
        
        let menu_x = self.margin.left + menu_offset.x;
        let menu_y = self.margin.top + menu_offset.y;
        
        self.menu_rect = Rect::new(menu_x, menu_y, menu_width, total_height);
        
        // Layout title
        let mut current_y = menu_y;
        if self.title.is_some() {
            self.title_rect = Some(Rect::new(
                menu_x,
                current_y,
                menu_width,
                title_height - self.spacing,
            ));
            current_y += title_height;
        } else {
            self.title_rect = None;
        }
        
        // Layout buttons
        self.button_rects.clear();
        
        match self.layout {
            ChoiceLayout::Vertical => {
                for i in 0..self.options.len() {
                    let rect = Rect::new(
                        menu_x,
                        current_y + (button_height + self.spacing) * i as f32,
                        button_width,
                        button_height,
                    );
                    self.button_rects.push(rect);
                }
            }
            ChoiceLayout::Horizontal => {
                let individual_width = (menu_width - self.spacing * (self.options.len().saturating_sub(1)) as f32)
                    / self.options.len() as f32;
                for i in 0..self.options.len() {
                    let rect = Rect::new(
                        menu_x + (individual_width + self.spacing) * i as f32,
                        current_y,
                        individual_width,
                        button_height,
                    );
                    self.button_rects.push(rect);
                }
            }
            ChoiceLayout::Grid { columns } => {
                let col_width = (menu_width - self.spacing * (columns.saturating_sub(1)) as f32)
                    / columns as f32;
                for i in 0..self.options.len() {
                    let col = i % columns as usize;
                    let row = i / columns as usize;
                    let rect = Rect::new(
                        menu_x + (col_width + self.spacing) * col as f32,
                        current_y + (button_height + self.spacing) * row as f32,
                        col_width,
                        button_height,
                    );
                    self.button_rects.push(rect);
                }
            }
        }
        
        self.state.size = parent_size;
        self.state.needs_layout = false;
        
        parent_size
    }
    
    fn paint(&self, painter: &mut dyn Painter) {
        painter.save();
        painter.translate(self.state.offset);
        
        // Draw title
        if let (Some(title), Some(title_rect)) = (&self.title, self.title_rect) {
            let style = self.title_style.as_ref().unwrap_or(&self.text_style);
            painter.draw_text(
                title,
                Offset::new(
                    title_rect.x + (title_rect.width - style.font_size * title.len() as f32 * 0.6) / 2.0,
                    title_rect.y,
                ),
                style,
            );
        }
        
        // Draw buttons
        for (i, (option, rect)) in self.options.iter().zip(self.button_rects.iter()).enumerate() {
            // Determine decoration based on state
            let decoration = if Some(i) == self.selected_index {
                self.selected_decoration.as_ref().or(self.button_decoration.as_ref())
            } else if Some(i) == self.hovered_index && option.enabled {
                self.hover_decoration.as_ref().or(self.button_decoration.as_ref())
            } else {
                self.button_decoration.as_ref()
            };
            
            // Draw button background
            if let Some(dec) = decoration {
                if let Some(color) = dec.color {
                    let alpha = if option.enabled { color.a } else { color.a * 0.5 };
                    painter.draw_rounded_rect(
                        *rect,
                        dec.border_radius.unwrap_or(BorderRadius::ZERO),
                        color.with_alpha(alpha),
                    );
                }
            }
            
            // Draw button text
            let style = if option.enabled {
                &self.text_style
            } else {
                self.disabled_style.as_ref().unwrap_or(&self.text_style)
            };
            
            let text_x = rect.x + self.button_padding.left;
            let text_y = rect.y + self.button_padding.top;
            
            painter.draw_text(
                &option.text,
                Offset::new(text_x, text_y),
                style,
            );
            
            // Draw hint if present
            if let Some(ref hint) = option.hint {
                let hint_style = TextStyle {
                    font_size: style.font_size * 0.7,
                    color: style.color.with_alpha(style.color.a * 0.5),
                    ..style.clone()
                };
                painter.draw_text(
                    hint,
                    Offset::new(
                        rect.x + rect.width - self.button_padding.right - hint.len() as f32 * hint_style.font_size * 0.6,
                        text_y,
                    ),
                    &hint_style,
                );
            }
        }
        
        painter.restore();
    }
    
    fn hit_test(&self, position: Offset) -> HitTestResult {
        for (i, rect) in self.button_rects.iter().enumerate() {
            if rect.contains(position) && self.options.get(i).is_some_and(|o| o.enabled) {
                return HitTestResult::Hit;
            }
        }
        
        if self.menu_rect.contains(position) {
            HitTestResult::HitTransparent
        } else {
            HitTestResult::Miss
        }
    }
    
    fn handle_event(&mut self, event: &InputEvent) -> EventResult {
        match event {
            InputEvent::Hover { position, .. } => {
                let old_hovered = self.hovered_index;
                self.hovered_index = None;
                
                for (i, rect) in self.button_rects.iter().enumerate() {
                    if rect.contains(*position) && self.options.get(i).is_some_and(|o| o.enabled) {
                        self.hovered_index = Some(i);
                        break;
                    }
                }
                
                if self.hovered_index != old_hovered {
                    self.state.mark_needs_paint();
                }
                
                if self.hovered_index.is_some() {
                    EventResult::Consumed
                } else {
                    EventResult::Ignored
                }
            }
            
            InputEvent::Tap { position } | InputEvent::MouseDown { position, .. } => {
                for (i, rect) in self.button_rects.iter().enumerate() {
                    if rect.contains(*position) {
                        if let Some(option) = self.options.get(i) {
                            if option.enabled {
                                self.selected_index = Some(i);
                                self.state.mark_needs_paint();
                                
                                return EventResult::Message(UIMessage::OptionSelect {
                                    index: i,
                                    label: Some(option.text.clone()),
                                });
                            }
                        }
                        return EventResult::Consumed;
                    }
                }
                
                EventResult::Ignored
            }
            
            _ => EventResult::Ignored,
        }
    }
}
