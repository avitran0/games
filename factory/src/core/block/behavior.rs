use api::glam::IVec2;

use crate::core::{
    item::{Item, ItemStack},
    world::terrain::OreKind,
};

pub(super) trait BlockBehavior {
    /// Returns true when the block consumes one unit of stored power.
    fn tick(&mut self, ore: Option<OreKind>, tick: usize, powered: bool) -> bool;
    fn items(&self) -> Option<ItemStack>;
    fn can_accept(&self, item: Item) -> bool;
    fn accept(&mut self, item: Item, tick: usize, input_direction: IVec2) -> bool;
    fn take(&mut self) -> Option<Item>;

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
