use crate::core::{
    item::{Item, ItemStack},
    world::terrain::OreKind,
};

pub(super) trait BlockBehavior {
    fn tick(&mut self, ore: Option<OreKind>);
    fn items(&self) -> Option<ItemStack>;
    fn can_accept(&self, item: Item) -> bool;
    fn accept(&mut self, item: Item) -> bool;
    fn take(&mut self) -> Option<Item>;

    fn output_item(&self) -> Option<Item> {
        self.items().map(ItemStack::item)
    }
}
