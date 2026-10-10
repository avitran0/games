use crate::{screen::main::MainMenu, state::State};

mod core;
mod screen;
mod state;

fn main() {
    api::run("Factory", State::default(), |ctx| {
        Ok(Box::new(MainMenu::new()))
    });
}
