use std::collections::HashMap;

use api::{AnimatedSprite, Assets, ScreenAction, Sprite};

use crate::{core::block_kind::BlockKind, state::State};

const BLOCK_ANIMATION_DIVISOR: u16 = 8;

pub struct SpriteStore {
    blocks: HashMap<BlockKind, AnimatedSprite>,
    pub cursor: CursorSprites,
}

pub struct CursorSprites {
    pub single: Sprite,
    pub double: Sprite,
    pub triple: Sprite,
    pub quad: Sprite,
}

impl SpriteStore {
    pub fn load(assets: &mut Assets) -> Result<Self, ScreenAction<State>> {
        let mut blocks = HashMap::new();
        for kind in BlockKind::iter() {
            blocks.insert(kind, assets.load_animated_sprite(block_asset(kind))?);
        }

        Ok(Self {
            blocks,
            cursor: CursorSprites {
                single: assets.load_sprite(include_bytes!("assets/cursor/single.pxs"))?,
                double: assets.load_sprite(include_bytes!("assets/cursor/double.pxs"))?,
                triple: assets.load_sprite(include_bytes!("assets/cursor/triple.pxs"))?,
                quad: assets.load_sprite(include_bytes!("assets/cursor/quad.pxs"))?,
            },
        })
    }

    pub fn block(&self, kind: BlockKind) -> AnimatedSprite {
        let mut sprite = self.blocks[&kind].clone();
        sprite.set_animation("default");
        sprite.set_divisor(BLOCK_ANIMATION_DIVISOR);
        sprite
    }
}

fn block_asset(kind: BlockKind) -> &'static [u8] {
    match kind {
        BlockKind::Conveyor => include_bytes!("assets/blocks/conveyor.pxa"),
        BlockKind::Drill => include_bytes!("assets/blocks/drill.pxa"),
    }
}
