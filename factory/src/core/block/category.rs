use strum::EnumIter;

use crate::core::block_kind::BlockKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumIter)]

pub enum BlockCategory {
    Logistics,
    Extraction,
}

impl BlockCategory {
    pub fn blocks(self) -> impl Iterator<Item = BlockKind> {
        BlockKind::iter().filter(move |kind| kind.category() == self)
    }
}
