use api::glam::IVec2;

use crate::core::{
    block::behavior::BlockBehavior,
    item::{Item, ItemStack},
    world::terrain::OreKind,
};

const STORAGE_CAPACITY: u8 = 20;
const OUTPUT_INTERVAL: usize = 30;

pub(super) struct Storage {
    items: Option<ItemStack>,
    next_output_tick: Option<usize>,
}

impl Storage {
    pub(super) fn new() -> Self {
        Self {
            items: None,
            next_output_tick: None,
        }
    }
}

impl BlockBehavior for Storage {
    fn tick(&mut self, _ore: Option<OreKind>, _tick: usize, _powered: bool) -> bool {
        false
    }

    fn items(&self) -> Option<ItemStack> {
        self.items
    }

    fn can_accept(&self, item: Item) -> bool {
        self.items
            .is_none_or(|mut stack| stack.try_add(item, STORAGE_CAPACITY))
    }

    fn accept(&mut self, item: Item, tick: usize, _input_direction: IVec2) -> bool {
        let accepted = match &mut self.items {
            Some(stack) => stack.try_add(item, STORAGE_CAPACITY),
            None => {
                self.items = Some(ItemStack::single(item));
                true
            }
        };
        if accepted && self.next_output_tick.is_none() {
            self.next_output_tick = Some(tick.saturating_add(OUTPUT_INTERVAL));
        }
        accepted
    }

    fn take(&mut self) -> Option<Item> {
        let (remaining, item) = self.items.take()?.take_one();
        self.items = remaining;
        if self.items.is_none() {
            self.next_output_tick = None;
        }
        Some(item)
    }

    fn output_item(&self, tick: usize) -> Option<Item> {
        (self.next_output_tick? <= tick)
            .then(|| self.items.map(ItemStack::item))
            .flatten()
    }
}
