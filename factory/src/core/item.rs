#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Item {
    IronOre,
    IronPlate,
    Coal,
    TitaniumOre,
}

impl Item {
    pub const COUNT: usize = 4;

    pub const fn index(self) -> usize {
        match self {
            Self::IronOre => 0,
            Self::IronPlate => 1,
            Self::Coal => 2,
            Self::TitaniumOre => 3,
        }
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
