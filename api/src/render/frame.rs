use glam::{IVec2, UVec2, Vec2, vec2};

use crate::{
    AnimatedSprite, Color, Font, Sprite, Tilemap,
    render::draw_cmd::{DrawCmd, ShapeCmd, SpriteCmd, TextCmd, TilemapCmd},
};

#[derive(Default)]
pub struct Frame {
    draw_cmds: Vec<DrawCmd>,
}

impl Frame {
    pub(crate) fn add(&mut self, cmd: DrawCmd) {
        self.draw_cmds.push(cmd);
    }

    pub(crate) fn cmds(&self) -> &[DrawCmd] {
        &self.draw_cmds
    }

    pub(crate) fn clear(&mut self) {
        self.draw_cmds.clear();
    }

    pub fn sprite(&mut self, sprite: &Sprite, position: IVec2) {
        self.sprite_flip(sprite, position, Flip::None);
    }

    pub fn tilemap(&mut self, tilemap: &Tilemap, position: IVec2) {
        self.add(DrawCmd::Tilemap(TilemapCmd {
            tilemap: tilemap.id,
            position,
        }));
    }

    /// Draws a filled rectangle in logical screen pixels. Its sides are horizontal and vertical.
    pub fn rectangle(&mut self, position: IVec2, size: UVec2, color: Color) {
        self.shape(
            position,
            size.as_vec2(),
            color,
            0.0,
            Color::Transparent,
            0.0,
        );
    }

    /// Draws a filled rectangle with rounded corners.
    pub fn rounded_rectangle(&mut self, position: IVec2, size: UVec2, radius: u32, color: Color) {
        self.shape(
            position,
            size.as_vec2(),
            color,
            radius as f32,
            Color::Transparent,
            0.0,
        );
    }

    /// Draws a rounded rectangle with an inset border.
    pub fn bordered_rectangle(
        &mut self,
        position: IVec2,
        size: UVec2,
        radius: u32,
        color: Color,
        border_color: Color,
        border_width: u32,
    ) {
        self.shape(
            position,
            size.as_vec2(),
            color,
            radius as f32,
            border_color,
            border_width as f32,
        );
    }

    /// Draws a rounded rectangle outline with a transparent fill.
    pub fn rounded_rectangle_outline(
        &mut self,
        position: IVec2,
        size: UVec2,
        radius: u32,
        border_color: Color,
        border_width: u32,
    ) {
        self.bordered_rectangle(
            position,
            size,
            radius,
            Color::Transparent,
            border_color,
            border_width,
        );
    }

    fn shape(
        &mut self,
        position: IVec2,
        size: Vec2,
        color: Color,
        radius: f32,
        border_color: Color,
        border_width: f32,
    ) {
        self.add(DrawCmd::Shape(ShapeCmd {
            position,
            size,
            rotation: 0.0,
            color,
            radius,
            border_color,
            border_width,
        }));
    }

    /// Draws a one-pixel-wide line between two logical screen positions.
    pub fn line(&mut self, start: IVec2, end: IVec2, color: Color) {
        let delta = (end - start).as_vec2();
        let length = delta.length();
        self.add(DrawCmd::Shape(ShapeCmd {
            position: start,
            size: vec2(length.max(1.0), 1.0),
            rotation: if length == 0.0 {
                0.0
            } else {
                delta.y.atan2(delta.x)
            },
            color,
            radius: 0.0,
            border_color: Color::Transparent,
            border_width: 0.0,
        }));
    }

    pub fn sprite_rotate(
        &mut self,
        sprite: &Sprite,
        position: IVec2,
        rotation: f32,
        rotation_anchor: Anchor,
    ) {
        self.add(DrawCmd::Sprite(SpriteCmd {
            sprite: sprite.id,
            position,
            rotation: rotation.to_radians(),
            rotation_anchor,
            animation: None,
            animation_tick: 0,
            animation_divisor: 1,
            flip: Flip::None,
            flip_diagonal: false,
        }));
    }

    pub fn sprite_flip(&mut self, sprite: &Sprite, position: IVec2, flip: Flip) {
        self.add(DrawCmd::Sprite(SpriteCmd {
            sprite: sprite.id,
            position,
            rotation: 0.0,
            rotation_anchor: Anchor::TopLeft,
            animation: None,
            animation_tick: 0,
            animation_divisor: 1,
            flip,
            flip_diagonal: false,
        }));
    }

    pub fn animated_sprite(&mut self, sprite: &AnimatedSprite, position: IVec2) {
        self.animated_sprite_flip(sprite, position, Flip::None);
    }

    pub fn animated_sprite_rotate(
        &mut self,
        sprite: &AnimatedSprite,
        position: IVec2,
        rotation: f32,
        rotation_anchor: Anchor,
    ) {
        self.add(DrawCmd::Sprite(SpriteCmd {
            sprite: sprite.id,
            position,
            rotation: rotation.to_radians(),
            rotation_anchor,
            animation: sprite.animation().map(str::to_owned),
            animation_tick: sprite.animation_tick(),
            animation_divisor: sprite.animation_divisor(),
            flip: Flip::None,
            flip_diagonal: false,
        }));
    }

    pub fn animated_sprite_flip(&mut self, sprite: &AnimatedSprite, position: IVec2, flip: Flip) {
        self.add(DrawCmd::Sprite(SpriteCmd {
            sprite: sprite.id,
            position,
            rotation: 0.0,
            rotation_anchor: Anchor::TopLeft,
            animation: sprite.animation().map(str::to_owned),
            animation_tick: sprite.animation_tick(),
            animation_divisor: sprite.animation_divisor(),
            flip,
            flip_diagonal: false,
        }));
    }

    pub fn text(&mut self, text: impl Into<String>, position: IVec2, anchor: Anchor) {
        self.add(DrawCmd::Text(TextCmd {
            font: None,
            text: text.into(),
            position,
            anchor,
            color: Color::White,
        }));
    }

    pub fn text_font(
        &mut self,
        font: Font,
        text: impl Into<String>,
        position: IVec2,
        anchor: Anchor,
    ) {
        self.add(DrawCmd::Text(TextCmd {
            font: Some(font),
            text: text.into(),
            position,
            anchor,
            color: Color::White,
        }))
    }

    pub fn text_color(
        &mut self,
        text: impl Into<String>,
        position: IVec2,
        anchor: Anchor,
        color: Color,
    ) {
        self.add(DrawCmd::Text(TextCmd {
            font: None,
            text: text.into(),
            position,
            anchor,
            color,
        }))
    }
}

#[derive(Clone, Copy)]
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
    pub(crate) fn offset(&self, size: Vec2) -> Vec2 {
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

pub enum Flip {
    None,
    Horizontal,
    Vertical,
    Both,
}

impl Flip {
    pub(crate) fn vector(&self) -> Vec2 {
        match self {
            Self::None => Vec2::NEG_ONE,
            Self::Horizontal => Vec2::X + Vec2::NEG_Y,
            Self::Vertical => Vec2::NEG_X + Vec2::Y,
            Self::Both => Vec2::ONE,
        }
    }
}
