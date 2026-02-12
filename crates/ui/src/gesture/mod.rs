//! Gesture Detection Module
//!
//! Provides gesture recognition from raw input events.
//! Detects high-level gestures like Tap and LongPress from MouseDown/MouseUp sequences.

use std::time::Instant;
use std::collections::VecDeque;

use hoshimi_types::Offset;

use crate::events::InputEvent;

/// Configuration for gesture detection thresholds
#[derive(Debug, Clone)]
pub struct GestureConfig {
    /// Distance threshold for tap detection (pixels)
    /// If the mouse moves more than this distance between down and up, it's not a tap
    pub tap_distance_threshold: f32,
    
    /// Time threshold for long press detection (seconds)
    /// If held longer than this, it's a long press instead of a tap
    pub long_press_threshold: f32,
    
    /// Maximum time for a tap gesture (seconds)
    /// If released within this time and distance threshold, it's a tap
    pub tap_time_threshold: f32,
}

impl Default for GestureConfig {
    fn default() -> Self {
        Self {
            tap_distance_threshold: 10.0,
            long_press_threshold: 0.5,
            tap_time_threshold: 0.3,
        }
    }
}

/// State machine for detecting gestures from raw input events
#[derive(Debug)]
pub struct GestureDetector {
    /// Configuration
    config: GestureConfig,
    
    /// Position where mouse was pressed
    mouse_down_position: Option<Offset>,
    
    /// Time when mouse was pressed
    mouse_down_time: Option<Instant>,
    
    /// Last known mouse position
    last_mouse_position: Offset,
}

impl Default for GestureDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl GestureDetector {
    /// Create a new gesture detector with default configuration
    pub fn new() -> Self {
        Self {
            config: GestureConfig::default(),
            mouse_down_position: None,
            mouse_down_time: None,
            last_mouse_position: Offset::ZERO,
        }
    }
    
    /// Create a gesture detector with custom configuration
    pub fn with_config(config: GestureConfig) -> Self {
        Self {
            config,
            mouse_down_position: None,
            mouse_down_time: None,
            last_mouse_position: Offset::ZERO,
        }
    }
    
    /// Process a raw input event and return any additional high-level gesture events
    /// 
    /// This method takes a raw input event (like MouseDown, MouseUp) and may generate
    /// additional gesture events (like Tap, LongPress) based on the gesture state machine.
    /// 
    /// Returns a vector of events - the original event plus any detected gestures.
    pub fn process_event(&mut self, event: InputEvent) -> Vec<InputEvent> {
        let mut events = vec![event.clone()];
        
        match &event {
            InputEvent::MouseDown { position, .. } => {
                self.mouse_down_position = Some(*position);
                self.mouse_down_time = Some(Instant::now());
                self.last_mouse_position = *position;
            }
            
            InputEvent::MouseUp { position, .. } => {
                if let Some(gesture) = self.detect_gesture_on_release(*position) {
                    events.push(gesture);
                }
                
                // Reset state
                self.mouse_down_position = None;
                self.mouse_down_time = None;
                self.last_mouse_position = *position;
            }
            
            InputEvent::MouseMove { position, .. } => {
                self.last_mouse_position = *position;
            }
            
            _ => {}
        }
        
        events
    }
    
    /// Detect gesture when mouse is released
    fn detect_gesture_on_release(&self, up_position: Offset) -> Option<InputEvent> {
        let down_pos = self.mouse_down_position?;
        let down_time = self.mouse_down_time?;
        
        // Calculate distance moved
        let dx = up_position.x - down_pos.x;
        let dy = up_position.y - down_pos.y;
        let distance = (dx * dx + dy * dy).sqrt();
        
        // Calculate time held
        let duration = down_time.elapsed().as_secs_f32();
        
        // Check if within tap distance threshold
        if distance <= self.config.tap_distance_threshold {
            if duration >= self.config.long_press_threshold {
                // Long press
                Some(InputEvent::LongPress { position: up_position })
            } else if duration <= self.config.tap_time_threshold {
                // Tap
                Some(InputEvent::Tap { position: up_position })
            } else {
                // Medium duration - not quite a tap, not quite long press
                // Treat as tap for now
                Some(InputEvent::Tap { position: up_position })
            }
        } else {
            // Moved too far, not a gesture
            None
        }
    }
    
    /// Reset the gesture detector state
    /// Call this when losing focus or when gestures should be cancelled
    pub fn reset(&mut self) {
        self.mouse_down_position = None;
        self.mouse_down_time = None;
    }
    
    /// Check if a gesture is currently in progress (mouse is down)
    pub fn is_gesture_in_progress(&self) -> bool {
        self.mouse_down_position.is_some()
    }
    
    /// Get the current gesture configuration
    pub fn config(&self) -> &GestureConfig {
        &self.config
    }
    
    /// Set a new gesture configuration
    pub fn set_config(&mut self, config: GestureConfig) {
        self.config = config;
    }
}

/// Event queue for batching and processing input events
/// 
/// This provides a way to queue events from the application's event loop
/// and process them in batches during the UI update cycle.
#[derive(Debug)]
pub struct InputEventQueue {
    /// Queued events
    events: VecDeque<InputEvent>,
    
    /// Gesture detector for generating high-level gestures
    gesture_detector: GestureDetector,
}

impl Default for InputEventQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl InputEventQueue {
    /// Create a new empty event queue
    pub fn new() -> Self {
        Self {
            events: VecDeque::new(),
            gesture_detector: GestureDetector::new(),
        }
    }
    
    /// Create an event queue with custom gesture configuration
    pub fn with_gesture_config(config: GestureConfig) -> Self {
        Self {
            events: VecDeque::new(),
            gesture_detector: GestureDetector::with_config(config),
        }
    }
    
    /// Push a raw input event into the queue
    /// 
    /// The event will be processed by the gesture detector, which may generate
    /// additional high-level gesture events (Tap, LongPress, etc.)
    pub fn push(&mut self, event: InputEvent) {
        let events = self.gesture_detector.process_event(event);
        self.events.extend(events);
    }
    
    /// Push a raw input event without gesture detection
    /// 
    /// Use this for events that should not trigger gesture detection,
    /// or when you've already done gesture detection externally.
    pub fn push_raw(&mut self, event: InputEvent) {
        self.events.push_back(event);
    }
    
    /// Pop the next event from the queue
    pub fn pop(&mut self) -> Option<InputEvent> {
        self.events.pop_front()
    }
    
    /// Drain all events from the queue
    pub fn drain(&mut self) -> impl Iterator<Item = InputEvent> + '_ {
        self.events.drain(..)
    }
    
    /// Check if the queue is empty
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
    
    /// Get the number of events in the queue
    pub fn len(&self) -> usize {
        self.events.len()
    }
    
    /// Clear all events from the queue
    pub fn clear(&mut self) {
        self.events.clear();
    }
    
    /// Reset the gesture detector state
    pub fn reset_gesture_state(&mut self) {
        self.gesture_detector.reset();
    }
    
    /// Get a reference to the gesture detector
    pub fn gesture_detector(&self) -> &GestureDetector {
        &self.gesture_detector
    }
    
    /// Get a mutable reference to the gesture detector
    pub fn gesture_detector_mut(&mut self) -> &mut GestureDetector {
        &mut self.gesture_detector
    }
}
