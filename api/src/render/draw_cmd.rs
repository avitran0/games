use glam::{IVec2, Vec2};

use crate::{
    Color, Font,
    handle::{SpriteId, TilemapId},
    render::frame::{Anchor, Flip},
};

pub(crate) enum DrawCmd {
    Sprite(SpriteCmd),
    AnimatedSprite(AnimatedSpriteCmd),
    Tilemap(TilemapCmd),
    Text(TextCmd),
    Shape(ShapeCmd),
}

pub(crate) struct SpriteCmd {
    pub(crate) sprite: SpriteId,
    pub(crate) position: IVec2,
    pub(crate) anchor: Anchor,
    pub(crate) rotation: f32,
    pub(crate) rotation_anchor: Anchor,
    pub(crate) flip: Flip,
    pub(crate) flip_diagonal: bool,
}

pub(crate) struct AnimatedSpriteCmd {
    pub(crate) sprite: SpriteId,
    pub(crate) position: IVec2,
    pub(crate) anchor: Anchor,
    pub(crate) rotation: f32,
    pub(crate) rotation_anchor: Anchor,
    pub(crate) animation: Option<String>,
    pub(crate) animation_tick: u16,
    pub(crate) animation_divisor: u16,
    pub(crate) flip: Flip,
    pub(crate) flip_diagonal: bool,
}

pub(crate) struct TilemapCmd {
    pub(crate) tilemap: TilemapId,
    pub(crate) position: IVec2,
}

pub(crate) struct TextCmd {
    pub(crate) font: Option<Font>,
    pub(crate) text: String,
    pub(crate) position: IVec2,
    pub(crate) anchor: Anchor,
    pub(crate) color: Color,
}

pub(crate) struct ShapeCmd {
    pub(crate) position: IVec2,
    pub(crate) size: Vec2,
    pub(crate) rotation: f32,
    pub(crate) color: Color,
    pub(crate) radius: f32,
    pub(crate) border_color: Color,
    pub(crate) border_width: f32,
}
