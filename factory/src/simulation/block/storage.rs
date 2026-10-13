use api::glam::IVec2;

use crate::simulation::{
    block::behavior::{BlockBehavior, TickContext},
    item::{Item, ItemStack},
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
    fn tick(&mut self, _context: &TickContext) {}

    fn items(&self) -> Option<ItemStack> {
        self.items
    }

    fn can_accept(&self, item: Item) -> bool {
        self.items
            .is_none_or(|stack| stack.can_add(item, STORAGE_CAPACITY))
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

    fn take(&mut self, tick: usize) -> Option<Item> {
        let (remaining, item) = self.items.take()?.take_one();
        self.items = remaining;
        self.next_output_tick = self.items.map(|_| tick.saturating_add(OUTPUT_INTERVAL));
        Some(item)
    }

    fn output_item(&self, tick: usize) -> Option<Item> {
        (self.next_output_tick? <= tick)
            .then(|| self.items.map(ItemStack::item))
            .flatten()
    }
}

#[cfg(test)]
mod tests {
    use api::glam::IVec2;

    use crate::simulation::{block::behavior::BlockBehavior, item::Item};

    use super::{OUTPUT_INTERVAL, Storage};

    #[test]
    fn storage_waits_the_output_interval_between_items() {
        let mut storage = Storage::new();
        assert!(storage.accept(Item::IronOre, 0, IVec2::ZERO));
        assert!(storage.accept(Item::IronOre, 0, IVec2::ZERO));
        assert_eq!(storage.output_item(OUTPUT_INTERVAL - 1), None);
        assert_eq!(storage.output_item(OUTPUT_INTERVAL), Some(Item::IronOre));
        assert_eq!(storage.take(OUTPUT_INTERVAL), Some(Item::IronOre));
        assert_eq!(storage.output_item(OUTPUT_INTERVAL * 2 - 1), None);
        assert_eq!(
            storage.output_item(OUTPUT_INTERVAL * 2),
            Some(Item::IronOre)
        );
        assert_eq!(storage.take(OUTPUT_INTERVAL * 2), Some(Item::IronOre));
        assert_eq!(storage.output_item(OUTPUT_INTERVAL * 3), None);
    }
}
