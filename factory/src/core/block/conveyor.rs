use crate::core::{
    block::behavior::BlockBehavior,
    item::{Item, ItemStack},
    world::terrain::OreKind,
};

pub(super) struct Conveyor {
    items: Option<ItemStack>,
}

impl Conveyor {
    const CAPACITY: u8 = 1;

    pub(super) fn new() -> Self {
        Self { items: None }
    }
}

impl BlockBehavior for Conveyor {
    fn tick(&mut self, _ore: Option<OreKind>) {}

    fn items(&self) -> Option<ItemStack> {
        self.items
    }

    fn can_accept(&self, item: Item) -> bool {
        self.items
            .is_none_or(|mut stack| stack.try_add(item, Self::CAPACITY))
    }

    fn accept(&mut self, item: Item) -> bool {
        match &mut self.items {
            Some(stack) => stack.try_add(item, Self::CAPACITY),
            None => {
                self.items = Some(ItemStack::single(item));
                true
            }
        }
    }

    fn take(&mut self) -> Option<Item> {
        let (remaining, item) = self.items.take()?.take_one();
        self.items = remaining;
        Some(item)
    }
}
