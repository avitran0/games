use crate::{screen::main::MainMenu, state::State};

mod rendering;
mod screen;
mod simulation;
mod sprite_store;
mod state;

fn main() {
    api::run("Factory", State::default(), |_ctx| {
        Ok(Box::new(MainMenu::new()))
    });
}
