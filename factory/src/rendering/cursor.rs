use api::{Anchor, Assets, Camera2D, Frame, ScreenAction, Sprite, SpriteOptions, glam::UVec2};

use crate::core::{block::BlockSize, block::Rotation, block_kind::BlockKind};

pub struct CursorRenderer {
    single: Sprite,
    double: Sprite,
    triple: Sprite,
    quad: Sprite,
}

impl CursorRenderer {
    pub fn new<State>(assets: &mut Assets) -> Result<Self, ScreenAction<State>> {
        Ok(Self {
            single: assets.load_sprite(include_bytes!("../assets/cursor/single.pxs"))?,
            double: assets.load_sprite(include_bytes!("../assets/cursor/double.pxs"))?,
            triple: assets.load_sprite(include_bytes!("../assets/cursor/triple.pxs"))?,
            quad: assets.load_sprite(include_bytes!("../assets/cursor/quad.pxs"))?,
        })
    }

    pub fn draw(
        &self,
        camera: &Camera2D,
        frame: &mut Frame,
        tile_size: UVec2,
        position: UVec2,
        selected_block: BlockKind,
        rotation: Rotation,
    ) {
        let size = selected_block.size();
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
