use strum::{EnumCount, EnumIter, IntoEnumIterator};

#[repr(usize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, EnumIter, EnumCount)]
pub enum Item {
    IronOre,
    IronPlate,
    Coal,
    TitaniumOre,
}

impl Item {
    pub const COUNT: usize = <Self as EnumCount>::COUNT;

    pub const fn index(self) -> usize {
        self as usize
    }

    pub fn iter() -> impl Iterator<Item = Self> {
        <Self as IntoEnumIterator>::iter()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ItemStack {
    item: Item,
    count: u8,
}

impl ItemStack {
    pub const fn single(item: Item) -> Self {
        Self { item, count: 1 }
    }

    pub const fn item(self) -> Item {
        self.item
    }

    pub const fn count(self) -> u8 {
        self.count
    }

    pub fn can_add(self, item: Item, capacity: u8) -> bool {
        self.item == item && self.count < capacity
    }

    pub fn try_add(&mut self, item: Item, capacity: u8) -> bool {
        if !self.can_add(item, capacity) {
            return false;
        }
        self.count += 1;
        true
    }

    pub fn take_one(self) -> (Option<Self>, Item) {
        let remaining = (self.count > 1).then_some(Self {
            count: self.count - 1,
            ..self
        });
        (remaining, self.item)
    }
}
