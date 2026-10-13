use api::glam::IVec2;

use crate::core::{
    block_kind::BlockKind,
    item::{Item, ItemStack},
    world::terrain::OreKind,
};

use super::{
    behavior::BlockBehavior, conveyor::Conveyor, drill::Drill, smelter::Smelter, storage::Storage,
};

pub(super) enum BlockState {
    Conveyor(Conveyor),
    Drill(Drill),
    Storage(Storage),
    Smelter(Smelter),
    SolarPanel,
    Core,
}

impl BlockState {
    pub(super) fn new(kind: BlockKind) -> Self {
        match kind {
            BlockKind::Conveyor => Self::Conveyor(Conveyor::new()),
            BlockKind::Drill => Self::Drill(Drill::new()),
            BlockKind::Storage => Self::Storage(Storage::new()),
            BlockKind::Smelter => Self::Smelter(Smelter::new()),
            BlockKind::SolarPanel => Self::SolarPanel,
            BlockKind::Core => Self::Core,
        }
    }
}

impl BlockBehavior for BlockState {
    fn tick(&mut self, ore: Option<OreKind>, tick: usize, powered: bool) -> bool {
        match self {
            Self::Conveyor(block) => block.tick(ore, tick, powered),
            Self::Drill(block) => block.tick(ore, tick, powered),
            Self::Storage(block) => block.tick(ore, tick, powered),
            Self::Smelter(block) => block.tick(ore, tick, powered),
            Self::SolarPanel | Self::Core => false,
        }
    }

    fn output_item(&self, tick: usize) -> Option<Item> {
        match self {
            Self::Conveyor(block) => block.output_item(tick),
            Self::Drill(block) => block.output_item(tick),
            Self::Storage(block) => block.output_item(tick),
            Self::Smelter(block) => block.output_item(tick),
            Self::SolarPanel | Self::Core => None,
        }
    }

    fn item_progress(&self, tick: usize) -> Option<f32> {
        match self {
            Self::Conveyor(block) => block.item_progress(tick),
            Self::Drill(block) => block.item_progress(tick),
            Self::Storage(block) => block.item_progress(tick),
            Self::Smelter(block) => block.item_progress(tick),
            Self::SolarPanel | Self::Core => None,
        }
    }

    fn item_input_direction(&self) -> Option<IVec2> {
        match self {
            Self::Conveyor(block) => block.item_input_direction(),
            Self::Drill(block) => block.item_input_direction(),
            Self::Storage(block) => block.item_input_direction(),
            Self::Smelter(block) => block.item_input_direction(),
            Self::SolarPanel | Self::Core => None,
        }
    }

    fn items(&self) -> Option<ItemStack> {
        match self {
            Self::Conveyor(block) => block.items(),
            Self::Drill(block) => block.items(),
            Self::Storage(block) => block.items(),
            Self::Smelter(block) => block.items(),
            Self::SolarPanel | Self::Core => None,
        }
    }

    fn can_accept(&self, item: Item) -> bool {
        match self {
            Self::Conveyor(block) => block.can_accept(item),
            Self::Drill(block) => block.can_accept(item),
            Self::Storage(block) => block.can_accept(item),
            Self::Smelter(block) => block.can_accept(item),
            Self::SolarPanel => false,
            Self::Core => true,
        }
    }

    fn accept(&mut self, item: Item, tick: usize, input_direction: IVec2) -> bool {
        match self {
            Self::Conveyor(block) => block.accept(item, tick, input_direction),
            Self::Drill(block) => block.accept(item, tick, input_direction),
            Self::Storage(block) => block.accept(item, tick, input_direction),
            Self::Smelter(block) => block.accept(item, tick, input_direction),
            Self::SolarPanel => false,
            Self::Core => true,
        }
    }

    fn take(&mut self) -> Option<Item> {
        match self {
            Self::Conveyor(block) => block.take(),
            Self::Drill(block) => block.take(),
            Self::Storage(block) => block.take(),
            Self::Smelter(block) => block.take(),
            Self::SolarPanel | Self::Core => None,
        }
    }
}
