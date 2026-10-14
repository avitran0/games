use crate::{menu::MainMenu, state::State};

mod cursor;
mod difficulty;
mod game;
mod menu;
mod pause;
mod sprites;
mod state;
mod tile;

fn main() {
    api::run("Minesweeper", State::default(), |_ctx| {
        Ok(Box::new(MainMenu::new()))
    });
}
