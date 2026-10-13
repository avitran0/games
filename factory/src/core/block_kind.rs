use crate::core::block::BlockSize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockKind {
    Conveyor,
    Drill,
}

impl BlockKind {
    pub fn size(self) -> BlockSize {
        match self {
            Self::Conveyor => BlockSize::Single,
            Self::Drill => BlockSize::Double,
        }
    }
}
