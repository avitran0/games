use crate::simulation::block_kind::BlockKind;

#[derive(Default)]
pub struct State {
    pub picked_block: Option<BlockKind>,
}
