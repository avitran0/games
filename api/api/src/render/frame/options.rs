use glam::Vec2;

use crate::{Color, Font};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Anchor {
    TopLeft,
    Top,
    TopRight,
    Left,
    Center,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

impl Anchor {
    pub(crate) fn offset(self, size: Vec2) -> Vec2 {
        match self {
            Self::TopLeft => Vec2::ZERO,
            Self::Top => Vec2::new(size.x * 0.5, 0.0),
            Self::TopRight => Vec2::new(size.x, 0.0),
            Self::Left => Vec2::new(0.0, size.y * 0.5),
            Self::Center => size * 0.5,
            Self::Right => Vec2::new(size.x, size.y * 0.5),
            Self::BottomLeft => Vec2::new(0.0, size.y),
            Self::Bottom => Vec2::new(size.x * 0.5, size.y),
            Self::BottomRight => size,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flip {
    None,
    Horizontal,
    Vertical,
    Both,
}

impl Flip {
    pub(crate) fn vector(self) -> Vec2 {
        match self {
            Self::None => Vec2::NEG_ONE,
            Self::Horizontal => Vec2::X + Vec2::NEG_Y,
            Self::Vertical => Vec2::NEG_X + Vec2::Y,
            Self::Both => Vec2::ONE,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SpriteOptions {
    pub(super) anchor: Anchor,
    pub(super) rotation: f32,
    pub(super) rotation_anchor: Anchor,
    pub(super) flip: Flip,
    pub(super) scale: Vec2,
    pub(super) opacity: f32,
}

impl Default for SpriteOptions {
    fn default() -> Self {
        Self {
            anchor: Anchor::TopLeft,
            rotation: 0.0,
            rotation_anchor: Anchor::TopLeft,
            flip: Flip::None,
            scale: Vec2::ONE,
            opacity: 1.0,
        }
    }
}

impl SpriteOptions {
    #[must_use]
    pub(super) fn scaled(mut self, scale: f32) -> Self {
        self.scale *= Vec2::splat(scale);
        self
    }

    #[must_use]
    pub fn anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }

    #[must_use]
    pub fn rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }

    #[must_use]
    pub fn rotation_anchor(mut self, anchor: Anchor) -> Self {
        self.rotation_anchor = anchor;
        self
    }

    #[must_use]
    pub fn flip(mut self, flip: Flip) -> Self {
        self.flip = flip;
        self
    }

    #[must_use]
    pub fn scale(mut self, scale: Vec2) -> Self {
        self.scale = scale.max(Vec2::ZERO);
        self
    }

    #[must_use]
    pub fn uniform_scale(self, scale: f32) -> Self {
        self.scale(Vec2::splat(scale))
    }

    #[must_use]
    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TilemapOptions {
    pub(super) scale: Vec2,
    pub(super) opacity: f32,
}

impl TilemapOptions {
    #[must_use]
    pub(super) fn scaled(mut self, scale: f32) -> Self {
        self.scale *= Vec2::splat(scale);
        self
    }

    #[must_use]
    pub fn scale(mut self, scale: Vec2) -> Self {
        self.scale = scale;
        self
    }

    #[must_use]
    pub fn uniform_scale(self, scale: f32) -> Self {
        self.scale(Vec2::splat(scale))
    }

    #[must_use]
    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }
}

impl Default for TilemapOptions {
    fn default() -> Self {
        Self {
            scale: Vec2::ONE,
            opacity: 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ShapeOptions {
    pub(super) radius: f32,
    pub(super) border_color: Color,
    pub(super) border_width: f32,
    pub(super) opacity: f32,
}

impl ShapeOptions {
    #[must_use]
    pub(super) fn scaled(mut self, scale: f32) -> Self {
        self.radius *= scale;
        self.border_width *= scale;
        self
    }

    #[must_use]
    pub fn radius(mut self, radius: u32) -> Self {
        self.radius = radius as f32;
        self
    }

    #[must_use]
    pub fn border(mut self, color: Color, width: u32) -> Self {
        self.border_color = color;
        self.border_width = width as f32;
        self
    }

    #[must_use]
    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }
}

impl Default for ShapeOptions {
    fn default() -> Self {
        Self {
            radius: 0.0,
            border_color: Color::Transparent,
            border_width: 0.0,
            opacity: 1.0,
        }
    }
}

#[derive(Clone, Copy)]
pub struct TextOptions {
    pub(super) font: Option<Font>,
    pub(super) anchor: Anchor,
    pub(super) color: Color,
    pub(super) scale: Vec2,
    pub(super) opacity: f32,
}

impl TextOptions {
    #[must_use]
    pub(super) fn scaled(mut self, scale: f32) -> Self {
        self.scale *= Vec2::splat(scale);
        self
    }

    #[must_use]
    pub fn font(mut self, font: Font) -> Self {
        self.font = Some(font);
        self
    }

    #[must_use]
    pub fn anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }

    #[must_use]
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    #[must_use]
    pub fn scale(mut self, scale: Vec2) -> Self {
        self.scale = scale.max(Vec2::ZERO);
        self
    }

    #[must_use]
    pub fn uniform_scale(self, scale: f32) -> Self {
        self.scale(Vec2::splat(scale))
    }

    #[must_use]
    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }
}

impl Default for TextOptions {
    fn default() -> Self {
        Self {
            font: None,
            anchor: Anchor::TopLeft,
            color: Color::Mischka,
            scale: Vec2::ONE,
            opacity: 1.0,
        }
    }
}
