//! State Traits
//!
//! Defines the core traits for state management in the UI system.

use crate::InputEvent;
use crate::Widget;
use std::fmt::Debug;

/// Trait for managing widget state.
///
/// WidgetState is responsible for:
/// - Managing mutable state for a StatefulWidget
/// - Handling input events and updating state accordingly
/// - Determining when the widget needs to be rebuilt
///
/// # Example
///
/// ```ignore
/// struct CounterState {
///     count: i32,
/// }
///
/// impl WidgetState for CounterState {
///     fn build(&self) -> Box<dyn Widget> {
///         Text::new(format!("Count: {}", self.count))
///     }
///
///     fn handle_event(&mut self, event: &InputEvent) -> bool {
///         if matches!(event, InputEvent::Tap { .. }) {
///             self.count += 1;
///             return true; // Needs rebuild
///         }
///         false
///     }
///
///     fn tick(&mut self, _delta: f32) -> bool {
///         false
///     }
/// }
/// ```
pub trait WidgetState: Debug + Send + Sync {
    /// Build the widget tree based on current state.
    ///
    /// This method is called when the state changes and the widget
    /// needs to be rebuilt.
    fn build(&self) -> Box<dyn Widget>;

    /// Handle an input event.
    ///
    /// Returns `true` if the state changed and the widget needs to be rebuilt.
    fn handle_event(&mut self, event: &InputEvent) -> bool;

    /// Called every frame for time-based state updates.
    ///
    /// Returns `true` if the state changed and the widget needs to be rebuilt.
    ///
    /// # Arguments
    ///
    /// * `delta` - Time elapsed since the last frame in seconds
    fn tick(&mut self, delta: f32) -> bool;
}

/// Trait for widgets that have internal state.
///
/// StatefulWidgets maintain state that can change over time, such as:
/// - User interaction state (pressed, hovered, etc.)
/// - Form input values
/// - Animation progress
///
/// # Example
///
/// ```ignore
/// struct Button {
///     id: String,
///     label: String,
/// }
///
/// impl StatefulWidget for Button {
///     type State = ButtonState;
///
///     fn create_state(&self) -> Self::State {
///         ButtonState::new(self.id.clone())
///     }
/// }
/// ```
pub trait StatefulWidget: Widget {
    /// The state type associated with this widget.
    type State: WidgetState;

    /// Create the initial state for this widget.
    ///
    /// Called once when the widget is first mounted to the tree.
    fn create_state(&self) -> Self::State;
}

/// Trait for widgets that do not have internal state.
///
/// StatelessWidgets are pure functions of their configuration.
/// They are rebuilt entirely when their parent widget rebuilds.
///
/// # Example
///
/// ```ignore
/// struct Container {
///     child: Option<Box<dyn Widget>>,
///     padding: EdgeInsets,
/// }
///
/// impl StatelessWidget for Container {}
/// ```
pub trait StatelessWidget: Widget {}
