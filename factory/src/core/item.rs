#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Item {
    IronOre,
    Coal,
    TitaniumOre,
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

    pub fn try_add(&mut self, item: Item, capacity: u8) -> bool {
        if self.item != item || self.count >= capacity {
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
