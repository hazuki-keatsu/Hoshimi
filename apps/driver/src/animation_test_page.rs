//! Animation Test Page
//!
//! A page for testing route animations and transitions.
//!
//! This example shows:
//! - Navigation between pages
//! - Different visual styles
//! - Route transition animations

use hoshimi_ui::types::{Border, BoxShadow, Offset, TextAlign};
use hoshimi_ui::impl_page_common;
use hoshimi_ui::prelude::*;
use hoshimi_ui::router::TransitionType; // Import for custom transitions
use hoshimi_ui::widget::AnimatedBoxShadow;
use std::cell::Cell;

/// Animation test page with navigation
#[derive(Debug)]
pub struct AnimationTestPage {
    /// Page title
    title: String,

    /// Button pressed state
    btn_pressed: bool,

    /// Flag indicating if UI needs to be rebuilt
    needs_rebuild: Cell<bool>,
}

impl AnimationTestPage {
    /// Create a new animation test page
    pub fn new() -> Self {
        Self {
            title: "Animation Test".to_string(),
            btn_pressed: false,
            needs_rebuild: Cell::new(false),
        }
    }

    /// Set button pressed state
    pub fn set_button_pressed(&mut self, pressed: bool) {
        self.btn_pressed = pressed;
        self.needs_rebuild.set(true);
    }

    /// Create shadow based on pressed state
    fn make_shadow(pressed: bool) -> BoxShadow {
        if pressed {
            BoxShadow::new(Color::from_rgba8(0, 0, 0, 80), Offset::zero(), 16.0, -1.0)
        } else {
            BoxShadow::new(Color::from_rgba8(0, 0, 0, 150), Offset::zero(), 32.0, 0.0)
        }
    }
}

impl Default for AnimationTestPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for AnimationTestPage {
    fn route_name(&self) -> &str {
        "animation_test"
    }

    fn build(&self) -> Box<dyn Widget> {
        Box::new(
            Container::new()
                .with_decoration(BoxDecoration {
                    color: Some(Color::blue()),
                    ..Default::default()
                })
                .child(Center::new(
                    Column::new()
                        .with_cross_axis_alignment(CrossAxisAlignment::Center)
                        .child(SizedBox::from_height(32.0))
                        .child(
                            Text::new(&self.title)
                                .with_align(TextAlign::Center)
                                .with_style(TextStyle {
                                    font_size: 48.0,
                                    color: Color::white(),
                                    ..Default::default()
                                }),
                        )
                        .child(SizedBox::from_height(60.0))
                        .child(
                            GestureDetector::new(
                                AnimatedBoxShadow::new(
                                    Container::new()
                                        .child(
                                            Text::new("Back to Counter")
                                                .with_color(Color::white())
                                                .with_size(28.0)
                                                .with_align(TextAlign::Center),
                                        )
                                        .with_decoration(BoxDecoration {
                                            color: Some(Color::from_hex(0x7B1FA2)),
                                            border: Some(Border::new(
                                                Color::from_hex(0x9C27B0),
                                                2.0,
                                            )),
                                            border_radius: Some(BorderRadius::all(16.0)),
                                            box_shadow: None,
                                        })
                                        .with_padding_all(12.0)
                                        .with_alignment(Alignment::center()),
                                    Self::make_shadow(self.btn_pressed),
                                )
                                .with_duration(0.15),
                            )
                            .on_tap("btn_back_to_counter")
                            .on_press("btn_back_to_counter")
                            .on_release("btn_back_to_counter"),
                        ),
                )),
        )
    }

    fn needs_rebuild(&self) -> bool {
        self.needs_rebuild.get()
    }

    fn mark_rebuilt(&mut self) {
        self.needs_rebuild.set(false);
    }

    // Override the default enter transition: Scale/Zoom effect
    fn enter_transition(&self) -> TransitionType {
        TransitionType::slide_left().with_duration(1.0).with_curve(Curve::EaseInOutQuart)
    }

    // Override the default exit transition: Scale/Zoom effect (reverse)
    fn exit_transition(&self) -> TransitionType {
        TransitionType::slide_right().with_duration(1.0).with_curve(Curve::EaseInOutQuart)
    }

    impl_page_common!();
}
