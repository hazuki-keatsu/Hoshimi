//! Image Widget
//!
//! Displays images with various fit modes and alignment options.

use std::any::{Any, TypeId};

use hoshimi_shared::{Alignment, Constraints, ImageFit, Rect, Size};

use crate::key::WidgetKey;
use crate::painter::Painter;
use crate::render::{RenderObject, RenderObjectState};
use crate::widget::Widget;
use crate::impl_render_object_common;

/// Image widget for displaying images
#[derive(Debug, Clone)]
pub struct Image {
    /// Image source (key or path)
    pub source: String,
    
    /// How the image should fit within its bounds
    pub fit: ImageFit,
    
    /// Alignment within the widget bounds
    pub alignment: Alignment,
    
    /// Opacity (0.0 to 1.0)
    pub opacity: f32,
    
    /// Explicit width (None to use intrinsic size)
    pub width: Option<f32>,
    
    /// Explicit height (None to use intrinsic size)
    pub height: Option<f32>,
    
    /// Optional widget key
    pub key: Option<WidgetKey>,
}

impl Image {
    /// Create a new image widget
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            fit: ImageFit::Contain,
            alignment: Alignment::center(),
            opacity: 1.0,
            width: None,
            height: None,
            key: None,
        }
    }
    
    /// Set the fit mode
    pub fn with_fit(mut self, fit: ImageFit) -> Self {
        self.fit = fit;
        self
    }
    
    /// Set the alignment
    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }
    
    /// Set the opacity
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }
    
    /// Set explicit width
    pub fn with_width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }
    
    /// Set explicit height
    pub fn with_height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }
    
    /// Set explicit size
    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }
    
    /// Set the widget key
    pub fn with_key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.key = Some(key.into());
        self
    }
}

impl Widget for Image {
    fn widget_type(&self) -> TypeId {
        TypeId::of::<Self>()
    }
    
    fn key(&self) -> Option<WidgetKey> {
        self.key.clone()
    }
    
    fn create_render_object(&self) -> Box<dyn RenderObject> {
        Box::new(ImageRenderObject::new(
            self.source.clone(),
            self.fit,
            self.alignment,
            self.opacity,
            self.width,
            self.height,
        ))
    }
    
    fn update_render_object(&self, render_object: &mut dyn RenderObject) {
        if let Some(image_ro) = render_object.as_any_mut().downcast_mut::<ImageRenderObject>() {
            image_ro.source = self.source.clone();
            image_ro.fit = self.fit;
            image_ro.alignment = self.alignment;
            image_ro.opacity = self.opacity;
            image_ro.explicit_width = self.width;
            image_ro.explicit_height = self.height;
            image_ro.state.mark_needs_layout();
            image_ro.state.mark_needs_paint();
        }
    }
    
    fn should_update(&self, old: &dyn Widget) -> bool {
        if let Some(old_image) = old.as_any().downcast_ref::<Image>() {
            self.source != old_image.source ||
            self.fit != old_image.fit ||
            self.alignment != old_image.alignment ||
            (self.opacity - old_image.opacity).abs() > f32::EPSILON ||
            self.width != old_image.width ||
            self.height != old_image.height
        } else {
            true
        }
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Render object for Image widget
#[derive(Debug)]
pub struct ImageRenderObject {
    state: RenderObjectState,
    source: String,
    fit: ImageFit,
    alignment: Alignment,
    opacity: f32,
    explicit_width: Option<f32>,
    explicit_height: Option<f32>,
    
    /// Cached intrinsic size (would be set when image loads)
    intrinsic_size: Size,
}

impl ImageRenderObject {
    fn new(
        source: String,
        fit: ImageFit,
        alignment: Alignment,
        opacity: f32,
        explicit_width: Option<f32>,
        explicit_height: Option<f32>,
    ) -> Self {
        Self {
            state: RenderObjectState::new(),
            source,
            fit,
            alignment,
            opacity,
            explicit_width,
            explicit_height,
            // Default intrinsic size, should be updated when image loads
            intrinsic_size: Size::new(100.0, 100.0),
        }
    }
    
    /// Calculate the destination rect with a specific intrinsic size
    fn calculate_dest_rect_with_size(&self, bounds: Rect, intrinsic_size: Size) -> Rect {
        let image_aspect = intrinsic_size.width / intrinsic_size.height;
        let bounds_aspect = bounds.width / bounds.height;
        
        match self.fit {
            ImageFit::Fill => bounds,
            
            ImageFit::Contain => {
                let (width, height) = if image_aspect > bounds_aspect {
                    // Image is wider, fit to width
                    (bounds.width, bounds.width / image_aspect)
                } else {
                    // Image is taller, fit to height
                    (bounds.height * image_aspect, bounds.height)
                };
                
                let offset = self.alignment.align_offset(bounds.size(), Size::new(width, height));
                Rect::new(bounds.x + offset.x, bounds.y + offset.y, width, height)
            }
            
            ImageFit::Cover => {
                let (width, height) = if image_aspect > bounds_aspect {
                    // Image is wider, fit to height
                    (bounds.height * image_aspect, bounds.height)
                } else {
                    // Image is taller, fit to width
                    (bounds.width, bounds.width / image_aspect)
                };
                
                let offset = self.alignment.align_offset(bounds.size(), Size::new(width, height));
                Rect::new(bounds.x + offset.x, bounds.y + offset.y, width, height)
            }
            
            ImageFit::None => {
                let offset = self.alignment.align_offset(bounds.size(), intrinsic_size);
                Rect::new(
                    bounds.x + offset.x,
                    bounds.y + offset.y,
                    intrinsic_size.width,
                    intrinsic_size.height,
                )
            }
            
            ImageFit::ScaleDown => {
                if intrinsic_size.width <= bounds.width && 
                   intrinsic_size.height <= bounds.height {
                    // Image fits, don't scale
                    let offset = self.alignment.align_offset(bounds.size(), intrinsic_size);
                    Rect::new(
                        bounds.x + offset.x,
                        bounds.y + offset.y,
                        intrinsic_size.width,
                        intrinsic_size.height,
                    )
                } else {
                    // Scale down using Contain logic
                    let (width, height) = if image_aspect > bounds_aspect {
                        (bounds.width, bounds.width / image_aspect)
                    } else {
                        (bounds.height * image_aspect, bounds.height)
                    };
                    
                    let offset = self.alignment.align_offset(bounds.size(), Size::new(width, height));
                    Rect::new(bounds.x + offset.x, bounds.y + offset.y, width, height)
                }
            }
        }
    }
}

impl RenderObject for ImageRenderObject {
    impl_render_object_common!(state);
    
    fn layout(&mut self, constraints: Constraints) -> Size {
        let width = self.explicit_width.unwrap_or(self.intrinsic_size.width);
        let height = self.explicit_height.unwrap_or(self.intrinsic_size.height);
        
        let size = constraints.constrain(Size::new(width, height));
        self.state.size = size;
        self.state.needs_layout = false;
        
        size
    }
    
    fn paint(&self, painter: &mut dyn Painter) {
        // Get actual image size from painter if available
        let actual_intrinsic_size = painter
            .get_image_size(&self.source)
            .unwrap_or(self.intrinsic_size);
        
        let rect = self.state.get_rect();
        let dest_rect = self.calculate_dest_rect_with_size(rect, actual_intrinsic_size);
        
        if (self.opacity - 1.0).abs() < f32::EPSILON {
            painter.draw_image_rect(&self.source, dest_rect);
        } else {
            painter.save();
            // Note: For proper opacity support, the painter would need alpha blending
            painter.draw_image_rect(&self.source, dest_rect);
            painter.restore();
        }
    }
}
