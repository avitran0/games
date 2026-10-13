use api::{Anchor, Camera2D, Frame, SpriteOptions, glam::UVec2};

use crate::{
    simulation::{block::BlockSize, block_kind::BlockKind},
    sprite_store::SpriteStore,
};

use super::CursorView;

#[derive(Default)]
pub struct CursorRenderer;

impl CursorRenderer {
    pub fn new() -> Self {
        Self
    }

    pub fn draw(
        &self,
        camera: &Camera2D,
        frame: &mut Frame,
        tile_size: UVec2,
        cursor: CursorView,
        sprites: &SpriteStore,
    ) {
        let CursorView {
            position,
            selected_block,
            rotation,
        } = cursor;
        let size = selected_block.map_or(BlockSize::Single, BlockKind::size);
        let cursor_size = tile_size.as_ivec2() * size.cells() as i32;
        let center = position.as_ivec2() * tile_size.as_ivec2() + cursor_size / 2;
        let sprite = match size {
            BlockSize::Single => &sprites.cursor.single,
            BlockSize::Double => &sprites.cursor.double,
            BlockSize::Triple => &sprites.cursor.triple,
            BlockSize::Quad => &sprites.cursor.quad,
        };

        frame.world(camera).sprite(
            sprite,
            center,
            SpriteOptions::default()
                .anchor(Anchor::Center)
                .rotation(rotation.angle())
                .rotation_anchor(Anchor::Center)
                .opacity(0.5),
        );
    }
}
