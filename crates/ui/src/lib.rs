//! Hoshimi UI System
//!
//! A declarative UI framework designed for visual novel game engines.
//! 
//! # Architecture
//! 
//! The UI system follows a dual-tree architecture:
//! - **Widget Tree**: Immutable configuration describing the UI structure
//! - **RenderObject Tree**: Mutable objects responsible for layout and painting
//! 
//! # Core Concepts
//! 
//! ## Widgets
//! Widgets are lightweight, immutable descriptions of a piece of UI.
//! They implement the [`Widget`] trait and can be composed to build complex interfaces.
//! 
//! ```ignore
//! use hoshimi_ui::widget::*;
//! 
//! let ui = Column::new()
//!     .child(Text::new("Hello, World!"))
//!     .child(Container::new(Image::new("background.png")))
//!     .with_spacing(10.0);
//! ```
//! 
//! ## RenderObjects
//! RenderObjects handle the actual layout computation and painting.
//! They are created automatically from widgets and managed by the UI tree.
//! 
//! ## Events
//! The system uses a message-based event system where user interactions
//! generate [`UIMessage`] values that can be handled by the application.
//! 
//! # Visual Novel Components
//! 
//! The library includes specialized widgets for visual novel games:
//! - [`Background`] - Scene backgrounds with transitions
//! - [`Sprite`] - Character sprites with expressions
//! - [`DialogBox`] - Text boxes with typewriter effects
//! - [`ChoiceMenu`] - Interactive choice menus
//! 
//! # Example
//! 
//! ```ignore
//! use hoshimi_ui::prelude::*;
//! 
//! // Create a simple visual novel scene
//! let scene = Stack::new()
//!     .child(Background::new("cg/scene1.png"))
//!     .child(Sprite::new("alice", "happy").at_left())
//!     .child(DialogBox::new("Hello!")
//!         .with_speaker("Alice"));
//! 
//! // Create UI tree and render
//! let mut tree = UiTree::with_root(scene);
//! tree.set_size(1920.0, 1080.0);
//! tree.paint(&mut painter);
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod animation;
pub mod events;
pub mod key;
pub mod painter;
pub mod render;
pub mod tree;
pub mod widget;

// Re-export commonly used items
pub use animation::{AnimationController, AnimationStatus, Curve, Interpolate, RepeatMode, Tween};
pub use events::{EventResult, InputEvent, UIMessage};
pub use key::WidgetKey;
pub use painter::{Painter, TextMeasurer};
pub use render::{RenderObject, RenderObjectState};
pub use tree::UiTree;
pub use widget::Widget;

#[doc(hidden)]
/// Prelude module for convenient imports
pub mod prelude {
    //! Convenient re-exports of commonly used types.
    //!
    //! ```ignore
    //! use hoshimi_ui::prelude::*;
    //! ```

    // Core traits and types
    pub use crate::events::{EventResult, HitTestResult, InputEvent, UIMessage};
    pub use crate::key::{WidgetIdentity, WidgetKey};
    pub use crate::painter::{Painter, TextMeasurer};
    pub use crate::render::{RenderObject, RenderObjectState};
    pub use crate::tree::UiTree;
    pub use crate::widget::Widget;

    // Animation
    pub use crate::animation::{
        AnimationController, AnimationGroup, AnimationStatus, Curve, Interpolate,
        RepeatMode, Tween, TweenSequence,
    };

    // Basic widgets
    pub use crate::widget::basic::{
        Align, Center, Column, Container, Expanded, Flexible, FlexFit,
        GestureDetector, Image, Padding, Positioned, Row,
        SizedBox, Stack, StackFit, Text,
    };

    // Animated widgets
    pub use crate::widget::animated::{
        AnimatedOpacity, AnimatedPosition, AnimatedScale,
        FadeTransition, SlideDirection, SlideTransition,
    };

    // Visual novel widgets
    pub use crate::widget::novel::{
        Background, BackgroundTransition, ChoiceLayout, ChoiceMenu, ChoiceOption,
        DialogBox, DialogStyle, Sprite, SpritePosition, SpriteTransition, TextReveal,
    };

    // Re-export shared types that are commonly used
    pub use hoshimi_shared::{
        Alignment, BorderRadius, BoxDecoration, BoxShadow, Color, Constraints,
        CrossAxisAlignment, EdgeInsets, ImageFit, MainAxisAlignment, MainAxisSize,
        Offset, Rect, Size, TextOverflow, TextStyle,
    };
}
