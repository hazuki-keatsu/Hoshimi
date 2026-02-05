//! The definition of basic UI element
//! 
//! Provide the basic geographic and color types used by renderer, 
//! encapsulate the underlying types of Skia.

use skia_safe::{Color, Point, Rect, RRect};

/// UI颜色类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UIColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl UIColor {
    /// Create UIColor without alpha channel (0-255)
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Create UIColor with alpha channel (0-255)
    pub const fn new_with_alpha(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Create UIColor from float number (0.0-1.0)
    pub fn from_f32(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            r: (r.clamp(0.0, 1.0) * 255.0) as u8,
            g: (g.clamp(0.0, 1.0) * 255.0) as u8,
            b: (b.clamp(0.0, 1.0) * 255.0) as u8,
            a: (a.clamp(0.0, 1.0) * 255.0) as u8,
        }
    }

    /// Create UIColor from hex code without alpha channel
    pub const fn from_hex(hex: u32) -> Self {
        Self {
            r: ((hex >> 16) & 0xFF) as u8,
            g: ((hex >> 8) & 0xFF) as u8,
            b: (hex & 0xFF) as u8,
            a: 255,
        }
    }

    /// Create UIColor from hex code with alpha channel
    pub const fn from_hex_with_alpha(hex: u32) -> Self {
        Self {
            r: ((hex >> 24) & 0xFF) as u8,
            g: ((hex >> 16) & 0xFF) as u8,
            b: ((hex >> 8) & 0xFF) as u8,
            a: (hex & 0xFF) as u8,
        }
    }

    // Pre-defined UIColor
    pub const fn black() -> Self { Self::new(0, 0, 0) }
    pub const fn white() -> Self { Self::new(255, 255, 255) }
    pub const fn red() -> Self { Self::new(255, 0, 0) }
    pub const fn green() -> Self { Self::new(0, 255, 0) }
    pub const fn blue() -> Self { Self::new(0, 0, 255) }
    pub const fn transparent() -> Self { Self::new_with_alpha(0, 0, 0, 0) }

    /// Set the alpha channel for the UIColor
    pub const fn with_alpha(self, a: u8) -> Self {
        Self { a, ..self }
    }
}

// Transfer from UIColor to Skia Color
impl From<UIColor> for Color {
    fn from(c: UIColor) -> Self {
        Color::from_argb(c.a, c.r, c.g, c.b)
    }
}

// Transfer from Skia Color to UIColor
impl From<Color> for UIColor {
    fn from(c: Color) -> Self {
        Self {
            r: c.r(),
            g: c.g(),
            b: c.b(),
            a: c.a(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct UIPoint {
    pub x: f32,
    pub y: f32,
}

impl UIPoint {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub const fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    pub fn offset(&self, dx: f32, dy: f32) -> Self {
        Self { x: self.x + dx, y: self.y + dy }
    }
}

impl From<UIPoint> for Point {
    fn from(p: UIPoint) -> Self {
        Point::new(p.x, p.y)
    }
}

impl From<Point> for UIPoint {
    fn from(p: Point) -> Self {
        Self { x: p.x, y: p.y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct UISize {
    pub width: f32,
    pub height: f32,
}

impl UISize {
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    pub const fn zero() -> Self {
        Self { width: 0.0, height: 0.0 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct UIRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl UIRect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    pub const fn from_xywh(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    pub fn from_point_size(point: UIPoint, size: UISize) -> Self {
        Self {
            x: point.x,
            y: point.y,
            width: size.width,
            height: size.height,
        }
    }

    pub fn from_ltrb(left: f32, top: f32, right: f32, bottom: f32) -> Self {
        Self {
            x: left,
            y: top,
            width: right - left,
            height: bottom - top,
        }
    }

    pub const fn zero() -> Self {
        Self { x: 0.0, y: 0.0, width: 0.0, height: 0.0 }
    }

    // getter for UIRect
    pub fn left(&self) -> f32 { self.x }
    pub fn top(&self) -> f32 { self.y }
    pub fn right(&self) -> f32 { self.x + self.width }
    pub fn bottom(&self) -> f32 { self.y + self.height }
    pub fn center(&self) -> UIPoint { 
        UIPoint::new(self.x + self.width / 2.0, self.y + self.height / 2.0)
    }
    pub fn origin(&self) -> UIPoint { UIPoint::new(self.x, self.y) }
    pub fn size(&self) -> UISize { UISize::new(self.width, self.height) }

    /// Check whether the UIPoint is in the UIRect
    pub fn contains(&self, point: UIPoint) -> bool {
        point.x >= self.x && point.x <= self.right() &&
        point.y >= self.y && point.y <= self.bottom()
    }

    /// Offset UIRect
    pub fn offset(&self, dx: f32, dy: f32) -> Self {
        Self { x: self.x + dx, y: self.y + dy, ..*self }
    }

    /// Inset UIRect
    pub fn inset(&self, dx: f32, dy: f32) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
            width: self.width - 2.0 * dx,
            height: self.height - 2.0 * dy,
        }
    }
}

impl From<UIRect> for Rect {
    fn from(r: UIRect) -> Self {
        Rect::from_xywh(r.x, r.y, r.width, r.height)
    }
}

impl From<Rect> for UIRect {
    fn from(r: Rect) -> Self {
        Self {
            x: r.left,
            y: r.top,
            width: r.width(),
            height: r.height(),
        }
    }
}

/// Rounded UIRect
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UIRoundRect {
    pub rect: UIRect,
    pub radius: f32,
}

impl UIRoundRect {
    pub const fn new(rect: UIRect, radius: f32) -> Self {
        Self { rect, radius }
    }

    pub fn from_xywh(x: f32, y: f32, width: f32, height: f32, radius: f32) -> Self {
        Self {
            rect: UIRect::from_xywh(x, y, width, height),
            radius,
        }
    }
}

impl From<UIRoundRect> for RRect {
    fn from(r: UIRoundRect) -> Self {
        RRect::new_rect_xy(Rect::from(r.rect), r.radius, r.radius)
    }
}

#[derive(Debug, Clone)]
pub struct UIFont {
    pub path: String,
    pub size: f32,
}

impl UIFont {
    pub fn new(path: impl Into<String>, size: f32) -> Self {
        Self {
            path: path.into(),
            size,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
}

/// Paint style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PaintStyle {
    #[default]
    Fill,
    Stroke,
    StrokeAndFill,
}

impl From<PaintStyle> for skia_safe::paint::Style {
    fn from(style: PaintStyle) -> Self {
        match style {
            PaintStyle::Fill => skia_safe::paint::Style::Fill,
            PaintStyle::Stroke => skia_safe::paint::Style::Stroke,
            PaintStyle::StrokeAndFill => skia_safe::paint::Style::StrokeAndFill,
        }
    }
}
