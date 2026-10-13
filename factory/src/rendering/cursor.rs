use api::{Anchor, Camera2D, Frame, Sprite, SpriteOptions, glam::UVec2};

use crate::{
    core::{
        block::{BlockSize, Rotation},
        block_kind::BlockKind,
    },
    sprite_store::SpriteStore,
};

pub struct CursorRenderer {
    single: Sprite,
    double: Sprite,
    triple: Sprite,
    quad: Sprite,
}

impl CursorRenderer {
    pub fn new(sprites: &SpriteStore) -> Self {
        Self {
            single: sprites.cursor.single,
            double: sprites.cursor.double,
            triple: sprites.cursor.triple,
            quad: sprites.cursor.quad,
        }
    }

    pub fn draw(
        &self,
        camera: &Camera2D,
        frame: &mut Frame,
        tile_size: UVec2,
        position: UVec2,
        selected_block: Option<BlockKind>,
        rotation: Rotation,
    ) {
        let size = selected_block.map_or(BlockSize::Single, BlockKind::size);
        let cursor_size = tile_size.as_ivec2() * size.cells() as i32;
        let center = position.as_ivec2() * tile_size.as_ivec2() + cursor_size / 2;
        let angle = match rotation {
            Rotation::Up => 0.0,
            Rotation::Right => 90.0,
            Rotation::Down => 180.0,
            Rotation::Left => 270.0,
        };
        let sprite = match size {
            BlockSize::Single => &self.single,
            BlockSize::Double => &self.double,
            BlockSize::Triple => &self.triple,
            BlockSize::Quad => &self.quad,
        };

        frame.world(camera).sprite(
            sprite,
            center,
            SpriteOptions::default()
                .anchor(Anchor::Center)
                .rotation(angle)
                .rotation_anchor(Anchor::Center)
                .opacity(0.5),
        );
    }
}
