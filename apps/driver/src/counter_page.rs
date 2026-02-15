//! Simple Counter Page Example
//!
//! Demonstrates basic state management in a Page with the new Button component.
//!
//! This example shows:
//! - Storing state in a Page struct
//! - Handling messages directly in the page
//! - Using ElevatedButton, OutlinedButton, and TextButton

use hoshimi_ui::events::{GestureKind, UIMessage};
use hoshimi_ui::impl_page_common;
use hoshimi_ui::prelude::*;
use hoshimi_ui::types::TextAlign;
use hoshimi_ui::widget::{ButtonStyle, ElevatedButton, OutlinedButton, TextButton};
use std::cell::Cell;

/// A simple counter page with state management
#[derive(Debug)]
pub struct CounterPage {
    /// Current count value
    count: i32,

    /// Page title
    title: String,

    /// Flag indicating if UI needs to be rebuilt
    needs_rebuild: Cell<bool>,
}

impl CounterPage {
    /// Create a new counter page
    pub fn new() -> Self {
        Self {
            count: 0,
            title: "Counter Example".to_string(),
            needs_rebuild: Cell::new(false),
        }
    }

    /// Increment the counter
    pub fn increment(&mut self) {
        self.count += 1;
        self.needs_rebuild.set(true);
    }

    /// Decrement the counter
    pub fn decrement(&mut self) {
        self.count -= 1;
        self.needs_rebuild.set(true);
    }

    /// Reset the counter to zero
    pub fn reset(&mut self) {
        self.count = 0;
        self.needs_rebuild.set(true);
    }

    /// Get the current count (read-only access)
    pub fn count(&self) -> i32 {
        self.count
    }
}

impl Default for CounterPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for CounterPage {
    fn route_name(&self) -> &str {
        "counter"
    }

    fn build(&self) -> Box<dyn Widget> {
        Box::new(
            Container::new()
                .with_decoration(BoxDecoration {
                    color: Some(Color::white()),
                    ..Default::default()
                })
                .child(Center::new(
                    Column::new()
                        .child(
                            Text::new(&self.title)
                                .with_align(TextAlign::Center)
                                .with_style(TextStyle {
                                    font_size: 32.0,
                                    color: Color::white(),
                                    ..Default::default()
                                }),
                        )
                        .child(SizedBox::from_height(20.0))
                        .child(
                            Container::new()
                                .child(
                                    SizedBox::from_width(100.0).with_child(
                                        Text::new("Test Overflow")
                                            .with_overflow(TextOverflow::Ellipsis),
                                    ),
                                )
                                .with_decoration(BoxDecoration {
                                    color: Some(Color::cyan()),
                                    ..Default::default()
                                }),
                        )
                        .child(SizedBox::from_height(20.0))
                        .child(
                            Text::new(&format!("Count: {}", self.count))
                                .with_align(TextAlign::Center)
                                .with_style(TextStyle {
                                    font_size: 48.0,
                                    color: if self.count >= 0 {
                                        Color::from_rgba8(0, 128, 0, 255)
                                    } else {
                                        Color::from_rgba8(255, 0, 0, 255)
                                    },
                                    ..Default::default()
                                }),
                        )
                        .child(SizedBox::from_height(30.0))
                        .child(
                            Row::new()
                                .child(
                                    ElevatedButton::with_child(Text::new("-").with_size(28.0))
                                        .on_press("btn_decrement")
                                        .style(
                                            ButtonStyle::elevated()
                                                .background_color(
                                                    hoshimi_ui::widget::ButtonColorProperty::all(
                                                        Color::from_hex(0x1565C0),
                                                    ),
                                                )
                                                .border_radius(BorderRadius::all(16.0))
                                                .padding(EdgeInsets::symmetric(32.0, 64.0)),
                                        ),
                                )
                                .child(SizedBox::from_width(20.0))
                                .child(
                                    OutlinedButton::with_child(Text::new("Reset").with_size(28.0))
                                        .on_press("btn_reset")
                                        .style(
                                            ButtonStyle::outlined()
                                                .foreground_color(
                                                    hoshimi_ui::widget::ButtonColorProperty::all(
                                                        Color::from_hex(0x2E7D32),
                                                    ),
                                                )
                                                .border_radius(BorderRadius::all(16.0))
                                                .padding(EdgeInsets::symmetric(32.0, 64.0)),
                                        ),
                                )
                                .child(SizedBox::from_width(20.0))
                                .child(
                                    ElevatedButton::with_child(Text::new("+").with_size(28.0))
                                        .on_press("btn_increment")
                                        .style(
                                            ButtonStyle::elevated()
                                                .background_color(
                                                    hoshimi_ui::widget::ButtonColorProperty::all(
                                                        Color::from_hex(0xD84315),
                                                    ),
                                                )
                                                .border_radius(BorderRadius::all(16.0))
                                                .padding(EdgeInsets::symmetric(32.0, 64.0)),
                                        ),
                                )
                                .with_main_axis_alignment(MainAxisAlignment::Center),
                        )
                        .child(SizedBox::from_height(60.0))
                        .child(
                            TextButton::with_child(Text::new("Test Animations").with_size(28.0))
                                .on_press("btn_to_animation_test")
                                .style(
                                    ButtonStyle::text()
                                        .foreground_color(
                                            hoshimi_ui::widget::ButtonColorProperty::all(
                                                Color::from_hex(0x6A1B9A),
                                            ),
                                        )
                                        .padding(EdgeInsets::symmetric(32.0, 64.0)),
                                ),
                        )
                        .with_main_axis_alignment(MainAxisAlignment::Center)
                        .with_cross_axis_alignment(CrossAxisAlignment::Center),
                )),
        )
    }

    fn needs_rebuild(&self) -> bool {
        self.needs_rebuild.get()
    }

    fn mark_rebuilt(&mut self) {
        self.needs_rebuild.set(false);
    }

    fn handle_message(&mut self, message: &UIMessage) -> bool {
        match message {
            UIMessage::Gesture {
                id,
                kind: GestureKind::Tap,
            } => match id.as_str() {
                "btn_increment" => {
                    self.increment();
                    true
                }
                "btn_decrement" => {
                    self.decrement();
                    true
                }
                "btn_reset" => {
                    self.reset();
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }

    impl_page_common!();
}
