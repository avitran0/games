use api::glam::IVec2;

use crate::simulation::{
    block_kind::BlockKind,
    item::{Item, ItemStack},
};

use super::{
    behavior::{BlockBehavior, TickContext},
    conveyor::Conveyor,
    drill::Drill,
    smelter::Smelter,
    storage::Storage,
};

pub(super) enum BlockState {
    Conveyor(Conveyor),
    Drill(Drill),
    Storage(Storage),
    Smelter(Smelter),
    Core(Core),
}

impl BlockState {
    pub(super) fn new(
        kind: BlockKind,
        ore: Option<crate::simulation::world::terrain::OreKind>,
    ) -> Self {
        match kind {
            BlockKind::Conveyor => Self::Conveyor(Conveyor::new()),
            BlockKind::Drill => Self::Drill(Drill::new(ore)),
            BlockKind::Storage => Self::Storage(Storage::new()),
            BlockKind::Smelter => Self::Smelter(Smelter::new()),
            BlockKind::Core => Self::Core(Core),
        }
    }

    fn behavior(&self) -> &dyn BlockBehavior {
        match self {
            Self::Conveyor(block) => block,
            Self::Drill(block) => block,
            Self::Storage(block) => block,
            Self::Smelter(block) => block,
            Self::Core(block) => block,
        }
    }

    fn behavior_mut(&mut self) -> &mut dyn BlockBehavior {
        match self {
            Self::Conveyor(block) => block,
            Self::Drill(block) => block,
            Self::Storage(block) => block,
            Self::Smelter(block) => block,
            Self::Core(block) => block,
        }
    }

    pub(super) fn tick(&mut self, context: &TickContext) {
        self.behavior_mut().tick(context);
    }

    pub(super) fn output_item(&self, tick: usize) -> Option<Item> {
        self.behavior().output_item(tick)
    }

    pub(super) fn item_progress(&self, tick: usize) -> Option<f32> {
        self.behavior().item_progress(tick)
    }

    pub(super) fn item_input_direction(&self) -> Option<IVec2> {
        self.behavior().item_input_direction()
    }

    pub(super) fn items(&self) -> Option<ItemStack> {
        self.behavior().items()
    }

    pub(super) fn can_accept(&self, item: Item) -> bool {
        self.behavior().can_accept(item)
    }

    pub(super) fn accept(&mut self, item: Item, tick: usize, input_direction: IVec2) -> bool {
        self.behavior_mut().accept(item, tick, input_direction)
    }

    pub(super) fn take(&mut self, tick: usize) -> Option<Item> {
        self.behavior_mut().take(tick)
    }
}

pub(super) struct Core;
impl BlockBehavior for Core {
    fn can_accept(&self, _item: Item) -> bool {
        true
    }

    fn accept(&mut self, _item: Item, _tick: usize, _input_direction: IVec2) -> bool {
        true
    }
}
