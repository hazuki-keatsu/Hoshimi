//! Background Widget
//!
//! Widget for displaying visual novel backgrounds with transitions.

use std::any::{Any, TypeId};

use hoshimi_shared::{Color, Constraints, Rect, Size};

use crate::key::WidgetKey;
use crate::painter::Painter;
use crate::render::{RenderObject, RenderObjectState};
use crate::widget::Widget;
use crate::impl_render_object_common;

/// Background transition type
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum BackgroundTransition {
    /// No transition
    #[default]
    None,
    /// Fade in/out
    Fade {
        /// Transition duration in seconds
        duration: f32,
    },
    /// Dissolve between backgrounds
    Dissolve {
        /// Transition duration in seconds
        duration: f32,
    },
    /// Slide in from direction
    Slide {
        /// Direction to slide from (0 = left, 1 = right, 2 = top, 3 = bottom)
        direction: u8,
        /// Transition duration in seconds
        duration: f32,
    },
}

/// Background widget for visual novel scenes
#[derive(Debug)]
pub struct Background {
    /// Image source path
    pub source: String,
    
    /// Background color (shown while loading or as fallback)
    pub color: Option<Color>,
    
    /// Transition effect
    pub transition: BackgroundTransition,
    
    /// Opacity (0.0 - 1.0)
    pub opacity: f32,
    
    /// Optional widget key
    pub key: Option<WidgetKey>,
}

impl Background {
    /// Create a new background with image source
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            color: None,
            transition: BackgroundTransition::None,
            opacity: 1.0,
            key: None,
        }
    }
    
    /// Create a solid color background
    pub fn solid(color: Color) -> Self {
        Self {
            source: String::new(),
            color: Some(color),
            transition: BackgroundTransition::None,
            opacity: 1.0,
            key: None,
        }
    }
    
    /// Set fallback color
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
    
    /// Set transition effect
    pub fn with_transition(mut self, transition: BackgroundTransition) -> Self {
        self.transition = transition;
        self
    }
    
    /// Set opacity
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }
    
    /// Set the widget key
    pub fn with_key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.key = Some(key.into());
        self
    }
}

impl Widget for Background {
    fn widget_type(&self) -> TypeId {
        TypeId::of::<Self>()
    }
    
    fn key(&self) -> Option<WidgetKey> {
        self.key.clone()
    }
    
    fn children(&self) -> Vec<&dyn Widget> {
        vec![]
    }
    
    fn create_render_object(&self) -> Box<dyn RenderObject> {
        Box::new(BackgroundRenderObject::new(
            self.source.clone(),
            self.color,
            self.transition,
            self.opacity,
        ))
    }
    
    fn update_render_object(&self, render_object: &mut dyn RenderObject) {
        if let Some(bg_ro) = render_object.as_any_mut().downcast_mut::<BackgroundRenderObject>() {
            // Check if source changed to trigger transition
            if bg_ro.source != self.source {
                bg_ro.previous_source = Some(bg_ro.source.clone());
                bg_ro.source = self.source.clone();
                bg_ro.transition = self.transition;
                bg_ro.transition_progress = 0.0;
            }
            bg_ro.color = self.color;
            bg_ro.opacity = self.opacity;
            bg_ro.state.mark_needs_paint();
        }
    }
    
    fn should_update(&self, old: &dyn Widget) -> bool {
        if let Some(old_bg) = old.as_any().downcast_ref::<Background>() {
            self.source != old_bg.source ||
            self.color != old_bg.color ||
            self.transition != old_bg.transition ||
            (self.opacity - old_bg.opacity).abs() > f32::EPSILON
        } else {
            true
        }
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_boxed(&self) -> Box<dyn Widget> {
        Box::new(Background {
            source: self.source.clone(),
            color: self.color,
            transition: self.transition,
            opacity: self.opacity,
            key: self.key.clone(),
        })
    }
}

/// Render object for Background widget
#[derive(Debug)]
pub struct BackgroundRenderObject {
    state: RenderObjectState,
    source: String,
    previous_source: Option<String>,
    color: Option<Color>,
    transition: BackgroundTransition,
    transition_progress: f32,
    opacity: f32,
}

impl BackgroundRenderObject {
    fn new(
        source: String,
        color: Option<Color>,
        transition: BackgroundTransition,
        opacity: f32,
    ) -> Self {
        Self {
            state: RenderObjectState::new(),
            source,
            previous_source: None,
            color,
            transition,
            transition_progress: 1.0, // Start fully transitioned
            opacity,
        }
    }
    
    /// Update transition progress
    pub fn update_transition(&mut self, delta_time: f32) -> bool {
        if self.transition_progress >= 1.0 {
            return false;
        }
        
        let duration = match self.transition {
            BackgroundTransition::None => return false,
            BackgroundTransition::Fade { duration } => duration,
            BackgroundTransition::Dissolve { duration } => duration,
            BackgroundTransition::Slide { duration, .. } => duration,
        };
        
        if duration > 0.0 {
            self.transition_progress = (self.transition_progress + delta_time / duration).min(1.0);
            self.state.mark_needs_paint();
            
            // Clear previous source when transition completes
            if self.transition_progress >= 1.0 {
                self.previous_source = None;
            }
            
            true
        } else {
            self.transition_progress = 1.0;
            self.previous_source = None;
            false
        }
    }
    
    /// Check if transition is in progress
    pub fn is_transitioning(&self) -> bool {
        self.transition_progress < 1.0
    }
}

impl RenderObject for BackgroundRenderObject {
    impl_render_object_common!(state);
    
    fn layout(&mut self, constraints: Constraints) -> Size {
        // Background always fills available space
        let size = Size::new(constraints.max_width, constraints.max_height);
        self.state.size = size;
        self.state.needs_layout = false;
        size
    }
    
    fn paint(&self, painter: &mut dyn Painter) {
        painter.save();
        painter.translate(self.state.offset);
        
        let rect = Rect::from_size(self.state.size);
        
        // Draw background color if set
        if let Some(color) = self.color {
            painter.draw_rect(rect, color.with_alpha(color.a * self.opacity));
        }
        
        // Draw previous background during transition
        if let Some(ref prev_source) = self.previous_source {
            if self.transition_progress < 1.0 && !prev_source.is_empty() {
                let prev_opacity = (1.0 - self.transition_progress) * self.opacity;
                // Draw with calculated opacity
                painter.draw_image_to_rect(
                    prev_source,
                    rect,
                    prev_opacity,
                );
            }
        }
        
        // Draw current background
        if !self.source.is_empty() {
            let current_opacity = match self.transition {
                BackgroundTransition::None => self.opacity,
                BackgroundTransition::Fade { .. } => self.transition_progress * self.opacity,
                BackgroundTransition::Dissolve { .. } => self.transition_progress * self.opacity,
                BackgroundTransition::Slide { .. } => self.opacity, // Slide uses position, not opacity
            };
            
            let draw_rect = match self.transition {
                BackgroundTransition::Slide { direction, .. } if self.transition_progress < 1.0 => {
                    let progress = self.transition_progress;
                    match direction {
                        0 => Rect::new(
                            rect.x - rect.width * (1.0 - progress),
                            rect.y,
                            rect.width,
                            rect.height,
                        ),
                        1 => Rect::new(
                            rect.x + rect.width * (1.0 - progress),
                            rect.y,
                            rect.width,
                            rect.height,
                        ),
                        2 => Rect::new(
                            rect.x,
                            rect.y - rect.height * (1.0 - progress),
                            rect.width,
                            rect.height,
                        ),
                        _ => Rect::new(
                            rect.x,
                            rect.y + rect.height * (1.0 - progress),
                            rect.width,
                            rect.height,
                        ),
                    }
                }
                _ => rect,
            };
            
            painter.draw_image_to_rect(&self.source, draw_rect, current_opacity);
        }
        
        painter.restore();
    }
}
