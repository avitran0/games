use api::glam::IVec2;

use crate::core::{
    block_kind::BlockKind,
    item::{Item, ItemStack},
    world::terrain::OreKind,
};

use super::{behavior::BlockBehavior, conveyor::Conveyor, drill::Drill};

pub(super) enum BlockState {
    Conveyor(Conveyor),
    Drill(Drill),
}

impl BlockState {
    pub(super) fn new(kind: BlockKind) -> Self {
        match kind {
            BlockKind::Conveyor => Self::Conveyor(Conveyor::new()),
            BlockKind::Drill => Self::Drill(Drill::new()),
        }
    }
}

impl BlockBehavior for BlockState {
    fn tick(&mut self, ore: Option<OreKind>, tick: usize) {
        match self {
            Self::Conveyor(block) => block.tick(ore, tick),
            Self::Drill(block) => block.tick(ore, tick),
        }
    }

    fn output_item(&self, tick: usize) -> Option<Item> {
        match self {
            Self::Conveyor(block) => block.output_item(tick),
            Self::Drill(block) => block.output_item(tick),
        }
    }

    fn item_progress(&self, tick: usize) -> Option<f32> {
        match self {
            Self::Conveyor(block) => block.item_progress(tick),
            Self::Drill(block) => block.item_progress(tick),
        }
    }

    fn item_input_direction(&self) -> Option<IVec2> {
        match self {
            Self::Conveyor(block) => block.item_input_direction(),
            Self::Drill(block) => block.item_input_direction(),
        }
    }

    fn items(&self) -> Option<ItemStack> {
        match self {
            Self::Conveyor(block) => block.items(),
            Self::Drill(block) => block.items(),
        }
    }

    fn can_accept(&self, item: Item) -> bool {
        match self {
            Self::Conveyor(block) => block.can_accept(item),
            Self::Drill(block) => block.can_accept(item),
        }
    }

    fn accept(&mut self, item: Item, tick: usize, input_direction: IVec2) -> bool {
        match self {
            Self::Conveyor(block) => block.accept(item, tick, input_direction),
            Self::Drill(block) => block.accept(item, tick, input_direction),
        }
    }

    fn take(&mut self) -> Option<Item> {
        match self {
            Self::Conveyor(block) => block.take(),
            Self::Drill(block) => block.take(),
        }
    }
}
