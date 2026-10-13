use api::glam::IVec2;

use crate::simulation::item::{Item, ItemStack};

pub(super) struct TickContext {
    pub tick: usize,
}

pub(super) trait BlockBehavior {
    /// Runs one simulation step.
    fn tick(&mut self, _context: &TickContext) {}

    fn items(&self) -> Option<ItemStack> {
        None
    }

    fn can_accept(&self, _item: Item) -> bool {
        false
    }

    fn accept(&mut self, _item: Item, _tick: usize, _input_direction: IVec2) -> bool {
        false
    }

    fn take(&mut self, _tick: usize) -> Option<Item> {
        None
    }

    fn output_item(&self, _tick: usize) -> Option<Item> {
        self.items().map(ItemStack::item)
    }

    fn item_progress(&self, _tick: usize) -> Option<f32> {
        None
    }

    fn item_input_direction(&self) -> Option<IVec2> {
        None
    }
}
