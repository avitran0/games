use api::glam::IVec2;

use crate::core::{
    block::behavior::BlockBehavior,
    item::{Item, ItemStack},
    world::terrain::OreKind,
};

use super::CONVEYOR_TRAVEL_TICKS;

pub(super) struct Conveyor {
    items: Option<ItemStack>,
    start_tick: Option<usize>,
    input_direction: Option<IVec2>,
}

impl Conveyor {
    const CAPACITY: u8 = 1;

    pub(super) fn new() -> Self {
        Self {
            items: None,
            start_tick: None,
            input_direction: None,
        }
    }

    fn progress(&self, tick: usize) -> Option<f32> {
        let start_tick = self.start_tick?;
        self.items?;
        let elapsed = tick.saturating_sub(start_tick);
        Some((elapsed as f32 / CONVEYOR_TRAVEL_TICKS as f32).min(0.95))
    }
}

impl BlockBehavior for Conveyor {
    fn tick(&mut self, _ore: Option<OreKind>, _tick: usize, _powered: bool) -> bool {
        false
    }

    fn items(&self) -> Option<ItemStack> {
        self.items
    }

    fn can_accept(&self, item: Item) -> bool {
        self.items
            .is_none_or(|mut stack| stack.try_add(item, Self::CAPACITY))
    }

    fn accept(&mut self, item: Item, tick: usize, input_direction: IVec2) -> bool {
        match &mut self.items {
            Some(stack) => stack.try_add(item, Self::CAPACITY),
            None => {
                self.items = Some(ItemStack::single(item));
                let remainder = tick % CONVEYOR_TRAVEL_TICKS;
                self.start_tick = Some(if remainder == 0 {
                    tick
                } else {
                    tick + CONVEYOR_TRAVEL_TICKS - remainder
                });
                self.input_direction = Some(input_direction);
                true
            }
        }
    }

    fn take(&mut self) -> Option<Item> {
        let (remaining, item) = self.items.take()?.take_one();
        self.items = remaining;
        if self.items.is_none() {
            self.start_tick = None;
            self.input_direction = None;
        }
        Some(item)
    }

    fn output_item(&self, tick: usize) -> Option<Item> {
        let start_tick = self.start_tick?;
        if tick.saturating_sub(start_tick) < CONVEYOR_TRAVEL_TICKS {
            return None;
        }
        self.items.map(ItemStack::item)
    }

    fn item_progress(&self, tick: usize) -> Option<f32> {
        self.progress(tick)
    }

    fn item_input_direction(&self) -> Option<IVec2> {
        self.input_direction
    }
}
