//! SDL3 to Hoshimi UI Event Adapter
//!
//! This module converts SDL3 events into Hoshimi UI InputEvents.
//! Gesture detection (Tap, LongPress) is handled by the UI crate's InputEventQueue.

use hoshimi_shared::Offset;
use hoshimi_ui::prelude::{InputEvent, KeyCode, KeyModifiers, MouseButton};
use sdl3::event::Event as SdlEvent;
use sdl3::keyboard::Keycode as SdlKeycode;
use sdl3::keyboard::Mod as SdlMod;
use sdl3::mouse::MouseButton as SdlMouseButton;

/// Convert SDL3 mouse button to UI MouseButton
pub fn convert_mouse_button(button: SdlMouseButton) -> MouseButton {
    match button {
        SdlMouseButton::Left => MouseButton::Left,
        SdlMouseButton::Right => MouseButton::Right,
        SdlMouseButton::Middle => MouseButton::Middle,
        _ => MouseButton::Other(button as u8),
    }
}

/// Convert SDL3 keycode to UI KeyCode
pub fn convert_keycode(keycode: Option<SdlKeycode>) -> KeyCode {
    match keycode {
        Some(k) => match k {
            // Letters
            SdlKeycode::A => KeyCode::A,
            SdlKeycode::B => KeyCode::B,
            SdlKeycode::C => KeyCode::C,
            SdlKeycode::D => KeyCode::D,
            SdlKeycode::E => KeyCode::E,
            SdlKeycode::F => KeyCode::F,
            SdlKeycode::G => KeyCode::G,
            SdlKeycode::H => KeyCode::H,
            SdlKeycode::I => KeyCode::I,
            SdlKeycode::J => KeyCode::J,
            SdlKeycode::K => KeyCode::K,
            SdlKeycode::L => KeyCode::L,
            SdlKeycode::M => KeyCode::M,
            SdlKeycode::N => KeyCode::N,
            SdlKeycode::O => KeyCode::O,
            SdlKeycode::P => KeyCode::P,
            SdlKeycode::Q => KeyCode::Q,
            SdlKeycode::R => KeyCode::R,
            SdlKeycode::S => KeyCode::S,
            SdlKeycode::T => KeyCode::T,
            SdlKeycode::U => KeyCode::U,
            SdlKeycode::V => KeyCode::V,
            SdlKeycode::W => KeyCode::W,
            SdlKeycode::X => KeyCode::X,
            SdlKeycode::Y => KeyCode::Y,
            SdlKeycode::Z => KeyCode::Z,
            
            // Numbers
            SdlKeycode::_0 => KeyCode::Key0,
            SdlKeycode::_1 => KeyCode::Key1,
            SdlKeycode::_2 => KeyCode::Key2,
            SdlKeycode::_3 => KeyCode::Key3,
            SdlKeycode::_4 => KeyCode::Key4,
            SdlKeycode::_5 => KeyCode::Key5,
            SdlKeycode::_6 => KeyCode::Key6,
            SdlKeycode::_7 => KeyCode::Key7,
            SdlKeycode::_8 => KeyCode::Key8,
            SdlKeycode::_9 => KeyCode::Key9,
            
            // Function keys
            SdlKeycode::F1 => KeyCode::F1,
            SdlKeycode::F2 => KeyCode::F2,
            SdlKeycode::F3 => KeyCode::F3,
            SdlKeycode::F4 => KeyCode::F4,
            SdlKeycode::F5 => KeyCode::F5,
            SdlKeycode::F6 => KeyCode::F6,
            SdlKeycode::F7 => KeyCode::F7,
            SdlKeycode::F8 => KeyCode::F8,
            SdlKeycode::F9 => KeyCode::F9,
            SdlKeycode::F10 => KeyCode::F10,
            SdlKeycode::F11 => KeyCode::F11,
            SdlKeycode::F12 => KeyCode::F12,
            
            // Navigation
            SdlKeycode::Up => KeyCode::Up,
            SdlKeycode::Down => KeyCode::Down,
            SdlKeycode::Left => KeyCode::Left,
            SdlKeycode::Right => KeyCode::Right,
            SdlKeycode::Home => KeyCode::Home,
            SdlKeycode::End => KeyCode::End,
            SdlKeycode::PageUp => KeyCode::PageUp,
            SdlKeycode::PageDown => KeyCode::PageDown,
            
            // Editing
            SdlKeycode::Backspace => KeyCode::Backspace,
            SdlKeycode::Delete => KeyCode::Delete,
            SdlKeycode::Insert => KeyCode::Insert,
            SdlKeycode::Return => KeyCode::Enter,
            SdlKeycode::Tab => KeyCode::Tab,
            SdlKeycode::Escape => KeyCode::Escape,
            SdlKeycode::Space => KeyCode::Space,
            
            // Modifiers
            SdlKeycode::LShift => KeyCode::LeftShift,
            SdlKeycode::RShift => KeyCode::RightShift,
            SdlKeycode::LCtrl => KeyCode::LeftCtrl,
            SdlKeycode::RCtrl => KeyCode::RightCtrl,
            SdlKeycode::LAlt => KeyCode::LeftAlt,
            SdlKeycode::RAlt => KeyCode::RightAlt,
            
            _ => KeyCode::Unknown(k as u32),
        },
        None => KeyCode::Unknown(0),
    }
}

/// Convert SDL3 key modifiers to UI KeyModifiers
pub fn convert_key_modifiers(keymod: SdlMod) -> KeyModifiers {
    KeyModifiers {
        shift: keymod.contains(SdlMod::LSHIFTMOD) || keymod.contains(SdlMod::RSHIFTMOD),
        ctrl: keymod.contains(SdlMod::LCTRLMOD) || keymod.contains(SdlMod::RCTRLMOD),
        alt: keymod.contains(SdlMod::LALTMOD) || keymod.contains(SdlMod::RALTMOD),
        meta: keymod.contains(SdlMod::LGUIMOD) || keymod.contains(SdlMod::RGUIMOD),
    }
}

/// Convert SDL3 event to UI InputEvent
/// 
/// Returns None if the event is not convertible to an InputEvent
/// (e.g., window events, quit events).
/// 
/// Note: Gesture detection (Tap, LongPress) is now handled by the UI crate's
/// InputEventQueue. This function only converts raw input events.
pub fn convert_event(event: &SdlEvent) -> Option<InputEvent> {
    match event {
        SdlEvent::MouseButtonDown { x, y, mouse_btn, .. } => {
            Some(InputEvent::MouseDown {
                position: Offset::new(*x, *y),
                button: convert_mouse_button(*mouse_btn),
            })
        }
        
        SdlEvent::MouseButtonUp { x, y, mouse_btn, .. } => {
            Some(InputEvent::MouseUp {
                position: Offset::new(*x, *y),
                button: convert_mouse_button(*mouse_btn),
            })
        }
        
        SdlEvent::MouseMotion { x, y, xrel, yrel, .. } => {
            Some(InputEvent::MouseMove {
                position: Offset::new(*x, *y),
                delta: Offset::new(*xrel, *yrel),
            })
        }
        
        SdlEvent::MouseWheel { x, y, mouse_x, mouse_y, .. } => {
            Some(InputEvent::Scroll {
                position: Offset::new(*mouse_x, *mouse_y),
                delta: Offset::new(*x, *y),
            })
        }
        
        SdlEvent::KeyDown { keycode, keymod, .. } => {
            Some(InputEvent::KeyDown {
                key_code: convert_keycode(*keycode),
                modifiers: convert_key_modifiers(*keymod),
            })
        }
        
        SdlEvent::KeyUp { keycode, keymod, .. } => {
            Some(InputEvent::KeyUp {
                key_code: convert_keycode(*keycode),
                modifiers: convert_key_modifiers(*keymod),
            })
        }
        
        SdlEvent::TextInput { text, .. } => {
            Some(InputEvent::TextInput {
                text: text.clone(),
            })
        }
        
        _ => None,
    }
}
