use api::glam::IVec2;

use crate::core::{
    block::behavior::BlockBehavior,
    item::{Item, ItemStack},
    world::terrain::OreKind,
};

const SMELT_TICKS: usize = 90;
const OUTPUT_CAPACITY: u8 = 10;

pub(super) struct Smelter {
    input: Option<(Item, usize)>,
    output: Option<ItemStack>,
}

impl Smelter {
    pub(super) fn new() -> Self {
        Self {
            input: None,
            output: None,
        }
    }
}

impl BlockBehavior for Smelter {
    fn tick(&mut self, _ore: Option<OreKind>, tick: usize, powered: bool) -> bool {
        let Some((Item::IronOre, start_tick)) = self.input else {
            return false;
        };
        if tick.saturating_sub(start_tick) < SMELT_TICKS
            || self
                .output
                .is_some_and(|stack| stack.count() >= OUTPUT_CAPACITY)
            || !powered
        {
            return false;
        }
        match &mut self.output {
            Some(stack) => {
                let _ = stack.try_add(Item::IronPlate, OUTPUT_CAPACITY);
            }
            None => self.output = Some(ItemStack::single(Item::IronPlate)),
        }
        self.input = None;
        true
    }

    fn output_item(&self, _tick: usize) -> Option<Item> {
        self.output.map(ItemStack::item)
    }

    fn items(&self) -> Option<ItemStack> {
        self.output
            .or_else(|| self.input.map(|(item, _)| ItemStack::single(item)))
    }

    fn can_accept(&self, item: Item) -> bool {
        item == Item::IronOre
            && self.input.is_none()
            && self
                .output
                .is_none_or(|stack| stack.count() < OUTPUT_CAPACITY)
    }

    fn accept(&mut self, item: Item, tick: usize, _input_direction: IVec2) -> bool {
        if !self.can_accept(item) {
            return false;
        }
        self.input = Some((item, tick));
        true
    }

    fn take(&mut self) -> Option<Item> {
        let (remaining, item) = self.output.take()?.take_one();
        self.output = remaining;
        Some(item)
    }
}
