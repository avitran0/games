use api::glam::IVec2;

use crate::simulation::{
    block::behavior::{BlockBehavior, TickContext},
    item::{Item, ItemStack},
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
        Some((elapsed as f32 / CONVEYOR_TRAVEL_TICKS as f32).min(1.0))
    }
}

impl BlockBehavior for Conveyor {
    fn tick(&mut self, _context: &TickContext) {}

    fn items(&self) -> Option<ItemStack> {
        self.items
    }

    fn can_accept(&self, item: Item) -> bool {
        self.items
            .is_none_or(|stack| stack.can_add(item, Self::CAPACITY))
    }

    fn accept(&mut self, item: Item, tick: usize, input_direction: IVec2) -> bool {
        match &mut self.items {
            Some(stack) => stack.try_add(item, Self::CAPACITY),
            None => {
                self.items = Some(ItemStack::single(item));
                self.start_tick = Some(tick);
                self.input_direction = Some(input_direction);
                true
            }
        }
    }

    fn take(&mut self, _tick: usize) -> Option<Item> {
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

#[cfg(test)]
mod tests {
    use api::glam::IVec2;

    use crate::simulation::{block::behavior::BlockBehavior, item::Item};

    use super::{CONVEYOR_TRAVEL_TICKS, Conveyor};

    #[test]
    fn conveyor_travel_time_starts_at_acceptance_without_global_phase_rounding() {
        let mut conveyor = Conveyor::new();
        assert!(conveyor.accept(Item::IronOre, 145, IVec2::NEG_Y));
        assert_eq!(conveyor.output_item(145 + CONVEYOR_TRAVEL_TICKS - 1), None);
        assert_eq!(
            conveyor.output_item(145 + CONVEYOR_TRAVEL_TICKS),
            Some(Item::IronOre)
        );
        assert_eq!(conveyor.item_progress(157), Some(0.5));
    }
}
