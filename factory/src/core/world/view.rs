use crate::core::world::arena::BlockArena;

pub struct WorldView<'world> {
    blocks: &'world BlockArena,
}
