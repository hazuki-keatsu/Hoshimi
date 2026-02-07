//! Sprite Widget
//!
//! Widget for displaying character sprites with expressions and animations.

use std::any::{Any, TypeId};

use hoshimi_shared::{Alignment, Constraints, Offset, Rect, Size};

use crate::key::WidgetKey;
use crate::painter::Painter;
use crate::render::{RenderObject, RenderObjectState};
use crate::widget::Widget;
use crate::impl_render_object_common;

/// Sprite position preset
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum SpritePosition {
    /// Custom position (x, y in normalized 0.0-1.0 coordinates)
    Custom(f32, f32),
    /// Left side of screen
    Left,
    /// Center of screen
    #[default]
    Center,
    /// Right side of screen
    Right,
    /// Far left (for 5-character scenes)
    FarLeft,
    /// Far right (for 5-character scenes)
    FarRight,
}

impl SpritePosition {
    /// Convert position to normalized x coordinate
    pub fn to_x(&self) -> f32 {
        match self {
            SpritePosition::Custom(x, _) => *x,
            SpritePosition::FarLeft => 0.1,
            SpritePosition::Left => 0.25,
            SpritePosition::Center => 0.5,
            SpritePosition::Right => 0.75,
            SpritePosition::FarRight => 0.9,
        }
    }
    
    /// Convert position to normalized y coordinate
    pub fn to_y(&self) -> f32 {
        match self {
            SpritePosition::Custom(_, y) => *y,
            _ => 1.0, // Default: bottom-aligned
        }
    }
}

/// Sprite transition effect
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum SpriteTransition {
    /// No transition
    #[default]
    None,
    /// Fade in/out
    Fade {
        /// The duration of the transition 
        duration: f32,
    },
    /// Slide from position
    Slide {
        /// The end position of the slide sprite
        from: SpritePosition,
        /// The duration of the transition
        duration: f32,
    },
}

/// Character sprite widget
#[derive(Debug)]
pub struct Sprite {
    /// Character identifier
    pub character_id: String,
    
    /// Expression/pose identifier
    pub expression: String,
    
    /// Optional outfit/costume
    pub outfit: Option<String>,
    
    /// Screen position
    pub position: SpritePosition,
    
    /// Vertical alignment (0 = top, 1 = bottom)
    pub vertical_align: Alignment,
    
    /// Scale factor
    pub scale: f32,
    
    /// Opacity
    pub opacity: f32,
    
    /// Flip horizontally
    pub flip_x: bool,
    
    /// Transition effect
    pub transition: SpriteTransition,
    
    /// Layer order (higher = on top)
    pub layer: i32,
    
    /// Optional widget key
    pub key: Option<WidgetKey>,
}

impl Sprite {
    /// Create a new sprite
    pub fn new(character_id: impl Into<String>, expression: impl Into<String>) -> Self {
        Self {
            character_id: character_id.into(),
            expression: expression.into(),
            outfit: None,
            position: SpritePosition::Center,
            vertical_align: Alignment::BOTTOM_CENTER,
            scale: 1.0,
            opacity: 1.0,
            flip_x: false,
            transition: SpriteTransition::None,
            layer: 0,
            key: None,
        }
    }
    
    /// Set outfit
    pub fn with_outfit(mut self, outfit: impl Into<String>) -> Self {
        self.outfit = Some(outfit.into());
        self
    }
    
    /// Set position
    pub fn with_position(mut self, position: SpritePosition) -> Self {
        self.position = position;
        self
    }
    
    /// Set position to left
    pub fn at_left(mut self) -> Self {
        self.position = SpritePosition::Left;
        self
    }
    
    /// Set position to center
    pub fn at_center(mut self) -> Self {
        self.position = SpritePosition::Center;
        self
    }
    
    /// Set position to right
    pub fn at_right(mut self) -> Self {
        self.position = SpritePosition::Right;
        self
    }
    
    /// Set vertical alignment
    pub fn with_vertical_align(mut self, align: Alignment) -> Self {
        self.vertical_align = align;
        self
    }
    
    /// Set scale
    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }
    
    /// Set opacity
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }
    
    /// Set horizontal flip
    pub fn with_flip_x(mut self, flip: bool) -> Self {
        self.flip_x = flip;
        self
    }
    
    /// Set transition
    pub fn with_transition(mut self, transition: SpriteTransition) -> Self {
        self.transition = transition;
        self
    }
    
    /// Set layer
    pub fn with_layer(mut self, layer: i32) -> Self {
        self.layer = layer;
        self
    }
    
    /// Set the widget key
    pub fn with_key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.key = Some(key.into());
        self
    }
    
    /// Build image source path from character, expression, and outfit
    pub fn build_source_path(&self) -> String {
        let outfit_part = self.outfit.as_ref()
            .map(|o| format!("/{}", o))
            .unwrap_or_default();
        
        format!("character/{}{}/{}.png", self.character_id, outfit_part, self.expression)
    }
}

impl Widget for Sprite {
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
        Box::new(SpriteRenderObject::new(
            self.build_source_path(),
            self.position,
            self.vertical_align,
            self.scale,
            self.opacity,
            self.flip_x,
            self.transition,
            self.layer,
        ))
    }
    
    fn update_render_object(&self, render_object: &mut dyn RenderObject) {
        if let Some(sprite_ro) = render_object.as_any_mut().downcast_mut::<SpriteRenderObject>() {
            let new_source = self.build_source_path();
            
            // Check if expression changed to trigger transition
            if sprite_ro.source != new_source {
                sprite_ro.source = new_source;
                sprite_ro.transition = self.transition;
                sprite_ro.transition_progress = 0.0;
            }
            
            // Check if position changed for slide animation
            if sprite_ro.position != self.position {
                sprite_ro.previous_position = Some(sprite_ro.position);
                sprite_ro.position = self.position;
            }
            
            sprite_ro.vertical_align = self.vertical_align;
            sprite_ro.scale = self.scale;
            sprite_ro.opacity = self.opacity;
            sprite_ro.flip_x = self.flip_x;
            sprite_ro.layer = self.layer;
            sprite_ro.state.mark_needs_layout();
        }
    }
    
    fn should_update(&self, old: &dyn Widget) -> bool {
        if let Some(old_sprite) = old.as_any().downcast_ref::<Sprite>() {
            self.character_id != old_sprite.character_id ||
            self.expression != old_sprite.expression ||
            self.outfit != old_sprite.outfit ||
            self.position != old_sprite.position ||
            self.vertical_align != old_sprite.vertical_align ||
            (self.scale - old_sprite.scale).abs() > f32::EPSILON ||
            (self.opacity - old_sprite.opacity).abs() > f32::EPSILON ||
            self.flip_x != old_sprite.flip_x ||
            self.transition != old_sprite.transition ||
            self.layer != old_sprite.layer
        } else {
            true
        }
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_boxed(&self) -> Box<dyn Widget> {
        Box::new(Sprite {
            character_id: self.character_id.clone(),
            expression: self.expression.clone(),
            outfit: self.outfit.clone(),
            position: self.position,
            vertical_align: self.vertical_align,
            scale: self.scale,
            opacity: self.opacity,
            flip_x: self.flip_x,
            transition: self.transition,
            layer: self.layer,
            key: self.key.clone(),
        })
    }
}

/// Render object for Sprite widget
#[derive(Debug)]
pub struct SpriteRenderObject {
    state: RenderObjectState,
    source: String,
    position: SpritePosition,
    previous_position: Option<SpritePosition>,
    vertical_align: Alignment,
    scale: f32,
    opacity: f32,
    flip_x: bool,
    transition: SpriteTransition,
    transition_progress: f32,
    layer: i32,
    /// Intrinsic sprite size (loaded from image)
    sprite_size: Option<Size>,
}

impl SpriteRenderObject {
    fn new(
        source: String,
        position: SpritePosition,
        vertical_align: Alignment,
        scale: f32,
        opacity: f32,
        flip_x: bool,
        transition: SpriteTransition,
        layer: i32,
    ) -> Self {
        Self {
            state: RenderObjectState::new(),
            source,
            position,
            previous_position: None,
            vertical_align,
            scale,
            opacity,
            flip_x,
            transition,
            transition_progress: 1.0,
            layer,
            sprite_size: None,
        }
    }
    
    /// Set the sprite's intrinsic size (call after loading the image)
    pub fn set_sprite_size(&mut self, size: Size) {
        self.sprite_size = Some(size);
        self.state.mark_needs_layout();
    }
    
    /// Get layer for sorting
    pub fn layer(&self) -> i32 {
        self.layer
    }
    
    /// Update transition progress
    pub fn update_transition(&mut self, delta_time: f32) -> bool {
        if self.transition_progress >= 1.0 {
            return false;
        }
        
        let duration = match self.transition {
            SpriteTransition::None => return false,
            SpriteTransition::Fade { duration } => duration,
            SpriteTransition::Slide { duration, .. } => duration,
        };
        
        if duration > 0.0 {
            self.transition_progress = (self.transition_progress + delta_time / duration).min(1.0);
            self.state.mark_needs_paint();
            
            if self.transition_progress >= 1.0 {
                self.previous_position = None;
            }
            
            true
        } else {
            self.transition_progress = 1.0;
            self.previous_position = None;
            false
        }
    }
}

impl RenderObject for SpriteRenderObject {
    impl_render_object_common!(state);
    
    fn layout(&mut self, constraints: Constraints) -> Size {
        // Sprite positions itself within the parent
        let parent_size = Size::new(constraints.max_width, constraints.max_height);
        
        // Use sprite's intrinsic size if available, otherwise estimate
        let sprite_size = self.sprite_size.unwrap_or(Size::new(200.0, 400.0));
        let scaled_size = Size::new(
            sprite_size.width * self.scale,
            sprite_size.height * self.scale,
        );
        
        // Calculate position based on preset
        let x = self.position.to_x() * parent_size.width - scaled_size.width / 2.0;
        let y = if self.vertical_align.y < 0.0 {
            // Top aligned
            0.0
        } else if self.vertical_align.y == 0.0 {
            // Center aligned
            (parent_size.height - scaled_size.height) / 2.0
        } else {
            // Bottom aligned (default)
            parent_size.height - scaled_size.height
        };
        
        self.state.offset = Offset::new(x, y);
        self.state.size = scaled_size;
        self.state.needs_layout = false;
        
        scaled_size
    }
    
    fn paint(&self, painter: &mut dyn Painter) {
        painter.save();
        
        // Handle flip
        if self.flip_x {
            painter.translate(Offset::new(
                self.state.offset.x + self.state.size.width,
                self.state.offset.y,
            ));
            painter.scale(-1.0, 1.0);
        } else {
            painter.translate(self.state.offset);
        }
        
        // Calculate opacity based on transition
        let current_opacity = match self.transition {
            SpriteTransition::Fade { .. } if self.transition_progress < 1.0 => {
                self.transition_progress * self.opacity
            }
            _ => self.opacity,
        };
        
        let rect = Rect::from_size(self.state.size);
        painter.draw_image_to_rect(&self.source, rect, current_opacity);
        
        painter.restore();
    }
}