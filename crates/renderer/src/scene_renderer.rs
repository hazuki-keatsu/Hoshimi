//! # SceneRenderer
//!
//! A stateless rendering API that encapsulates Skia's drawing capabilities.
//!
//! ## Design Philosophy
//!
//! SceneRenderer is a **stateless** rendering tool:
//! - It does NOT manage rendering state (colors, fonts, etc.)
//! - State management is the responsibility of the Widget/Component layer
//! - Each draw call receives all necessary parameters explicitly
//! - Only manages GPU resources (Surface, Context) and caches (images, fonts)
//!
//! ## Usage
//!
//! ```ignore
//! // Widget manages its own state
//! struct MyButton {
//!     rect: UIRect,
//!     color: UIColor,
//!     text: String,
//! }
//!
//! impl Widget for MyButton {
//!     fn render(&self, renderer: &mut SceneRenderer) -> RendererResult<()> {
//!         // Pass state directly to draw calls
//!         renderer.fill_rect(self.rect, self.color)?;
//!         renderer.draw_text(&self.text, pos, 16.0, UIColor::white())?;
//!         Ok(())
//!     }
//! }
//! ```

use std::collections::HashMap;

use hoshimi_shared::logger;
use skia_safe::gpu::gl::FramebufferInfo;
use skia_safe::gpu::{Protected, SurfaceOrigin};
use skia_safe::paint::Style as PaintStyle;
use skia_safe::{
    gpu, Canvas, Color, ColorType, Data, Font, FontMgr, Image, Paint, Point, RRect, Rect, Surface,
    Typeface,
};

use crate::error::{RendererError, RendererResult};
use crate::types::{UIColor, UIPoint, UIRect, UIRoundRect, UISize};

/// SceneRenderer - A stateless rendering API
///
/// Manages GPU resources and provides drawing primitives.
/// All rendering state (colors, sizes, etc.) is passed explicitly to each draw call.
pub struct SceneRenderer {
    // Skia core resources
    context: gpu::DirectContext,
    surface: Surface,

    // Renderer dimensions
    width: i32,
    height: i32,

    // Resource caches (these are kept for performance)
    image_cache: HashMap<String, Image>,
    font_cache: HashMap<String, Typeface>,

    // Font manager
    font_mgr: FontMgr,
}

impl SceneRenderer {
    /// Create a new SceneRenderer
    ///
    /// ## Arguments
    /// * `width` - Surface width
    /// * `height` - Surface height
    pub fn new(width: i32, height: i32) -> RendererResult<Self> {
        let interface = skia_safe::gpu::gl::Interface::new_native().ok_or_else(|| {
            RendererError::InitializationFailed("Failed to create native GL interface".to_string())
        })?;

        let mut context = gpu::direct_contexts::make_gl(interface, None).ok_or_else(|| {
            RendererError::InitializationFailed("Failed to create Skia GL context".to_string())
        })?;

        let surface = Self::create_surface_internal(&mut context, width, height)?;
        let font_mgr = FontMgr::new();

        logger::info!("SceneRenderer: Initialized ({}x{})", width, height);

        Ok(Self {
            context,
            surface,
            width,
            height,
            image_cache: HashMap::new(),
            font_cache: HashMap::new(),
            font_mgr,
        })
    }

    // ==================== Properties ====================

    /// Get the renderer width
    pub fn width(&self) -> i32 {
        self.width
    }

    /// Get the renderer height
    pub fn height(&self) -> i32 {
        self.height
    }

    /// Get the renderer size
    pub fn size(&self) -> UISize {
        UISize::new(self.width as f32, self.height as f32)
    }

    /// Resize the renderer
    pub fn resize(&mut self, width: i32, height: i32) -> RendererResult<()> {
        self.surface = Self::create_surface_internal(&mut self.context, width, height)?;
        self.width = width;
        self.height = height;
        logger::debug!("SceneRenderer: Resized to {}x{}", width, height);
        Ok(())
    }

    // ==================== Frame Management ====================

    /// Begin a new frame
    ///
    /// ## Arguments
    /// * `clear_color` - Optional background color to clear with
    pub fn begin_frame(&mut self, clear_color: Option<UIColor>) -> RendererResult<()> {
        if let Some(color) = clear_color {
            self.surface.canvas().clear(Color::from(color));
        }
        Ok(())
    }

    /// End the current frame and submit to GPU
    pub fn end_frame(&mut self) -> RendererResult<()> {
        self.context.flush_and_submit();
        Ok(())
    }

    /// Get the frame pixel data (RGBA)
    pub fn get_frame_data(&mut self) -> RendererResult<Vec<u8>> {
        let image_info = self.surface.image_info();
        let row_bytes = image_info.width() as usize * 4;
        let mut pixels = vec![0u8; row_bytes * image_info.height() as usize];

        self.surface
            .read_pixels(&image_info, &mut pixels, row_bytes, (0, 0));

        Ok(pixels)
    }

    // ==================== Canvas State (for transforms/clips) ====================

    /// Save the current canvas state (transforms, clips)
    pub fn save(&mut self) {
        self.surface.canvas().save();
    }

    /// Restore the previous canvas state
    pub fn restore(&mut self) {
        self.surface.canvas().restore();
    }

    // ==================== Shape Drawing ====================

    /// Fill a rectangle with the specified color
    pub fn fill_rect(&mut self, rect: UIRect, color: UIColor) -> RendererResult<()> {
        let paint = Self::create_fill_paint(color);
        let skia_rect: Rect = rect.into();
        self.canvas().draw_rect(skia_rect, &paint);
        Ok(())
    }

    /// Stroke a rectangle outline
    pub fn stroke_rect(
        &mut self,
        rect: UIRect,
        color: UIColor,
        stroke_width: f32,
    ) -> RendererResult<()> {
        let paint = Self::create_stroke_paint(color, stroke_width);
        let skia_rect: Rect = rect.into();
        self.canvas().draw_rect(skia_rect, &paint);
        Ok(())
    }

    /// Fill a rounded rectangle
    pub fn fill_rounded_rect(
        &mut self,
        rect: UIRect,
        radius: f32,
        color: UIColor,
    ) -> RendererResult<()> {
        let paint = Self::create_fill_paint(color);
        let rrect = UIRoundRect::new(rect, radius);
        let skia_rrect: RRect = rrect.into();
        self.canvas().draw_rrect(skia_rrect, &paint);
        Ok(())
    }

    /// Stroke a rounded rectangle outline
    pub fn stroke_rounded_rect(
        &mut self,
        rect: UIRect,
        radius: f32,
        color: UIColor,
        stroke_width: f32,
    ) -> RendererResult<()> {
        let paint = Self::create_stroke_paint(color, stroke_width);
        let rrect = UIRoundRect::new(rect, radius);
        let skia_rrect: RRect = rrect.into();
        self.canvas().draw_rrect(skia_rrect, &paint);
        Ok(())
    }

    /// Fill a circle
    pub fn fill_circle(&mut self, center: UIPoint, radius: f32, color: UIColor) -> RendererResult<()> {
        let paint = Self::create_fill_paint(color);
        let skia_center: Point = center.into();
        self.canvas().draw_circle(skia_center, radius, &paint);
        Ok(())
    }

    /// Stroke a circle outline
    pub fn stroke_circle(
        &mut self,
        center: UIPoint,
        radius: f32,
        color: UIColor,
        stroke_width: f32,
    ) -> RendererResult<()> {
        let paint = Self::create_stroke_paint(color, stroke_width);
        let skia_center: Point = center.into();
        self.canvas().draw_circle(skia_center, radius, &paint);
        Ok(())
    }

    /// Fill an oval/ellipse
    pub fn fill_oval(&mut self, rect: UIRect, color: UIColor) -> RendererResult<()> {
        let paint = Self::create_fill_paint(color);
        let skia_rect: Rect = rect.into();
        self.canvas().draw_oval(skia_rect, &paint);
        Ok(())
    }

    /// Draw a line
    pub fn draw_line(
        &mut self,
        start: UIPoint,
        end: UIPoint,
        color: UIColor,
        stroke_width: f32,
    ) -> RendererResult<()> {
        let paint = Self::create_stroke_paint(color, stroke_width);
        let skia_start: Point = start.into();
        let skia_end: Point = end.into();
        self.canvas().draw_line(skia_start, skia_end, &paint);
        Ok(())
    }

    // ==================== Text Drawing ====================

    /// Draw text at the specified position
    ///
    /// ## Arguments
    /// * `text` - The text to draw
    /// * `pos` - Position (baseline origin)
    /// * `font_size` - Font size in points
    /// * `color` - Text color
    pub fn draw_text(
        &mut self,
        text: &str,
        pos: UIPoint,
        font_size: f32,
        color: UIColor,
    ) -> RendererResult<()> {
        let paint = Self::create_fill_paint(color);
        let font = Self::create_font(font_size);
        let skia_pos: Point = pos.into();
        self.canvas().draw_str(text, skia_pos, &font, &paint);
        Ok(())
    }

    /// Draw text with a custom font
    pub fn draw_text_with_font(
        &mut self,
        text: &str,
        pos: UIPoint,
        font_path: &str,
        font_size: f32,
        color: UIColor,
    ) -> RendererResult<()> {
        let paint = Self::create_fill_paint(color);
        let typeface = self.get_or_load_font(font_path)?;
        let font = Font::from_typeface(typeface, font_size);
        let skia_pos: Point = pos.into();
        self.canvas().draw_str(text, skia_pos, &font, &paint);
        Ok(())
    }

    /// Draw text centered within a rectangle
    pub fn draw_text_centered(
        &mut self,
        text: &str,
        rect: UIRect,
        font_size: f32,
        color: UIColor,
    ) -> RendererResult<()> {
        let paint = Self::create_fill_paint(color);
        let font = Self::create_font(font_size);

        let (text_width, _) = font.measure_str(text, Some(&paint));
        let metrics = font.metrics();

        let x = rect.x + (rect.width - text_width) / 2.0;
        let y = rect.y + (rect.height - metrics.0) / 2.0 - metrics.1.ascent;

        self.canvas().draw_str(text, (x, y), &font, &paint);
        Ok(())
    }

    /// Measure text dimensions
    pub fn measure_text(&self, text: &str, font_size: f32) -> UISize {
        let font = Self::create_font(font_size);
        let (width, bounds) = font.measure_str(text, None::<&Paint>);
        UISize::new(width, bounds.height())
    }

    /// Draw text with a font by cache key (for use with load_font_from_data)
    pub fn draw_text_with_font_key(
        &mut self,
        text: &str,
        pos: UIPoint,
        font_key: &str,
        font_size: f32,
        color: UIColor,
    ) -> RendererResult<()> {
        let typeface = self.font_cache.get(font_key)
            .ok_or_else(|| RendererError::FontLoadFailed(format!("Font '{}' not found in cache", font_key)))?
            .clone();
        let paint = Self::create_fill_paint(color);
        let font = Font::from_typeface(typeface, font_size);
        let skia_pos: Point = pos.into();
        self.canvas().draw_str(text, skia_pos, &font, &paint);
        Ok(())
    }

    /// Draw text centered with a font by cache key
    pub fn draw_text_centered_with_font_key(
        &mut self,
        text: &str,
        rect: UIRect,
        font_key: &str,
        font_size: f32,
        color: UIColor,
    ) -> RendererResult<()> {
        let typeface = self.font_cache.get(font_key)
            .ok_or_else(|| RendererError::FontLoadFailed(format!("Font '{}' not found in cache", font_key)))?
            .clone();
        let paint = Self::create_fill_paint(color);
        let font = Font::from_typeface(typeface, font_size);

        let (text_width, _) = font.measure_str(text, Some(&paint));
        let metrics = font.metrics();

        let x = rect.x + (rect.width - text_width) / 2.0;
        let y = rect.y + (rect.height - metrics.0) / 2.0 - metrics.1.ascent;

        self.canvas().draw_str(text, (x, y), &font, &paint);
        Ok(())
    }

    /// Measure text dimensions with a specific font by cache key
    pub fn measure_text_with_font_key(&self, text: &str, font_key: &str, font_size: f32) -> RendererResult<UISize> {
        let typeface = self.font_cache.get(font_key)
            .ok_or_else(|| RendererError::FontLoadFailed(format!("Font '{}' not found in cache", font_key)))?;
        let font = Font::from_typeface(typeface, font_size);
        let (width, bounds) = font.measure_str(text, None::<&Paint>);
        Ok(UISize::new(width, bounds.height()))
    }

    // ==================== Image Drawing ====================

    /// Draw an image at the specified position
    pub fn draw_image(&mut self, path: &str, pos: UIPoint) -> RendererResult<()> {
        let image = self.get_or_load_image(path)?;
        let skia_pos: Point = pos.into();
        self.canvas().draw_image(&image, skia_pos, None);
        Ok(())
    }

    /// Draw an image with opacity
    pub fn draw_image_with_alpha(&mut self, path: &str, pos: UIPoint, alpha: f32) -> RendererResult<()> {
        let image = self.get_or_load_image(path)?;
        let paint = Self::create_alpha_paint(alpha);
        let skia_pos: Point = pos.into();
        self.canvas().draw_image(&image, skia_pos, Some(&paint));
        Ok(())
    }

    /// Draw an image scaled to fit a rectangle
    pub fn draw_image_rect(&mut self, path: &str, dest_rect: UIRect) -> RendererResult<()> {
        let image = self.get_or_load_image(path)?;
        let skia_dest: Rect = dest_rect.into();
        self.canvas()
            .draw_image_rect(&image, None, skia_dest, &Paint::default());
        Ok(())
    }

    /// Draw a portion of an image to a destination rectangle
    pub fn draw_image_rect_src(
        &mut self,
        path: &str,
        src_rect: UIRect,
        dest_rect: UIRect,
    ) -> RendererResult<()> {
        let image = self.get_or_load_image(path)?;
        let skia_src: Rect = src_rect.into();
        let skia_dest: Rect = dest_rect.into();
        self.canvas().draw_image_rect(
            &image,
            Some((&skia_src, skia_safe::canvas::SrcRectConstraint::Strict)),
            skia_dest,
            &Paint::default(),
        );
        Ok(())
    }

    /// Draw an image by cache key (for use with load_image_from_data)
    pub fn draw_image_by_key(&mut self, key: &str, pos: UIPoint) -> RendererResult<()> {
        let image = self.image_cache.get(key)
            .ok_or_else(|| RendererError::ImageLoadFailed(format!("Image '{}' not found in cache", key)))?
            .clone();
        let skia_pos: Point = pos.into();
        self.canvas().draw_image(&image, skia_pos, None);
        Ok(())
    }

    /// Draw an image by cache key with opacity
    pub fn draw_image_by_key_with_alpha(
        &mut self,
        key: &str,
        pos: UIPoint,
        alpha: f32,
    ) -> RendererResult<()> {
        let image = self.image_cache.get(key)
            .ok_or_else(|| RendererError::ImageLoadFailed(format!("Image '{}' not found in cache", key)))?
            .clone();
        let paint = Self::create_alpha_paint(alpha);
        let skia_pos: Point = pos.into();
        self.canvas().draw_image(&image, skia_pos, Some(&paint));
        Ok(())
    }

    /// Draw an image by cache key scaled to fit a rectangle
    pub fn draw_image_by_key_rect(&mut self, key: &str, dest_rect: UIRect) -> RendererResult<()> {
        let image = self.image_cache.get(key)
            .ok_or_else(|| RendererError::ImageLoadFailed(format!("Image '{}' not found in cache", key)))?
            .clone();
        let skia_dest: Rect = dest_rect.into();
        self.canvas()
            .draw_image_rect(&image, None, skia_dest, &Paint::default());
        Ok(())
    }

    /// Draw an image by cache key with source and destination rectangles
    pub fn draw_image_by_key_rect_src(
        &mut self,
        key: &str,
        src_rect: UIRect,
        dest_rect: UIRect,
    ) -> RendererResult<()> {
        let image = self.image_cache.get(key)
            .ok_or_else(|| RendererError::ImageLoadFailed(format!("Image '{}' not found in cache", key)))?
            .clone();
        let skia_src: Rect = src_rect.into();
        let skia_dest: Rect = dest_rect.into();
        self.canvas().draw_image_rect(
            &image,
            Some((&skia_src, skia_safe::canvas::SrcRectConstraint::Strict)),
            skia_dest,
            &Paint::default(),
        );
        Ok(())
    }

    // ==================== Transforms ====================

    /// Translate the canvas
    pub fn translate(&mut self, dx: f32, dy: f32) {
        self.canvas().translate((dx, dy));
    }

    /// Rotate the canvas (degrees)
    pub fn rotate(&mut self, degrees: f32) {
        self.canvas().rotate(degrees, None);
    }

    /// Rotate around a point
    pub fn rotate_around(&mut self, degrees: f32, center: UIPoint) {
        let skia_center: Point = center.into();
        self.canvas().rotate(degrees, Some(skia_center));
    }

    /// Scale the canvas
    pub fn scale(&mut self, sx: f32, sy: f32) {
        self.canvas().scale((sx, sy));
    }

    // ==================== Clipping ====================

    /// Set a rectangular clip region
    pub fn clip_rect(&mut self, rect: UIRect) {
        let skia_rect: Rect = rect.into();
        self.canvas()
            .clip_rect(skia_rect, skia_safe::ClipOp::Intersect, true);
    }

    /// Set a rounded rectangle clip region
    pub fn clip_rounded_rect(&mut self, rect: UIRect, radius: f32) {
        let rrect = UIRoundRect::new(rect, radius);
        let skia_rrect: RRect = rrect.into();
        self.canvas()
            .clip_rrect(skia_rrect, skia_safe::ClipOp::Intersect, true);
    }

    // ==================== Resource Management ====================

    /// Preload an image from file into cache
    pub fn preload_image(&mut self, path: &str) -> RendererResult<()> {
        self.get_or_load_image(path)?;
        Ok(())
    }

    /// Preload a font from file into cache
    pub fn preload_font(&mut self, path: &str) -> RendererResult<()> {
        self.get_or_load_font(path)?;
        Ok(())
    }

    /// Load an image from memory data into cache
    ///
    /// This is useful for resource managers that handle their own file loading.
    ///
    /// ## Arguments
    /// * `key` - A unique key to identify this image in cache
    /// * `data` - The raw image data (PNG, JPG, WebP, etc.)
    pub fn load_image_from_data(&mut self, key: &str, data: &[u8]) -> RendererResult<()> {
        let skia_data = Data::new_copy(data);
        let image = Image::from_encoded(skia_data)
            .ok_or_else(|| RendererError::ImageLoadFailed(key.to_string()))?;

        self.image_cache.insert(key.to_string(), image);
        logger::debug!("SceneRenderer: Loaded image from data '{}'", key);
        Ok(())
    }

    /// Load a font from memory data into cache
    ///
    /// This is useful for resource managers that handle their own file loading.
    ///
    /// ## Arguments
    /// * `key` - A unique key to identify this font in cache
    /// * `data` - The raw font data (TTF, OTF, etc.)
    pub fn load_font_from_data(&mut self, key: &str, data: &[u8]) -> RendererResult<()> {
        let skia_data = Data::new_copy(data);
        let typeface = self
            .font_mgr
            .new_from_data(&skia_data, None)
            .ok_or_else(|| RendererError::FontLoadFailed(key.to_string()))?;

        self.font_cache.insert(key.to_string(), typeface);
        logger::debug!("SceneRenderer: Loaded font from data '{}'", key);
        Ok(())
    }

    /// Check if an image is loaded in cache
    pub fn has_image(&self, key: &str) -> bool {
        self.image_cache.contains_key(key)
    }

    /// Check if a font is loaded in cache
    pub fn has_font(&self, key: &str) -> bool {
        self.font_cache.contains_key(key)
    }

    /// Clear image cache
    pub fn clear_image_cache(&mut self) {
        self.image_cache.clear();
        logger::debug!("SceneRenderer: Image cache cleared");
    }

    /// Clear font cache
    pub fn clear_font_cache(&mut self) {
        self.font_cache.clear();
        logger::debug!("SceneRenderer: Font cache cleared");
    }

    /// Clear all caches
    pub fn clear_all_cache(&mut self) {
        self.clear_image_cache();
        self.clear_font_cache();
    }

    // ==================== Internal Methods ====================

    fn canvas(&mut self) -> &Canvas {
        self.surface.canvas()
    }

    fn create_fill_paint(color: UIColor) -> Paint {
        let mut paint = Paint::default();
        paint.set_color(Color::from(color));
        paint.set_style(PaintStyle::Fill);
        paint.set_anti_alias(true);
        paint
    }

    fn create_stroke_paint(color: UIColor, stroke_width: f32) -> Paint {
        let mut paint = Paint::default();
        paint.set_color(Color::from(color));
        paint.set_style(PaintStyle::Stroke);
        paint.set_stroke_width(stroke_width);
        paint.set_anti_alias(true);
        paint
    }

    fn create_alpha_paint(alpha: f32) -> Paint {
        let mut paint = Paint::default();
        paint.set_alpha((alpha.clamp(0.0, 1.0) * 255.0) as u8);
        paint.set_anti_alias(true);
        paint
    }

    fn create_font(size: f32) -> Font {
        Font::default().with_size(size).unwrap_or_default()
    }

    fn get_or_load_image(&mut self, path: &str) -> RendererResult<Image> {
        if let Some(image) = self.image_cache.get(path) {
            return Ok(image.clone());
        }

        let data = std::fs::read(path)?;
        let data = Data::new_copy(&data);
        let image =
            Image::from_encoded(data).ok_or_else(|| RendererError::ImageLoadFailed(path.to_string()))?;

        self.image_cache.insert(path.to_string(), image.clone());
        logger::debug!("SceneRenderer: Loaded image '{}'", path);
        Ok(image)
    }

    fn get_or_load_font(&mut self, path: &str) -> RendererResult<Typeface> {
        if let Some(typeface) = self.font_cache.get(path) {
            return Ok(typeface.clone());
        }

        let data = std::fs::read(path)?;
        let data = Data::new_copy(&data);
        let typeface = self
            .font_mgr
            .new_from_data(&data, None)
            .ok_or_else(|| RendererError::FontLoadFailed(path.to_string()))?;

        self.font_cache.insert(path.to_string(), typeface.clone());
        logger::debug!("SceneRenderer: Loaded font '{}'", path);
        Ok(typeface)
    }

    fn create_surface_internal(
        context: &mut gpu::DirectContext,
        width: i32,
        height: i32,
    ) -> RendererResult<Surface> {
        let fb_info = FramebufferInfo {
            fboid: 0,
            format: skia_safe::gpu::gl::Format::RGBA8.into(),
            protected: Protected::No,
        };

        let target = gpu::backend_render_targets::make_gl((width, height), 0, 8, fb_info);

        gpu::surfaces::wrap_backend_render_target(
            context,
            &target,
            SurfaceOrigin::BottomLeft,
            ColorType::RGBA8888,
            None,
            None,
        )
        .ok_or_else(|| {
            RendererError::SurfaceCreationFailed(format!(
                "Failed to create surface ({}x{})",
                width, height
            ))
        })
    }
}
