use std::num::NonZeroU32;

use crate::simulation::{block::Block, world::BlockId};

struct Slot {
    generation: NonZeroU32,
    block: Option<Block>,
}

#[derive(Default)]
pub struct BlockArena {
    slots: Vec<Slot>,
    free: Vec<u32>,
}

impl BlockArena {
    pub fn insert(&mut self, block: Block) -> BlockId {
        if let Some(index) = self.free.pop() {
            let slot = &mut self.slots[index as usize];
            slot.block = Some(block);
            return BlockId {
                index,
                generation: slot.generation,
            };
        }

        let index = self.slots.len() as u32;
        let generation = NonZeroU32::MIN;
        self.slots.push(Slot {
            generation,
            block: Some(block),
        });
        BlockId { index, generation }
    }

    pub fn get(&self, id: BlockId) -> Option<&Block> {
        let slot = self.slots.get(id.index as usize)?;
        if slot.generation != id.generation {
            return None;
        }
        slot.block.as_ref()
    }

    pub fn get_mut(&mut self, id: BlockId) -> Option<&mut Block> {
        let slot = self.slots.get_mut(id.index as usize)?;
        if slot.generation != id.generation {
            return None;
        }
        slot.block.as_mut()
    }

    pub fn remove(&mut self, id: BlockId) -> Option<Block> {
        let slot = self.slots.get_mut(id.index as usize)?;
        if slot.generation != id.generation {
            return None;
        }
        let block = slot.block.take()?;
        if let Some(next) = slot.generation.checked_add(1) {
            slot.generation = next;
            self.free.push(id.index);
        }

        Some(block)
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (BlockId, &mut Block)> {
        self.slots
            .iter_mut()
            .enumerate()
            .filter_map(|(index, slot)| {
                let block = slot.block.as_mut()?;
                Some((
                    BlockId {
                        index: index as u32,
                        generation: slot.generation,
                    },
                    block,
                ))
            })
    }

    pub fn iter(&self) -> impl Iterator<Item = (BlockId, &Block)> {
        self.slots.iter().enumerate().filter_map(|(index, slot)| {
            let block = slot.block.as_ref()?;
            Some((
                BlockId {
                    index: index as u32,
                    generation: slot.generation,
                },
                block,
            ))
        })
    }
}
