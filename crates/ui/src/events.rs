//! UI Events and Messages
//!
//! This module defines input events and UI messages for the Hoshimi UI system.

use hoshimi_shared::{Offset, Rect};

// ============================================================================
// Input Events
// ============================================================================

/// Input event types that can be handled by the UI system
#[derive(Debug, Clone)]
pub enum InputEvent {
    /// Mouse/touch tap event
    Tap {
        /// The spatial position of the tap on the display surface.
        position: Offset,
    },
    
    /// Mouse/touch long press event
    LongPress {
        /// The spatial position of the long press on the display surface.
        position: Offset,
    },
    
    /// Mouse hover event
    Hover {
        /// The spatial position of the hover on the display surface.
        position: Offset,
        /// True if mouse entered, false if exited
        entered: bool,
    },
    
    /// Mouse move event
    MouseMove {
        /// The endpoint of move event on the display surface
        position: Offset,
        /// The offset volume between start and end
        delta: Offset,
    },
    
    /// Mouse button press
    MouseDown {
        /// The spatial position of the mouse down on the display surface.
        position: Offset,
        /// The button clicked while the event occurred.
        button: MouseButton,
    },
    
    /// Mouse button release
    MouseUp {
        /// The spatial position of the mouse up on the display surface.
        position: Offset,
        /// The button released.
        button: MouseButton,
    },
    
    /// Scroll event
    Scroll {
        /// The spatial position of the mouse up on the display surface.
        position: Offset,
        /// The offset volume of the wheel scroll (include vertical and horizontal)
        delta: Offset,
    },
    
    /// Keyboard key press
    KeyDown {
        /// The key on keyboard pressed.
        key_code: KeyCode,
        /// The modifier key on keyboard pressed.
        modifiers: KeyModifiers,
    },
    
    /// Keyboard key release
    KeyUp {
        /// The key on keyboard released.
        key_code: KeyCode,
        /// The modifier key on keyboard released.
        modifiers: KeyModifiers,
    },
    
    /// Text input event
    TextInput {
        /// The inputted text
        text: String,
    },
}

impl InputEvent {
    /// Get the position of the event, if applicable
    pub fn position(&self) -> Option<Offset> {
        match self {
            InputEvent::Tap { position } => Some(*position),
            InputEvent::LongPress { position } => Some(*position),
            InputEvent::Hover { position, .. } => Some(*position),
            InputEvent::MouseMove { position, .. } => Some(*position),
            InputEvent::MouseDown { position, .. } => Some(*position),
            InputEvent::MouseUp { position, .. } => Some(*position),
            InputEvent::Scroll { position, .. } => Some(*position),
            _ => None,
        }
    }
}

#[doc(hidden)]
/// Mouse button identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u8),
}

#[doc(hidden)]
/// Keyboard key code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    // Letters
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    
    // Numbers
    Key0, Key1, Key2, Key3, Key4,
    Key5, Key6, Key7, Key8, Key9,
    
    // Function keys
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    
    // Navigation
    Up, Down, Left, Right,
    Home, End, PageUp, PageDown,
    
    // Editing
    Backspace, Delete, Insert,
    Enter, Tab, Escape,
    Space,
    
    // Modifiers
    LeftShift, RightShift,
    LeftCtrl, RightCtrl,
    LeftAlt, RightAlt,
    
    // Other
    Unknown(u32),
}

#[doc(hidden)]
/// Keyboard modifier state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct KeyModifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}

impl KeyModifiers {
    pub const fn new() -> Self {
        Self {
            shift: false,
            ctrl: false,
            alt: false,
            meta: false,
        }
    }
    
    pub const fn none() -> Self {
        Self::new()
    }
    
    pub fn is_empty(&self) -> bool {
        !self.shift && !self.ctrl && !self.alt && !self.meta
    }
}

// ============================================================================
// Event Results
// ============================================================================

/// Result of handling an input event
#[derive(Debug, Clone, PartialEq)]
pub enum EventResult {
    /// Event was handled, stop propagation
    Handled,
    
    /// Event was not handled, continue propagation
    Ignored,
    
    /// Event was consumed and produced a UI message
    Consumed,
    
    /// Event produced a message
    Message(UIMessage),
}

impl EventResult {
    /// Check if event propagation should stop
    pub fn should_stop(&self) -> bool {
        matches!(self, EventResult::Handled | EventResult::Consumed | EventResult::Message(_))
    }
}

// ============================================================================
// Hit Test Results
// ============================================================================

/// Result of a hit test - simple enum version
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitTestResult {
    /// No hit
    Miss,
    /// Hit and absorbs events
    Hit,
    /// Hit but transparent to events
    HitTransparent,
}

impl Default for HitTestResult {
    fn default() -> Self {
        Self::Miss
    }
}

#[doc(hidden)]
impl HitTestResult {
    pub fn miss() -> Self {
        Self::Miss
    }
    
    pub fn hit() -> Self {
        Self::Hit
    }
    
    pub fn hit_transparent() -> Self {
        Self::HitTransparent
    }
    
    pub fn is_hit(&self) -> bool {
        matches!(self, Self::Hit | Self::HitTransparent)
    }
}

/// An entry in the hit test path
#[derive(Debug, Clone)]
pub struct HitTestEntry {
    /// The rect of the hit widget in global coordinates
    pub rect: Rect,
    
    /// Node identifier (for debugging)
    pub node_id: u64,
}

// ============================================================================
// UI Messages
// ============================================================================

/// Messages produced by UI interactions
/// 
/// These messages are collected by the UI system and returned to the application
/// for processing, decoupling UI from business logic.
#[derive(Debug, Clone, PartialEq)]
pub enum UIMessage {
    /// Dialog confirmation (e.g., user clicked to continue)
    DialogConfirm,
    
    /// Option selected in a choice menu
    OptionSelect {
        /// The selected option
        index: usize,
        /// Optional label of the selected option
        label: Option<String>,
    },
    
    /// Menu action triggered
    MenuAction(MenuAction),
    
    /// Button clicked
    ButtonClick {
        /// Button identifier
        id: String,
    },
    
    /// Custom message with arbitrary payload
    Custom(CustomMessage),
}

#[doc(hidden)]
/// Menu actions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    Save,
    Load,
    Settings,
    BackToTitle,
    Quit,
    Resume,
    History,
    Skip,
    Auto,
}

/// Custom message wrapper for arbitrary data
#[derive(Debug, Clone, PartialEq)]
pub struct CustomMessage {
    /// Message type identifier
    pub type_id: String,
    /// Optional string payload
    pub payload: Option<String>,
}

impl CustomMessage {
    /// Create a new CustomMessage
    pub fn new(type_id: impl Into<String>) -> Self {
        Self {
            type_id: type_id.into(),
            payload: None,
        }
    }
    
    /// Create a new CustomMessage with payload
    pub fn with_payload(type_id: impl Into<String>, payload: impl Into<String>) -> Self {
        Self {
            type_id: type_id.into(),
            payload: Some(payload.into()),
        }
    }
}
