//! Novel Widgets Module
//!
//! Contains visual novel specific UI components.

mod background;
mod sprite;
mod dialog_box;
mod choice_menu;

pub use background::{Background, BackgroundTransition, BackgroundRenderObject};
pub use sprite::{Sprite, SpritePosition, SpriteTransition, SpriteRenderObject};
pub use dialog_box::{DialogBox, DialogStyle, TextReveal, DialogBoxRenderObject};
pub use choice_menu::{ChoiceMenu, ChoiceOption, ChoiceLayout, ChoiceMenuRenderObject};
