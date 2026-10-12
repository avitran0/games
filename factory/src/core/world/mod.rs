use std::num::NonZeroU32;

mod chunk;
mod terrain;

pub struct World {}

#[derive(Clone, Copy, PartialEq, Hash)]
pub struct BlockId(NonZeroU32);

impl BlockId {
    pub fn new(id: NonZeroU32) -> Self {
        Self(id)
    }
}
