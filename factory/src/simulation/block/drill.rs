use api::glam::IVec2;

use crate::simulation::{
    block::behavior::{BlockBehavior, TickContext},
    item::{Item, ItemStack},
    world::terrain::OreKind,
};

pub struct Drill {
    output: Option<ItemStack>,
    progress: u16,
    ore: Option<OreKind>,
}

impl Drill {
    pub const OUTPUT_CAPACITY: u8 = 10;
    pub const EXTRACTION_PERIOD_TICKS: u16 = 120;

    pub(super) fn new(ore: Option<OreKind>) -> Self {
        Self {
            output: None,
            progress: 0,
            ore,
        }
    }
}

impl BlockBehavior for Drill {
    fn tick(&mut self, _context: &TickContext) {
        if self
            .output
            .is_some_and(|stack| stack.count() >= Self::OUTPUT_CAPACITY)
        {
            return;
        }

        let Some(ore) = self.ore else {
            self.progress = 0;
            return;
        };
        let item = ore.item();
        if self.output.is_some_and(|stack| stack.item() != item) {
            return;
        }

        self.progress += 1;
        if self.progress >= Self::EXTRACTION_PERIOD_TICKS {
            match &mut self.output {
                Some(stack) => {
                    let added = stack.try_add(item, Self::OUTPUT_CAPACITY);
                    debug_assert!(added);
                }
                None => self.output = Some(ItemStack::single(item)),
            }
            self.progress = 0;
        }
    }

    fn items(&self) -> Option<ItemStack> {
        self.output
    }

    fn can_accept(&self, _item: Item) -> bool {
        false
    }

    fn accept(&mut self, _item: Item, _tick: usize, _input_direction: IVec2) -> bool {
        false
    }

    fn take(&mut self, _tick: usize) -> Option<Item> {
        let (remaining, item) = self.output.take()?.take_one();
        self.output = remaining;
        Some(item)
    }
}
