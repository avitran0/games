use crate::core::{item::Item, world::BlockId};

pub enum WorldCommand {
    TransferItem {from: BlockId, to: BlockId, item: Item}
}
