//! Animation Test Page
//!
//! A page for testing route animations and transitions.
//!
//! This example shows:
//! - Navigation between pages
//! - Different visual styles
//! - Route transition animations
//! - Using ElevatedButton for navigation

use hoshimi_ui::types::TextAlign;
use hoshimi_ui::impl_page_common;
use hoshimi_ui::prelude::*;
use hoshimi_ui::widget::{ElevatedButton, ButtonStyle};
use hoshimi_ui::router::TransitionType;
use std::cell::Cell;

/// Animation test page with navigation
#[derive(Debug)]
pub struct AnimationTestPage {
    /// Page title
    title: String,

    /// Flag indicating if UI needs to be rebuilt
    needs_rebuild: Cell<bool>,
}

impl AnimationTestPage {
    /// Create a new animation test page
    pub fn new() -> Self {
        Self {
            title: "Animation Test".to_string(),
            needs_rebuild: Cell::new(false),
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
                            ElevatedButton::with_child(Text::new("Back to Counter"))
                                .on_press("btn_back_to_counter")
                                .style(
                                    ButtonStyle::elevated()
                                        .background_color(
                                            hoshimi_ui::widget::ButtonColorProperty::all(
                                                Color::from_hex(0x7B1FA2)
                                            )
                                        )
                                        .border_radius(BorderRadius::all(16.0))
                                        .padding(EdgeInsets::symmetric(32.0, 64.0))
                                )
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

    fn enter_transition(&self) -> TransitionType {
        TransitionType::slide_left().with_duration(1.0).with_curve(Curve::EaseInOutQuart)
    }

    fn exit_transition(&self) -> TransitionType {
        TransitionType::slide_right().with_duration(1.0).with_curve(Curve::EaseInOutQuart)
    }

    impl_page_common!();
}
