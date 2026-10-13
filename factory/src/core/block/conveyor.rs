use crate::core::{item::Item, world::command::WorldCommand};

pub struct Conveyor {
    item: Option<Item>,
}

impl Conveyor {
    pub fn tick(&mut self) -> Option<WorldCommand> {
        None
    }
}
