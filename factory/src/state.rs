use crate::{core::block_kind::BlockKind, sprite_store::SpriteStore};

#[derive(Default)]
pub struct State {
    pub picked_block: Option<BlockKind>,
    pub sprites: Option<SpriteStore>,
}
