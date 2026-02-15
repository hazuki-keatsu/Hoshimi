//! Button Widget
//!
//! A comprehensive button buttons.
//!
//! # Button Types
//!
//! - `ElevatedButton`: A button with elevation (shadow)
//! - `OutlinedButton`: A button with a border outline
//! - `TextButton`: A button with no border or elevation
//!
//! # Button States
//!
//! Buttons support the following states:
//! - `enabled`: Default state
//! - `disabled`: Button is not interactive
//! - `hovered`: Mouse cursor is over the button
//! - `pressed`: Button is being pressed
//! - `focused`: Button has keyboard focus
//!
//! # Example
//!
//! ```ignore
//! ElevatedButton::with_child(Text::new("Click Me"))
//!     .on_press("my_button")
//!     .style(ButtonStyle::default())
//! ```

use std::any::{Any, TypeId};

use hoshimi_types::{
    BorderRadius, Color, Constraints, EdgeInsets, Offset, Rect, Size, BoxDecoration, Border,
};

use crate::events::{EventResult, HitTestResult, InputEvent, UIMessage, GestureKind, MouseButton};
use crate::key::WidgetKey;
use crate::painter::{Painter, TextMeasurer};
use crate::render_object::{
    EventHandlable, Layoutable, Lifecycle, Paintable, Parent, RenderObject, RenderObjectState,
};
use crate::widget::Widget;

// ============================================================================
// Button State
// ============================================================================

/// Interaction state of a button
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonInteractionState {
    /// Button is enabled and not being interacted with
    #[default]
    Enabled,
    /// Button is disabled and cannot be interacted with
    Disabled,
    /// Mouse cursor is hovering over the button
    Hovered,
    /// Button is being pressed
    Pressed,
    /// Button has keyboard focus
    Focused,
}

impl ButtonInteractionState {
    /// Check if the button is enabled (not disabled)
    pub fn is_enabled(&self) -> bool {
        !matches!(self, ButtonInteractionState::Disabled)
    }

    /// Check if the button is interactive
    pub fn is_interactive(&self) -> bool {
        matches!(
            self,
            ButtonInteractionState::Enabled
                | ButtonInteractionState::Hovered
                | ButtonInteractionState::Focused
        )
    }
}

// ============================================================================
// Button Style
// ============================================================================

/// Style configuration for a button
///
/// This struct defines the visual appearance of a button in different states.
/// Inspired by Flutter's ButtonStyle.
#[derive(Debug, Clone, PartialEq)]
pub struct ButtonStyle {
    /// Background color for each state
    pub background_color: ButtonColorProperty,

    /// Foreground (text/icon) color for each state
    pub foreground_color: ButtonColorProperty,

    /// Border radius
    pub border_radius: BorderRadius,

    /// Padding inside the button
    pub padding: EdgeInsets,

    /// Minimum size of the button
    pub minimum_size: Size,

    /// Border configuration (for OutlinedButton)
    pub border: Option<ButtonBorderProperty>,

    /// Elevation (shadow depth) for each state
    pub elevation: ButtonElevationProperty,

    /// Shadow color
    pub shadow_color: Color,

    /// Animation duration for state transitions (in seconds)
    pub animation_duration: f32,
}

impl Default for ButtonStyle {
    fn default() -> Self {
        Self {
            background_color: ButtonColorProperty::default_elevated(),
            foreground_color: ButtonColorProperty::default_foreground(),
            border_radius: BorderRadius::all(4.0),
            padding: EdgeInsets::symmetric(16.0, 8.0),
            minimum_size: Size::new(64.0, 36.0),
            border: None,
            elevation: ButtonElevationProperty::default_elevated(),
            shadow_color: Color::new(0.0, 0.0, 0.0, 0.2),
            animation_duration: 0.2,
        }
    }
}

impl ButtonStyle {
    /// Create a style for elevated buttons
    pub fn elevated() -> Self {
        Self {
            background_color: ButtonColorProperty::default_elevated(),
            foreground_color: ButtonColorProperty::default_foreground(),
            elevation: ButtonElevationProperty::default_elevated(),
            ..Default::default()
        }
    }

    /// Create a style for outlined buttons
    pub fn outlined() -> Self {
        Self {
            background_color: ButtonColorProperty::default_outlined(),
            foreground_color: ButtonColorProperty::default_foreground(),
            border: Some(ButtonBorderProperty::default_outlined()),
            elevation: ButtonElevationProperty::none(),
            ..Default::default()
        }
    }

    /// Create a style for text buttons
    pub fn text() -> Self {
        Self {
            background_color: ButtonColorProperty::default_text(),
            foreground_color: ButtonColorProperty::default_foreground(),
            elevation: ButtonElevationProperty::none(),
            ..Default::default()
        }
    }

    /// Set the background color property
    pub fn background_color(mut self, color: ButtonColorProperty) -> Self {
        self.background_color = color;
        self
    }

    /// Set the foreground color property
    pub fn foreground_color(mut self, color: ButtonColorProperty) -> Self {
        self.foreground_color = color;
        self
    }

    /// Set the border radius
    pub fn border_radius(mut self, radius: BorderRadius) -> Self {
        self.border_radius = radius;
        self
    }

    /// Set the padding
    pub fn padding(mut self, padding: EdgeInsets) -> Self {
        self.padding = padding;
        self
    }

    /// Set the minimum size
    pub fn minimum_size(mut self, size: Size) -> Self {
        self.minimum_size = size;
        self
    }

    /// Set the border property
    pub fn border(mut self, border: Option<ButtonBorderProperty>) -> Self {
        self.border = border;
        self
    }

    /// Set the elevation property
    pub fn elevation(mut self, elevation: ButtonElevationProperty) -> Self {
        self.elevation = elevation;
        self
    }

    /// Get the background color for a given state
    pub fn background_color_for(&self, state: ButtonInteractionState) -> Color {
        self.background_color.resolve(state)
    }

    /// Get the foreground color for a given state
    pub fn foreground_color_for(&self, state: ButtonInteractionState) -> Color {
        self.foreground_color.resolve(state)
    }

    /// Get the elevation for a given state
    pub fn elevation_for(&self, state: ButtonInteractionState) -> f32 {
        self.elevation.resolve(state)
    }

    /// Get the border for a given state
    pub fn border_for(&self, state: ButtonInteractionState) -> Option<Border> {
        self.border.as_ref().map(|b| b.resolve(state))
    }

    /// Create a BoxDecoration for the current state
    pub fn decoration_for(&self, state: ButtonInteractionState) -> Option<BoxDecoration> {
        let bg_color = self.background_color_for(state);
        let border = self.border_for(state);

        if bg_color.a == 0.0 && border.is_none() {
            return None;
        }

        Some(BoxDecoration {
            color: if bg_color.a > 0.0 { Some(bg_color) } else { None },
            border,
            border_radius: Some(self.border_radius),
            box_shadow: None,
        })
    }
}

// ============================================================================
// State-based Property Types
// ============================================================================

/// A property that can have different values based on button state
#[derive(Debug, Clone, PartialEq)]
pub struct ButtonColorProperty {
    /// Color when enabled
    pub enabled: Color,
    /// Color when disabled
    pub disabled: Color,
    /// Color when hovered
    pub hovered: Color,
    /// Color when pressed
    pub pressed: Color,
    /// Color when focused
    pub focused: Color,
}

impl ButtonColorProperty {
    /// Create a new color property with the same color for all states
    pub fn all(color: Color) -> Self {
        Self {
            enabled: color,
            disabled: color,
            hovered: color,
            pressed: color,
            focused: color,
        }
    }

    /// Default background colors for elevated buttons
    pub fn default_elevated() -> Self {
        Self {
            enabled: Color::new(0.98, 0.98, 0.98, 1.0),
            disabled: Color::new(0.9, 0.9, 0.9, 0.38),
            hovered: Color::new(0.95, 0.95, 0.95, 1.0),
            pressed: Color::new(0.92, 0.92, 0.92, 1.0),
            focused: Color::new(0.98, 0.98, 0.98, 1.0),
        }
    }

    /// Default background colors for outlined buttons
    pub fn default_outlined() -> Self {
        Self {
            enabled: Color::TRANSPARENT,
            disabled: Color::TRANSPARENT,
            hovered: Color::new(0.0, 0.0, 0.0, 0.04),
            pressed: Color::new(0.0, 0.0, 0.0, 0.08),
            focused: Color::new(0.0, 0.0, 0.0, 0.04),
        }
    }

    /// Default background colors for text buttons
    pub fn default_text() -> Self {
        Self {
            enabled: Color::TRANSPARENT,
            disabled: Color::TRANSPARENT,
            hovered: Color::new(0.0, 0.0, 0.0, 0.04),
            pressed: Color::new(0.0, 0.0, 0.0, 0.08),
            focused: Color::new(0.0, 0.0, 0.0, 0.04),
        }
    }

    /// Default foreground colors
    pub fn default_foreground() -> Self {
        Self {
            enabled: Color::new(0.0, 0.0, 0.0, 0.87),
            disabled: Color::new(0.0, 0.0, 0.0, 0.38),
            hovered: Color::new(0.0, 0.0, 0.0, 0.87),
            pressed: Color::new(0.0, 0.0, 0.0, 0.87),
            focused: Color::new(0.0, 0.0, 0.0, 0.87),
        }
    }

    /// Resolve the color for a given state
    pub fn resolve(&self, state: ButtonInteractionState) -> Color {
        match state {
            ButtonInteractionState::Enabled => self.enabled,
            ButtonInteractionState::Disabled => self.disabled,
            ButtonInteractionState::Hovered => self.hovered,
            ButtonInteractionState::Pressed => self.pressed,
            ButtonInteractionState::Focused => self.focused,
        }
    }
}

/// Border property that can change based on button state
#[derive(Debug, Clone, PartialEq)]
pub struct ButtonBorderProperty {
    /// Border when enabled
    pub enabled: Border,
    /// Border when disabled
    pub disabled: Border,
    /// Border when hovered
    pub hovered: Border,
    /// Border when pressed
    pub pressed: Border,
    /// Border when focused
    pub focused: Border,
}

impl ButtonBorderProperty {
    /// Default border for outlined buttons
    pub fn default_outlined() -> Self {
        Self {
            enabled: Border::new(Color::new(0.0, 0.0, 0.0, 0.12), 1.0),
            disabled: Border::new(Color::new(0.0, 0.0, 0.0, 0.12), 1.0),
            hovered: Border::new(Color::new(0.0, 0.0, 0.0, 0.12), 1.0),
            pressed: Border::new(Color::new(0.0, 0.0, 0.0, 0.12), 1.0),
            focused: Border::new(Color::new(0.0, 0.0, 0.0, 0.12), 1.0),
        }
    }

    /// Create a border property with the same border for all states
    pub fn all(border: Border) -> Self {
        Self {
            enabled: border,
            disabled: border,
            hovered: border,
            pressed: border,
            focused: border,
        }
    }

    /// Resolve the border for a given state
    pub fn resolve(&self, state: ButtonInteractionState) -> Border {
        match state {
            ButtonInteractionState::Enabled => self.enabled,
            ButtonInteractionState::Disabled => self.disabled,
            ButtonInteractionState::Hovered => self.hovered,
            ButtonInteractionState::Pressed => self.pressed,
            ButtonInteractionState::Focused => self.focused,
        }
    }
}

/// Elevation property that can change based on button state
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ButtonElevationProperty {
    /// Elevation when enabled
    pub enabled: f32,
    /// Elevation when disabled
    pub disabled: f32,
    /// Elevation when hovered
    pub hovered: f32,
    /// Elevation when pressed
    pub pressed: f32,
    /// Elevation when focused
    pub focused: f32,
}

impl ButtonElevationProperty {
    /// Default elevation for elevated buttons
    pub fn default_elevated() -> Self {
        Self {
            enabled: 1.0,
            disabled: 0.0,
            hovered: 3.0,
            pressed: 1.0,
            focused: 1.0,
        }
    }

    /// No elevation
    pub fn none() -> Self {
        Self {
            enabled: 0.0,
            disabled: 0.0,
            hovered: 0.0,
            pressed: 0.0,
            focused: 0.0,
        }
    }

    /// Resolve the elevation for a given state
    pub fn resolve(&self, state: ButtonInteractionState) -> f32 {
        match state {
            ButtonInteractionState::Enabled => self.enabled,
            ButtonInteractionState::Disabled => self.disabled,
            ButtonInteractionState::Hovered => self.hovered,
            ButtonInteractionState::Pressed => self.pressed,
            ButtonInteractionState::Focused => self.focused,
        }
    }
}

// ============================================================================
// Button Widget
// ============================================================================

/// A generic button widget
///
/// This is the base button implementation. Use `ElevatedButton`, `OutlinedButton`,
/// or `TextButton` for convenience constructors.
#[derive(Debug)]
pub struct Button {
    /// Child widget to display inside the button
    pub child: Option<Box<dyn Widget>>,

    /// Button style configuration
    pub style: ButtonStyle,

    /// Whether the button is enabled
    pub enabled: bool,

    /// ID for press events
    pub press_id: Option<String>,

    /// ID for long press events
    pub long_press_id: Option<String>,

    /// Optional widget key
    pub key: Option<WidgetKey>,
}

impl Button {
    /// Create a new button with a child widget
    pub fn new() -> Self {
        Self {
            child: None,
            style: ButtonStyle::elevated(),
            enabled: true,
            press_id: None,
            long_press_id: None,
            key: None,
        }
    }

    /// Create a button with a child widget
    pub fn with_child(child: impl Widget + 'static) -> Self {
        Self {
            child: Some(Box::new(child)),
            style: ButtonStyle::elevated(),
            enabled: true,
            press_id: None,
            long_press_id: None,
            key: None,
        }
    }

    /// Set the child widget
    pub fn child(mut self, child: impl Widget + 'static) -> Self {
        self.child = Some(Box::new(child));
        self
    }

    /// Set the button style
    pub fn style(mut self, style: ButtonStyle) -> Self {
        self.style = style;
        self
    }

    /// Set whether the button is enabled
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set the press event ID
    pub fn on_press(mut self, id: impl Into<String>) -> Self {
        self.press_id = Some(id.into());
        self
    }

    /// Set the long press event ID
    pub fn on_long_press(mut self, id: impl Into<String>) -> Self {
        self.long_press_id = Some(id.into());
        self
    }

    /// Set the widget key
    pub fn with_key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.key = Some(key.into());
        self
    }
}

impl Default for Button {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Button {
    fn widget_type(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn key(&self) -> Option<WidgetKey> {
        self.key.clone()
    }

    fn children(&self) -> Vec<&dyn Widget> {
        self.child.as_ref().map(|c| vec![c.as_ref()]).unwrap_or_default()
    }

    fn create_render_object(&self) -> Box<dyn RenderObject> {
        let child_ro = self.child.as_ref().map(|c| c.create_render_object());
        Box::new(ButtonRenderObject::new(
            child_ro,
            self.style.clone(),
            self.enabled,
            self.press_id.clone(),
            self.long_press_id.clone(),
        ))
    }

    fn update_render_object(&self, render_object: &mut dyn RenderObject) {
        if let Some(button_ro) = render_object.as_any_mut().downcast_mut::<ButtonRenderObject>() {
            button_ro.style = self.style.clone();
            button_ro.enabled = self.enabled;
            button_ro.press_id = self.press_id.clone();
            button_ro.long_press_id = self.long_press_id.clone();
            button_ro.state.needs_layout = true;
        }
    }

    fn should_update(&self, old: &dyn Widget) -> bool {
        if let Some(old_button) = old.as_any().downcast_ref::<Button>() {
            self.style.border_radius != old_button.style.border_radius
                || self.style.padding != old_button.style.padding
                || self.enabled != old_button.enabled
        } else {
            true
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_boxed(&self) -> Box<dyn Widget> {
        Box::new(Button {
            child: self.child.as_ref().map(|c| c.clone_boxed()),
            style: self.style.clone(),
            enabled: self.enabled,
            press_id: self.press_id.clone(),
            long_press_id: self.long_press_id.clone(),
            key: self.key.clone(),
        })
    }
}

// ============================================================================
// Elevated Button
// ============================================================================

/// An elevated button with a shadow
///
/// Use elevated buttons to add dimension to otherwise mostly flat layouts.
/// Avoid using elevated buttons on already-elevated content.
#[derive(Debug)]
pub struct ElevatedButton {
    /// Child widget to display inside the button
    pub child: Option<Box<dyn Widget>>,

    /// Custom button style (optional)
    pub style: Option<ButtonStyle>,

    /// Whether the button is enabled
    pub enabled: bool,

    /// ID for press events
    pub press_id: Option<String>,

    /// ID for long press events
    pub long_press_id: Option<String>,

    /// Optional widget key
    pub key: Option<WidgetKey>,
}

impl ElevatedButton {
    /// Create a new elevated button
    pub fn new() -> Self {
        Self {
            child: None,
            style: None,
            enabled: true,
            press_id: None,
            long_press_id: None,
            key: None,
        }
    }

    /// Create an elevated button with a child widget
    pub fn with_child(child: impl Widget + 'static) -> Self {
        Self {
            child: Some(Box::new(child)),
            style: None,
            enabled: true,
            press_id: None,
            long_press_id: None,
            key: None,
        }
    }

    /// Set the child widget
    pub fn child(mut self, child: impl Widget + 'static) -> Self {
        self.child = Some(Box::new(child));
        self
    }

    /// Set a custom button style
    pub fn style(mut self, style: ButtonStyle) -> Self {
        self.style = Some(style);
        self
    }

    /// Set whether the button is enabled
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set the press event ID
    pub fn on_press(mut self, id: impl Into<String>) -> Self {
        self.press_id = Some(id.into());
        self
    }

    /// Set the long press event ID
    pub fn on_long_press(mut self, id: impl Into<String>) -> Self {
        self.long_press_id = Some(id.into());
        self
    }

    /// Set the widget key
    pub fn with_key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.key = Some(key.into());
        self
    }
}

impl Default for ElevatedButton {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for ElevatedButton {
    fn widget_type(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn key(&self) -> Option<WidgetKey> {
        self.key.clone()
    }

    fn children(&self) -> Vec<&dyn Widget> {
        self.child.as_ref().map(|c| vec![c.as_ref()]).unwrap_or_default()
    }

    fn create_render_object(&self) -> Box<dyn RenderObject> {
        let style = self.style.clone().unwrap_or_else(ButtonStyle::elevated);
        let child_ro = self.child.as_ref().map(|c| c.create_render_object());
        Box::new(ButtonRenderObject::new(
            child_ro,
            style,
            self.enabled,
            self.press_id.clone(),
            self.long_press_id.clone(),
        ))
    }

    fn update_render_object(&self, render_object: &mut dyn RenderObject) {
        if let Some(button_ro) = render_object.as_any_mut().downcast_mut::<ButtonRenderObject>() {
            if let Some(ref style) = self.style {
                button_ro.style = style.clone();
            }
            button_ro.enabled = self.enabled;
            button_ro.press_id = self.press_id.clone();
            button_ro.long_press_id = self.long_press_id.clone();
            button_ro.state.needs_layout = true;
        }
    }

    fn should_update(&self, old: &dyn Widget) -> bool {
        if let Some(old_button) = old.as_any().downcast_ref::<ElevatedButton>() {
            self.style != old_button.style
                || self.enabled != old_button.enabled
        } else {
            true
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_boxed(&self) -> Box<dyn Widget> {
        Box::new(ElevatedButton {
            child: self.child.as_ref().map(|c| c.clone_boxed()),
            style: self.style.clone(),
            enabled: self.enabled,
            press_id: self.press_id.clone(),
            long_press_id: self.long_press_id.clone(),
            key: self.key.clone(),
        })
    }
}

// ============================================================================
// Outlined Button
// ============================================================================

/// An outlined button with a border
///
/// Outlined buttons are medium-emphasis buttons. They contain actions
/// that are important but aren't the primary action in an app.
#[derive(Debug)]
pub struct OutlinedButton {
    /// Child widget to display inside the button
    pub child: Option<Box<dyn Widget>>,

    /// Custom button style (optional)
    pub style: Option<ButtonStyle>,

    /// Whether the button is enabled
    pub enabled: bool,

    /// ID for press events
    pub press_id: Option<String>,

    /// ID for long press events
    pub long_press_id: Option<String>,

    /// Optional widget key
    pub key: Option<WidgetKey>,
}

impl OutlinedButton {
    /// Create a new outlined button
    pub fn new() -> Self {
        Self {
            child: None,
            style: None,
            enabled: true,
            press_id: None,
            long_press_id: None,
            key: None,
        }
    }

    /// Create an outlined button with a child widget
    pub fn with_child(child: impl Widget + 'static) -> Self {
        Self {
            child: Some(Box::new(child)),
            style: None,
            enabled: true,
            press_id: None,
            long_press_id: None,
            key: None,
        }
    }

    /// Set the child widget
    pub fn child(mut self, child: impl Widget + 'static) -> Self {
        self.child = Some(Box::new(child));
        self
    }

    /// Set a custom button style
    pub fn style(mut self, style: ButtonStyle) -> Self {
        self.style = Some(style);
        self
    }

    /// Set whether the button is enabled
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set the press event ID
    pub fn on_press(mut self, id: impl Into<String>) -> Self {
        self.press_id = Some(id.into());
        self
    }

    /// Set the long press event ID
    pub fn on_long_press(mut self, id: impl Into<String>) -> Self {
        self.long_press_id = Some(id.into());
        self
    }

    /// Set the widget key
    pub fn with_key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.key = Some(key.into());
        self
    }
}

impl Default for OutlinedButton {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for OutlinedButton {
    fn widget_type(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn key(&self) -> Option<WidgetKey> {
        self.key.clone()
    }

    fn children(&self) -> Vec<&dyn Widget> {
        self.child.as_ref().map(|c| vec![c.as_ref()]).unwrap_or_default()
    }

    fn create_render_object(&self) -> Box<dyn RenderObject> {
        let style = self.style.clone().unwrap_or_else(ButtonStyle::outlined);
        let child_ro = self.child.as_ref().map(|c| c.create_render_object());
        Box::new(ButtonRenderObject::new(
            child_ro,
            style,
            self.enabled,
            self.press_id.clone(),
            self.long_press_id.clone(),
        ))
    }

    fn update_render_object(&self, render_object: &mut dyn RenderObject) {
        if let Some(button_ro) = render_object.as_any_mut().downcast_mut::<ButtonRenderObject>() {
            if let Some(ref style) = self.style {
                button_ro.style = style.clone();
            }
            button_ro.enabled = self.enabled;
            button_ro.press_id = self.press_id.clone();
            button_ro.long_press_id = self.long_press_id.clone();
            button_ro.state.needs_layout = true;
        }
    }

    fn should_update(&self, old: &dyn Widget) -> bool {
        if let Some(old_button) = old.as_any().downcast_ref::<OutlinedButton>() {
            self.style != old_button.style
                || self.enabled != old_button.enabled
        } else {
            true
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_boxed(&self) -> Box<dyn Widget> {
        Box::new(OutlinedButton {
            child: self.child.as_ref().map(|c| c.clone_boxed()),
            style: self.style.clone(),
            enabled: self.enabled,
            press_id: self.press_id.clone(),
            long_press_id: self.long_press_id.clone(),
            key: self.key.clone(),
        })
    }
}

// ============================================================================
// Text Button
// ============================================================================

/// A text button without border or elevation
///
/// Text buttons are used for the least prominent actions.
/// They're often embedded in contained components like dialogs and cards.
#[derive(Debug)]
pub struct TextButton {
    /// Child widget to display inside the button
    pub child: Option<Box<dyn Widget>>,

    /// Custom button style (optional)
    pub style: Option<ButtonStyle>,

    /// Whether the button is enabled
    pub enabled: bool,

    /// ID for press events
    pub press_id: Option<String>,

    /// ID for long press events
    pub long_press_id: Option<String>,

    /// Optional widget key
    pub key: Option<WidgetKey>,
}

impl TextButton {
    /// Create a new text button
    pub fn new() -> Self {
        Self {
            child: None,
            style: None,
            enabled: true,
            press_id: None,
            long_press_id: None,
            key: None,
        }
    }

    /// Create a text button with a child widget
    pub fn with_child(child: impl Widget + 'static) -> Self {
        Self {
            child: Some(Box::new(child)),
            style: None,
            enabled: true,
            press_id: None,
            long_press_id: None,
            key: None,
        }
    }

    /// Set the child widget
    pub fn child(mut self, child: impl Widget + 'static) -> Self {
        self.child = Some(Box::new(child));
        self
    }

    /// Set a custom button style
    pub fn style(mut self, style: ButtonStyle) -> Self {
        self.style = Some(style);
        self
    }

    /// Set whether the button is enabled
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set the press event ID
    pub fn on_press(mut self, id: impl Into<String>) -> Self {
        self.press_id = Some(id.into());
        self
    }

    /// Set the long press event ID
    pub fn on_long_press(mut self, id: impl Into<String>) -> Self {
        self.long_press_id = Some(id.into());
        self
    }

    /// Set the widget key
    pub fn with_key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.key = Some(key.into());
        self
    }
}

impl Default for TextButton {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for TextButton {
    fn widget_type(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn key(&self) -> Option<WidgetKey> {
        self.key.clone()
    }

    fn children(&self) -> Vec<&dyn Widget> {
        self.child.as_ref().map(|c| vec![c.as_ref()]).unwrap_or_default()
    }

    fn create_render_object(&self) -> Box<dyn RenderObject> {
        let style = self.style.clone().unwrap_or_else(ButtonStyle::text);
        let child_ro = self.child.as_ref().map(|c| c.create_render_object());
        Box::new(ButtonRenderObject::new(
            child_ro,
            style,
            self.enabled,
            self.press_id.clone(),
            self.long_press_id.clone(),
        ))
    }

    fn update_render_object(&self, render_object: &mut dyn RenderObject) {
        if let Some(button_ro) = render_object.as_any_mut().downcast_mut::<ButtonRenderObject>() {
            if let Some(ref style) = self.style {
                button_ro.style = style.clone();
            }
            button_ro.enabled = self.enabled;
            button_ro.press_id = self.press_id.clone();
            button_ro.long_press_id = self.long_press_id.clone();
            button_ro.state.needs_layout = true;
        }
    }

    fn should_update(&self, old: &dyn Widget) -> bool {
        if let Some(old_button) = old.as_any().downcast_ref::<TextButton>() {
            self.style != old_button.style
                || self.enabled != old_button.enabled
        } else {
            true
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_boxed(&self) -> Box<dyn Widget> {
        Box::new(TextButton {
            child: self.child.as_ref().map(|c| c.clone_boxed()),
            style: self.style.clone(),
            enabled: self.enabled,
            press_id: self.press_id.clone(),
            long_press_id: self.long_press_id.clone(),
            key: self.key.clone(),
        })
    }
}

// ============================================================================
// Button Render Object
// ============================================================================

/// Render object for Button widgets
#[derive(Debug)]
pub struct ButtonRenderObject {
    /// Render state
    state: RenderObjectState,
    /// Child render object
    pub child: Option<Box<dyn RenderObject>>,
    /// Button style
    pub style: ButtonStyle,
    /// Whether the button is enabled
    pub enabled: bool,
    /// Press event ID
    pub press_id: Option<String>,
    /// Long press event ID
    pub long_press_id: Option<String>,
    /// Interaction state
    interaction_state: ButtonInteractionState,
    /// Whether pointer is down
    pointer_down: bool,
}

impl ButtonRenderObject {
    /// Create a new button render object
    pub fn new(
        child: Option<Box<dyn RenderObject>>,
        style: ButtonStyle,
        enabled: bool,
        press_id: Option<String>,
        long_press_id: Option<String>,
    ) -> Self {
        let interaction_state = if enabled {
            ButtonInteractionState::Enabled
        } else {
            ButtonInteractionState::Disabled
        };

        Self {
            state: RenderObjectState::new(),
            child,
            style,
            enabled,
            press_id,
            long_press_id,
            interaction_state,
            pointer_down: false,
        }
    }

    /// Get the current interaction state
    pub fn interaction_state(&self) -> ButtonInteractionState {
        self.interaction_state
    }

    /// Update interaction state based on events
    fn update_interaction_state(&mut self) {
        if !self.enabled {
            self.interaction_state = ButtonInteractionState::Disabled;
        } else if self.pointer_down {
            self.interaction_state = ButtonInteractionState::Pressed;
        } else {
            self.interaction_state = ButtonInteractionState::Enabled;
        }
    }

    /// Get the current decoration
    fn current_decoration(&self) -> Option<BoxDecoration> {
        self.style.decoration_for(self.interaction_state)
    }
}

impl Layoutable for ButtonRenderObject {
    fn layout(&mut self, constraints: Constraints, text_measurer: &dyn TextMeasurer) -> Size {
        let child_constraints = constraints.loosen();
        let child_size = if let Some(ref mut child_ro) = self.child {
            child_ro.layout(child_constraints, text_measurer)
        } else {
            Size::zero()
        };

        let padding = self.style.padding;
        let padding_size = padding.total_size();

        let width = (child_size.width + padding_size.width)
            .max(self.style.minimum_size.width)
            .max(constraints.min_width);
        let height = (child_size.height + padding_size.height)
            .max(self.style.minimum_size.height)
            .max(constraints.min_height);

        let size = constraints.constrain(Size::new(width, height));

        if let Some(ref mut child_ro) = self.child {
            let child_x = (size.width - child_size.width) / 2.0;
            let child_y = (size.height - child_size.height) / 2.0;
            child_ro.set_offset(Offset::new(child_x, child_y));
        }

        self.state.size = size;
        self.state.needs_layout = false;

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

impl Paintable for ButtonRenderObject {
    fn paint(&self, painter: &mut dyn Painter) {
        painter.save();
        painter.translate(self.state.offset);

        if let Some(decoration) = self.current_decoration() {
            let rect = Rect::from_size(self.state.size);
            painter.draw_box_decoration(rect, &decoration);
        }

        if let Some(ref child_ro) = self.child {
            child_ro.paint(painter);
        }

        painter.restore();
    }

    fn needs_paint(&self) -> bool {
        self.state.needs_paint
    }

    fn mark_needs_paint(&mut self) {
        self.state.needs_paint = true;
    }
}

impl EventHandlable for ButtonRenderObject {
    fn hit_test(&self, position: Offset) -> HitTestResult {
        if !self.enabled {
            return HitTestResult::Miss;
        }

        if position.x >= 0.0
            && position.x <= self.state.size.width
            && position.y >= 0.0
            && position.y <= self.state.size.height
        {
            HitTestResult::Hit
        } else {
            HitTestResult::Miss
        }
    }

    fn handle_event(&mut self, event: &InputEvent) -> EventResult {
        // Note: position is already in local coordinates (relative to this render object)
        // because dispatch_event_recursive transforms coordinates via with_offset()
        // before calling handle_event. Do NOT subtract offset again!
        if !self.enabled {
            return EventResult::Ignored;
        }

        match event {
            InputEvent::MouseDown { position, button, .. } => {
                if *button == MouseButton::Left {
                    if self.hit_test(*position).is_hit() {
                        self.pointer_down = true;
                        self.update_interaction_state();
                        self.state.needs_paint = true;
                        return EventResult::Consumed;
                    }
                }
            }
            InputEvent::MouseUp { position, button, .. } => {
                if *button == MouseButton::Left && self.pointer_down {
                    self.pointer_down = false;
                    self.update_interaction_state();
                    self.state.needs_paint = true;

                    if self.hit_test(*position).is_hit() {
                        return EventResult::Consumed;
                    }
                }
            }
            InputEvent::MouseMove { position, .. } => {
                let is_inside = self.hit_test(*position).is_hit();

                if self.pointer_down && !is_inside {
                    self.pointer_down = false;
                    self.update_interaction_state();
                    self.state.needs_paint = true;
                }
            }
            InputEvent::Hover { entered, .. } => {
                if !entered && self.pointer_down {
                    self.pointer_down = false;
                    self.update_interaction_state();
                    self.state.needs_paint = true;
                }
            }
            InputEvent::Tap { position } => {
                if self.hit_test(*position).is_hit() {
                    if let Some(ref id) = self.press_id {
                        return EventResult::Message(UIMessage::Gesture {
                            id: id.clone(),
                            kind: GestureKind::Tap,
                        });
                    }
                    return EventResult::Consumed;
                }
            }
            InputEvent::LongPress { position } => {
                if self.hit_test(*position).is_hit() {
                    if let Some(ref id) = self.long_press_id {
                        return EventResult::Message(UIMessage::Gesture {
                            id: id.clone(),
                            kind: GestureKind::LongPress,
                        });
                    }
                    return EventResult::Consumed;
                }
            }
            _ => {}
        }

        EventResult::Ignored
    }
}

impl Lifecycle for ButtonRenderObject {
    fn on_mount(&mut self) {
        if let Some(ref mut child) = self.child {
            child.on_mount();
        }
    }

    fn on_unmount(&mut self) {
        if let Some(ref mut child) = self.child {
            child.on_unmount();
        }
    }
}

impl Parent for ButtonRenderObject {
    fn children(&self) -> Vec<&dyn RenderObject> {
        self.child
            .as_ref()
            .map(|c| vec![c.as_ref()])
            .unwrap_or_default()
    }

    fn children_mut(&mut self) -> Vec<&mut dyn RenderObject> {
        self.child
            .as_mut()
            .map(|c| vec![c.as_mut()])
            .unwrap_or_default()
    }
}

impl RenderObject for ButtonRenderObject {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
