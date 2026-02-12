//! Simple Counter Page Example
//!
//! Demonstrates basic state management in a Page.
//!
//! This example shows:
//! - Storing state in a Page struct
//! - Modifying state through public methods
//! - Automatic UI rebuild when state changes

use hoshimi_types::TextAlign;
use hoshimi_types::{Border, BoxShadow, Offset};
use hoshimi_ui::impl_page_common;
use hoshimi_ui::prelude::*;
use hoshimi_ui::widget::AnimatedBoxShadow;
use std::cell::Cell;

/// A simple counter page with state management
#[derive(Debug)]
pub struct CounterPage {
    /// Current count value
    count: i32,

    /// Page title
    title: String,

    /// Button pressed states: (decrement, reset, increment, navigate)
    btn_pressed: (bool, bool, bool, bool),

    /// Flag indicating if UI needs to be rebuilt
    /// Using Cell allows modification in &self methods
    needs_rebuild: Cell<bool>,
}

impl CounterPage {
    /// Create a new counter page
    pub fn new() -> Self {
        Self {
            count: 0,
            title: "Counter Example".to_string(),
            needs_rebuild: Cell::new(false),
            btn_pressed: (false, false, false, false),
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

    /// Set button pressed state
    pub fn set_button_pressed(&mut self, btn_id: &str, pressed: bool) {
        match btn_id {
            "btn_decrement" => self.btn_pressed.0 = pressed,
            "btn_reset" => self.btn_pressed.1 = pressed,
            "btn_increment" => self.btn_pressed.2 = pressed,
            "btn_to_animation_test" => self.btn_pressed.3 = pressed,
            _ => {}
        }
        self.needs_rebuild.set(true);
    }

    /// Create shadow based on pressed state
    fn make_shadow(pressed: bool) -> BoxShadow {
        if pressed {
            // Pressed: shallow shadow (closer to surface)
            BoxShadow::new(Color::from_rgba8(0, 0, 0, 80), Offset::zero(), 16.0, -1.0)
        } else {
            // Not pressed: deeper shadow (elevated)
            BoxShadow::new(Color::from_rgba8(0, 0, 0, 150), Offset::zero(), 32.0, 0.0)
        }
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
                            Text::new(&format!("Count: {}", self.count))
                                .with_align(TextAlign::Center)
                                .with_style(TextStyle {
                                    font_size: 48.0,
                                    color: if self.count >= 0 {
                                        Color::from_rgba8(0, 128, 0, 255) // Green for positive
                                    } else {
                                        Color::from_rgba8(255, 0, 0, 255) // Red for negative
                                    },
                                    ..Default::default()
                                }),
                        )
                        .child(SizedBox::from_height(30.0))
                        .child(
                            Row::new()
                                .child(
                                    GestureDetector::new(
                                        AnimatedBoxShadow::new(
                                            Container::new()
                                                .child(
                                                    Text::new("-")
                                                        .with_color(Color::white())
                                                        .with_size(32.0)
                                                        .with_align(TextAlign::Center),
                                                )
                                                .with_padding(EdgeInsets::all(10.0))
                                                .with_decoration(BoxDecoration {
                                                    color: Some(Color::from_hex(0x1565C0)),
                                                    border: Some(Border::new(
                                                        Color::from_hex(0x00838F),
                                                        1.0,
                                                    )),
                                                    border_radius: Some(BorderRadius::all(16.0)),
                                                    box_shadow: None,
                                                })
                                                .with_alignment(Alignment::center())
                                                .with_padding(EdgeInsets::symmetric(32.0, 64.0)),
                                            Self::make_shadow(self.btn_pressed.0),
                                        )
                                        .with_duration(0.15),
                                    )
                                    .on_tap("btn_decrement")
                                    .on_press("btn_decrement")
                                    .on_release("btn_decrement"),
                                )
                                .child(SizedBox::from_width(20.0))
                                .child(
                                    GestureDetector::new(
                                        AnimatedBoxShadow::new(
                                            Container::new()
                                                .child(
                                                    Text::new("Reset")
                                                        .with_color(Color::white())
                                                        .with_size(32.0)
                                                        .with_align(TextAlign::Center),
                                                )
                                                .with_padding(EdgeInsets::all(10.0))
                                                .with_decoration(BoxDecoration {
                                                    color: Some(Color::from_hex(0x2E7D32)),
                                                    border: Some(Border::new(
                                                        Color::from_hex(0x558B2F),
                                                        1.0,
                                                    )),
                                                    border_radius: Some(BorderRadius::all(16.0)),
                                                    box_shadow: None,
                                                })
                                                .with_alignment(Alignment::center())
                                                .with_padding(EdgeInsets::symmetric(32.0, 64.0)),
                                            Self::make_shadow(self.btn_pressed.1),
                                        )
                                        .with_duration(0.15),
                                    )
                                    .on_tap("btn_reset")
                                    .on_press("btn_reset")
                                    .on_release("btn_reset"),
                                )
                                .child(SizedBox::from_width(20.0))
                                .child(
                                    GestureDetector::new(
                                        AnimatedBoxShadow::new(
                                            Container::new()
                                                .child(
                                                    Text::new("+")
                                                        .with_color(Color::white())
                                                        .with_size(32.0)
                                                        .with_align(TextAlign::Center),
                                                )
                                                .with_padding(EdgeInsets::all(10.0))
                                                .with_decoration(BoxDecoration {
                                                    color: Some(Color::from_hex(0xD84315)),
                                                    border: Some(Border::new(
                                                        Color::from_hex(0xEF6C00),
                                                        1.0,
                                                    )),
                                                    border_radius: Some(BorderRadius::all(16.0)),
                                                    box_shadow: None,
                                                })
                                                .with_alignment(Alignment::center())
                                                .with_padding(EdgeInsets::symmetric(32.0, 64.0)),
                                            Self::make_shadow(self.btn_pressed.2),
                                        )
                                        .with_duration(0.15),
                                    )
                                    .on_tap("btn_increment")
                                    .on_press("btn_increment")
                                    .on_release("btn_increment"),
                                )
                                .with_main_axis_alignment(MainAxisAlignment::Center),
                        )
                        .child(SizedBox::from_height(60.0))
                        .child(
                            GestureDetector::new(
                                AnimatedBoxShadow::new(
                                    Container::new()
                                        .child(
                                            Text::new("Test Animations")
                                                .with_color(Color::white())
                                                .with_size(32.0),
                                        )
                                        .with_decoration(BoxDecoration {
                                            color: Some(Color::from_hex(0x6A1B9A)),
                                            border: Some(Border::new(
                                                Color::from_hex(0x8E24AA),
                                                2.0,
                                            )),
                                            border_radius: Some(BorderRadius::all(16.0)),
                                            box_shadow: None,
                                        })
                                        .with_padding(EdgeInsets::symmetric(32.0, 64.0))
                                        .with_alignment(Alignment::center()),
                                    Self::make_shadow(self.btn_pressed.3),
                                )
                                .with_duration(0.15),
                            )
                            .on_tap("btn_to_animation_test")
                            .on_press("btn_to_animation_test")
                            .on_release("btn_to_animation_test"),
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

    impl_page_common!();
}
