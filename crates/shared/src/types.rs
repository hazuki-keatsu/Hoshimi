//! Common types shared across Hoshimi engine modules
//!
//! This module provides basic geometric and color types used by both
//! the renderer and UI systems.

use std::ops::{Add, Sub, Mul};

// ============================================================================
// Skia type conversions (when skia feature is enabled)
// ============================================================================

#[cfg(feature = "skia")]
mod skia_conversions {
    use super::*;
    use skia_safe::{Color as SkiaColor, Point as SkiaPoint, Rect as SkiaRect, RRect as SkiaRRect};

    impl From<Color> for SkiaColor {
        fn from(c: Color) -> Self {
            let [r, g, b, a] = c.to_rgba8();
            SkiaColor::from_argb(a, r, g, b)
        }
    }

    impl From<SkiaColor> for Color {
        fn from(c: SkiaColor) -> Self {
            Color::from_rgba8(c.r(), c.g(), c.b(), c.a())
        }
    }

    impl From<Offset> for SkiaPoint {
        fn from(p: Offset) -> Self {
            SkiaPoint::new(p.x, p.y)
        }
    }

    impl From<SkiaPoint> for Offset {
        fn from(p: SkiaPoint) -> Self {
            Self { x: p.x, y: p.y }
        }
    }

    impl From<Rect> for SkiaRect {
        fn from(r: Rect) -> Self {
            SkiaRect::from_xywh(r.x, r.y, r.width, r.height)
        }
    }

    impl From<SkiaRect> for Rect {
        fn from(r: SkiaRect) -> Self {
            Self {
                x: r.left,
                y: r.top,
                width: r.width(),
                height: r.height(),
            }
        }
    }

    /// Create a Skia RRect from a Rect and BorderRadius
    pub fn rect_to_rrect(rect: Rect, radius: BorderRadius) -> SkiaRRect {
        if radius.is_uniform() {
            let r = radius.top_left;
            SkiaRRect::new_rect_xy(SkiaRect::from(rect), r, r)
        } else {
            let radii = [
                skia_safe::Vector::new(radius.top_left, radius.top_left),
                skia_safe::Vector::new(radius.top_right, radius.top_right),
                skia_safe::Vector::new(radius.bottom_right, radius.bottom_right),
                skia_safe::Vector::new(radius.bottom_left, radius.bottom_left),
            ];
            SkiaRRect::new_rect_radii(SkiaRect::from(rect), &radii)
        }
    }
    
    /// Create a Skia RRect from a Rect with uniform radius
    pub fn rect_to_rrect_uniform(rect: Rect, radius: f32) -> SkiaRRect {
        SkiaRRect::new_rect_xy(SkiaRect::from(rect), radius, radius)
    }
}

#[cfg(feature = "skia")]
pub use skia_conversions::*;

// ============================================================================
// Geometric Types
// ============================================================================

/// A 2D offset/point representing a position in space
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Offset {
    pub x: f32,
    pub y: f32,
}

impl Offset {
    /// Zero offset constant
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    
    /// Create a new offset
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Zero offset (origin)
    pub const fn zero() -> Self {
        Self::ZERO
    }

    /// Create an offset with same value for both components
    pub const fn uniform(value: f32) -> Self {
        Self { x: value, y: value }
    }

    /// Calculate the distance from origin
    pub fn distance(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// Calculate the distance to another offset
    pub fn distance_to(&self, other: Offset) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

impl Add for Offset {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub for Offset {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Mul<f32> for Offset {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

/// A 2D size representing dimensions
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    /// Zero size constant
    pub const ZERO: Self = Self { width: 0.0, height: 0.0 };
    
    /// Create a new size
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    /// Zero size
    pub const fn zero() -> Self {
        Self::ZERO
    }

    /// Create a size with same value for both dimensions
    pub const fn square(value: f32) -> Self {
        Self { width: value, height: value }
    }

    /// Infinite size (for unconstrained layouts)
    pub const fn infinite() -> Self {
        Self { width: f32::INFINITY, height: f32::INFINITY }
    }

    /// Check if either dimension is infinite
    pub fn is_infinite(&self) -> bool {
        self.width.is_infinite() || self.height.is_infinite()
    }

    /// Check if either dimension is zero or negative
    pub fn is_empty(&self) -> bool {
        self.width <= 0.0 || self.height <= 0.0
    }

    /// Calculate the area
    pub fn area(&self) -> f32 {
        self.width * self.height
    }

    /// Create a size that contains both sizes
    pub fn union(&self, other: Size) -> Size {
        Size::new(
            self.width.max(other.width),
            self.height.max(other.height),
        )
    }

    /// Create a size that fits within both sizes
    pub fn intersect(&self, other: Size) -> Size {
        Size::new(
            self.width.min(other.width),
            self.height.min(other.height),
        )
    }
}

/// A rectangle defined by position and size
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    /// Zero rectangle constant
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, width: 0.0, height: 0.0 };
    
    /// Create a new rectangle from position and size
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    /// Create a rectangle from x, y, width, height
    pub const fn from_xywh(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }
    
    /// Create a rectangle from origin (0, 0) with given size
    pub const fn from_size(size: Size) -> Self {
        Self { x: 0.0, y: 0.0, width: size.width, height: size.height }
    }

    /// Create a rectangle from an offset and size
    pub fn from_offset_size(offset: Offset, size: Size) -> Self {
        Self {
            x: offset.x,
            y: offset.y,
            width: size.width,
            height: size.height,
        }
    }

    /// Create a rectangle from left, top, right, bottom coordinates
    pub fn from_ltrb(left: f32, top: f32, right: f32, bottom: f32) -> Self {
        Self {
            x: left,
            y: top,
            width: right - left,
            height: bottom - top,
        }
    }

    /// Zero rectangle
    pub const fn zero() -> Self {
        Self::ZERO
    }

    /// Get left edge
    pub fn left(&self) -> f32 { self.x }
    
    /// Get top edge
    pub fn top(&self) -> f32 { self.y }
    
    /// Get right edge
    pub fn right(&self) -> f32 { self.x + self.width }
    
    /// Get bottom edge
    pub fn bottom(&self) -> f32 { self.y + self.height }

    /// Get center point
    pub fn center(&self) -> Offset {
        Offset::new(self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    /// Get top-left corner as offset
    pub fn origin(&self) -> Offset {
        Offset::new(self.x, self.y)
    }

    /// Get size
    pub fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }

    /// Check if a point is inside the rectangle
    pub fn contains(&self, point: Offset) -> bool {
        point.x >= self.x && point.x < self.right() &&
        point.y >= self.y && point.y < self.bottom()
    }

    /// Check if this rectangle overlaps with another
    pub fn intersects(&self, other: &Rect) -> bool {
        self.left() < other.right() && self.right() > other.left() &&
        self.top() < other.bottom() && self.bottom() > other.top()
    }

    /// Create a new rectangle offset by the given amount
    pub fn translate(&self, offset: Offset) -> Self {
        Self {
            x: self.x + offset.x,
            y: self.y + offset.y,
            ..*self
        }
    }

    /// Create a new rectangle with insets applied
    pub fn inset(&self, insets: EdgeInsets) -> Self {
        Self {
            x: self.x + insets.left,
            y: self.y + insets.top,
            width: self.width - insets.left - insets.right,
            height: self.height - insets.top - insets.bottom,
        }
    }

    /// Create a new rectangle expanded by the given insets
    pub fn expand(&self, insets: EdgeInsets) -> Self {
        Self {
            x: self.x - insets.left,
            y: self.y - insets.top,
            width: self.width + insets.left + insets.right,
            height: self.height + insets.top + insets.bottom,
        }
    }
}

/// Edge insets (padding/margin)
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct EdgeInsets {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl EdgeInsets {
    /// Create new edge insets
    pub const fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self { top, right, bottom, left }
    }

    /// Create uniform edge insets
    pub const fn all(value: f32) -> Self {
        Self { top: value, right: value, bottom: value, left: value }
    }

    /// Create symmetric edge insets
    pub const fn symmetric(vertical: f32, horizontal: f32) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    /// Create edge insets with only horizontal values
    pub const fn horizontal(value: f32) -> Self {
        Self { top: 0.0, right: value, bottom: 0.0, left: value }
    }

    /// Create edge insets with only vertical values
    pub const fn vertical(value: f32) -> Self {
        Self { top: value, right: 0.0, bottom: value, left: 0.0 }
    }

    /// Zero edge insets
    pub const fn zero() -> Self {
        Self { top: 0.0, right: 0.0, bottom: 0.0, left: 0.0 }
    }

    /// Get total horizontal inset
    pub fn horizontal_total(&self) -> f32 {
        self.left + self.right
    }

    /// Get total vertical inset
    pub fn vertical_total(&self) -> f32 {
        self.top + self.bottom
    }

    /// Get the total size consumed by insets
    pub fn total_size(&self) -> Size {
        Size::new(self.horizontal_total(), self.vertical_total())
    }
}

// ============================================================================
// Color Types
// ============================================================================

/// RGBA Color with components in 0.0-1.0 range
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Default for Color {
    fn default() -> Self {
        Self::BLACK
    }
}

impl Color {
    /// Predefined color constants
    pub const BLACK: Self = Self::rgb(0.0, 0.0, 0.0);
    pub const WHITE: Self = Self::rgb(1.0, 1.0, 1.0);
    pub const RED: Self = Self::rgb(1.0, 0.0, 0.0);
    pub const GREEN: Self = Self::rgb(0.0, 1.0, 0.0);
    pub const BLUE: Self = Self::rgb(0.0, 0.0, 1.0);
    pub const TRANSPARENT: Self = Self::new(0.0, 0.0, 0.0, 0.0);
    
    /// Create a new color from RGBA components (0.0-1.0)
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Create a color from RGB components (0.0-1.0), alpha = 1.0
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    /// Create a color from RGBA u8 values (0-255)
    pub fn from_rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }

    /// Create a color from RGB u8 values (0-255), alpha = 255
    pub fn from_rgb8(r: u8, g: u8, b: u8) -> Self {
        Self::from_rgba8(r, g, b, 255)
    }

    /// Create a color from hex value (0xRRGGBB)
    pub fn from_hex(hex: u32) -> Self {
        Self::from_rgb8(
            ((hex >> 16) & 0xFF) as u8,
            ((hex >> 8) & 0xFF) as u8,
            (hex & 0xFF) as u8,
        )
    }

    /// Create a color from hex value with alpha (0xRRGGBBAA)
    pub fn from_hex_with_alpha(hex: u32) -> Self {
        Self::from_rgba8(
            ((hex >> 24) & 0xFF) as u8,
            ((hex >> 16) & 0xFF) as u8,
            ((hex >> 8) & 0xFF) as u8,
            (hex & 0xFF) as u8,
        )
    }

    /// Get color with different alpha
    pub const fn with_alpha(self, a: f32) -> Self {
        Self { a, ..self }
    }

    /// Convert to RGBA u8 array
    pub fn to_rgba8(&self) -> [u8; 4] {
        [
            (self.r * 255.0).clamp(0.0, 255.0) as u8,
            (self.g * 255.0).clamp(0.0, 255.0) as u8,
            (self.b * 255.0).clamp(0.0, 255.0) as u8,
            (self.a * 255.0).clamp(0.0, 255.0) as u8,
        ]
    }

    // Predefined colors (function versions for compatibility)
    pub const fn black() -> Self { Self::BLACK }
    pub const fn white() -> Self { Self::WHITE }
    pub const fn red() -> Self { Self::RED }
    pub const fn green() -> Self { Self::GREEN }
    pub const fn blue() -> Self { Self::BLUE }
    pub const fn yellow() -> Self { Self::rgb(1.0, 1.0, 0.0) }
    pub const fn cyan() -> Self { Self::rgb(0.0, 1.0, 1.0) }
    pub const fn magenta() -> Self { Self::rgb(1.0, 0.0, 1.0) }
    pub const fn transparent() -> Self { Self::TRANSPARENT }
    pub const fn gray() -> Self { Self::rgb(0.5, 0.5, 0.5) }
    pub const fn light_gray() -> Self { Self::rgb(0.75, 0.75, 0.75) }
    pub const fn dark_gray() -> Self { Self::rgb(0.25, 0.25, 0.25) }
}

// ============================================================================
// Layout Types
// ============================================================================

/// Layout constraints for widgets
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Constraints {
    pub min_width: f32,
    pub max_width: f32,
    pub min_height: f32,
    pub max_height: f32,
}

impl Default for Constraints {
    fn default() -> Self {
        Self::loose(Size::infinite())
    }
}

impl Constraints {
    /// Create new constraints
    pub const fn new(min_width: f32, max_width: f32, min_height: f32, max_height: f32) -> Self {
        Self { min_width, max_width, min_height, max_height }
    }

    /// Create tight constraints (exact size)
    pub fn tight(size: Size) -> Self {
        Self {
            min_width: size.width,
            max_width: size.width,
            min_height: size.height,
            max_height: size.height,
        }
    }

    /// Create tight constraints for width only
    pub fn tight_width(width: f32) -> Self {
        Self {
            min_width: width,
            max_width: width,
            min_height: 0.0,
            max_height: f32::INFINITY,
        }
    }

    /// Create tight constraints for height only
    pub fn tight_height(height: f32) -> Self {
        Self {
            min_width: 0.0,
            max_width: f32::INFINITY,
            min_height: height,
            max_height: height,
        }
    }

    /// Create loose constraints (zero minimum)
    pub fn loose(size: Size) -> Self {
        Self {
            min_width: 0.0,
            max_width: size.width,
            min_height: 0.0,
            max_height: size.height,
        }
    }

    /// Create unbounded constraints
    pub fn unbounded() -> Self {
        Self {
            min_width: 0.0,
            max_width: f32::INFINITY,
            min_height: 0.0,
            max_height: f32::INFINITY,
        }
    }

    /// Check if constraints are tight (exact size)
    pub fn is_tight(&self) -> bool {
        self.min_width == self.max_width && self.min_height == self.max_height
    }

    /// Get the tight size if constraints are tight
    pub fn tight_size(&self) -> Option<Size> {
        if self.is_tight() {
            Some(Size::new(self.min_width, self.min_height))
        } else {
            None
        }
    }

    /// Constrain a size to fit within these constraints
    pub fn constrain(&self, size: Size) -> Size {
        Size::new(
            size.width.clamp(self.min_width, self.max_width),
            size.height.clamp(self.min_height, self.max_height),
        )
    }

    /// Get the smallest size that satisfies these constraints
    pub fn smallest(&self) -> Size {
        Size::new(self.min_width, self.min_height)
    }

    /// Get the largest size that satisfies these constraints
    pub fn biggest(&self) -> Size {
        Size::new(self.max_width, self.max_height)
    }

    /// Create constraints with a different max width
    pub fn with_max_width(self, max_width: f32) -> Self {
        Self { max_width, ..self }
    }

    /// Create constraints with a different max height
    pub fn with_max_height(self, max_height: f32) -> Self {
        Self { max_height, ..self }
    }

    /// Deflate constraints by the given insets
    pub fn deflate(&self, insets: EdgeInsets) -> Self {
        let horizontal = insets.horizontal_total();
        let vertical = insets.vertical_total();
        Self {
            min_width: (self.min_width - horizontal).max(0.0),
            max_width: (self.max_width - horizontal).max(0.0),
            min_height: (self.min_height - vertical).max(0.0),
            max_height: (self.max_height - vertical).max(0.0),
        }
    }
    
    /// Create loose constraints (keeps max, sets min to 0)
    pub fn loosen(&self) -> Self {
        Self {
            min_width: 0.0,
            max_width: self.max_width,
            min_height: 0.0,
            max_height: self.max_height,
        }
    }
}

/// Alignment in 2D space
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Alignment {
    /// X alignment: -1.0 = left, 0.0 = center, 1.0 = right
    pub x: f32,
    /// Y alignment: -1.0 = top, 0.0 = center, 1.0 = bottom
    pub y: f32,
}

impl Alignment {
    /// Predefined alignment constants
    pub const TOP_LEFT: Self = Self { x: -1.0, y: -1.0 };
    pub const TOP_CENTER: Self = Self { x: 0.0, y: -1.0 };
    pub const TOP_RIGHT: Self = Self { x: 1.0, y: -1.0 };
    pub const CENTER_LEFT: Self = Self { x: -1.0, y: 0.0 };
    pub const CENTER: Self = Self { x: 0.0, y: 0.0 };
    pub const CENTER_RIGHT: Self = Self { x: 1.0, y: 0.0 };
    pub const BOTTOM_LEFT: Self = Self { x: -1.0, y: 1.0 };
    pub const BOTTOM_CENTER: Self = Self { x: 0.0, y: 1.0 };
    pub const BOTTOM_RIGHT: Self = Self { x: 1.0, y: 1.0 };
    
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    // Predefined alignments (function versions for compatibility)
    pub const fn top_left() -> Self { Self::TOP_LEFT }
    pub const fn top_center() -> Self { Self::TOP_CENTER }
    pub const fn top_right() -> Self { Self::TOP_RIGHT }
    pub const fn center_left() -> Self { Self::CENTER_LEFT }
    pub const fn center() -> Self { Self::CENTER }
    pub const fn center_right() -> Self { Self::CENTER_RIGHT }
    pub const fn bottom_left() -> Self { Self::BOTTOM_LEFT }
    pub const fn bottom_center() -> Self { Self::BOTTOM_CENTER }
    pub const fn bottom_right() -> Self { Self::BOTTOM_RIGHT }

    /// Calculate the offset for aligning a child within a parent
    pub fn align_offset(&self, parent_size: Size, child_size: Size) -> Offset {
        let excess_width = parent_size.width - child_size.width;
        let excess_height = parent_size.height - child_size.height;
        Offset::new(
            excess_width * (self.x + 1.0) / 2.0,
            excess_height * (self.y + 1.0) / 2.0,
        )
    }
    
    /// Calculate the offset for aligning a child within a parent (alias for align_offset)
    pub fn align(&self, child_size: Size, parent_size: Size) -> Offset {
        self.align_offset(parent_size, child_size)
    }
}

/// Main axis alignment for flex layouts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MainAxisAlignment {
    #[default]
    Start,
    End,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

/// Cross axis alignment for flex layouts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CrossAxisAlignment {
    #[default]
    Start,
    End,
    Center,
    Stretch,
}

/// Main axis size behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MainAxisSize {
    /// Take minimum space needed
    Min,
    /// Expand to fill available space
    #[default]
    Max,
}

/// Text alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
    Justify,
}

/// Text overflow behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextOverflow {
    #[default]
    Clip,
    Ellipsis,
    Fade,
}

/// Image fit mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ImageFit {
    /// Fill the entire area, may distort aspect ratio
    Fill,
    /// Scale to fit within area, maintain aspect ratio
    #[default]
    Contain,
    /// Scale to cover entire area, maintain aspect ratio
    Cover,
    /// No scaling
    None,
    /// Scale down only if needed
    ScaleDown,
}

// ============================================================================
// Style Types
// ============================================================================

/// Text style configuration
#[derive(Debug, Clone, PartialEq)]
pub struct TextStyle {
    pub font_family: Option<String>,
    pub font_size: f32,
    pub color: Color,
    pub font_weight: FontWeight,
    pub font_style: FontStyle,
    pub letter_spacing: f32,
    pub line_height: Option<f32>,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_family: None,
            font_size: 14.0,
            color: Color::black(),
            font_weight: FontWeight::Normal,
            font_style: FontStyle::Normal,
            letter_spacing: 0.0,
            line_height: None,
        }
    }
}

impl TextStyle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn with_font_family(mut self, family: impl Into<String>) -> Self {
        self.font_family = Some(family.into());
        self
    }

    pub fn with_weight(mut self, weight: FontWeight) -> Self {
        self.font_weight = weight;
        self
    }
}

/// Font weight
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FontWeight {
    Thin,
    ExtraLight,
    Light,
    #[default]
    Normal,
    Medium,
    SemiBold,
    Bold,
    ExtraBold,
    Black,
}

impl FontWeight {
    /// Get numeric weight value (100-900)
    pub fn value(&self) -> u16 {
        match self {
            FontWeight::Thin => 100,
            FontWeight::ExtraLight => 200,
            FontWeight::Light => 300,
            FontWeight::Normal => 400,
            FontWeight::Medium => 500,
            FontWeight::SemiBold => 600,
            FontWeight::Bold => 700,
            FontWeight::ExtraBold => 800,
            FontWeight::Black => 900,
        }
    }
}

/// Font style (normal or italic)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FontStyle {
    #[default]
    Normal,
    Italic,
}

/// Box decoration for containers
#[derive(Debug, Clone, PartialEq, Default)]
pub struct BoxDecoration {
    pub color: Option<Color>,
    pub border: Option<Border>,
    pub border_radius: Option<BorderRadius>,
    pub box_shadow: Option<BoxShadow>,
}

impl BoxDecoration {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn with_border(mut self, border: Border) -> Self {
        self.border = Some(border);
        self
    }

    pub fn with_border_radius(mut self, radius: BorderRadius) -> Self {
        self.border_radius = Some(radius);
        self
    }

    pub fn with_shadow(mut self, shadow: BoxShadow) -> Self {
        self.box_shadow = Some(shadow);
        self
    }
}

/// Border configuration
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Border {
    pub color: Color,
    pub width: f32,
}

impl Border {
    pub const fn new(color: Color, width: f32) -> Self {
        Self { color, width }
    }
}

/// Border radius configuration
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct BorderRadius {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_right: f32,
    pub bottom_left: f32,
}

impl BorderRadius {
    /// Zero border radius constant
    pub const ZERO: Self = Self {
        top_left: 0.0,
        top_right: 0.0,
        bottom_right: 0.0,
        bottom_left: 0.0,
    };
    
    pub const fn all(radius: f32) -> Self {
        Self {
            top_left: radius,
            top_right: radius,
            bottom_right: radius,
            bottom_left: radius,
        }
    }

    pub const fn new(top_left: f32, top_right: f32, bottom_right: f32, bottom_left: f32) -> Self {
        Self { top_left, top_right, bottom_right, bottom_left }
    }

    pub const fn zero() -> Self {
        Self::ZERO
    }

    /// Check if all corners have the same radius
    pub fn is_uniform(&self) -> bool {
        self.top_left == self.top_right &&
        self.top_right == self.bottom_right &&
        self.bottom_right == self.bottom_left
    }

    /// Get uniform radius if all corners are the same
    pub fn uniform_radius(&self) -> Option<f32> {
        if self.is_uniform() {
            Some(self.top_left)
        } else {
            None
        }
    }
}

/// Box shadow configuration
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoxShadow {
    pub color: Color,
    pub offset: Offset,
    pub blur_radius: f32,
    pub spread_radius: f32,
}

impl BoxShadow {
    pub fn new(color: Color, offset: Offset, blur_radius: f32, spread_radius: f32) -> Self {
        Self { color, offset, blur_radius, spread_radius }
    }
}
