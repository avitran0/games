use crate::{menu::MainMenu, state::State};

mod difficulty;
mod game;
mod menu;
mod sprites;
mod state;

fn main() {
    api::run("Minesweeper", State::default(), |ctx| {
        Ok(Box::new(MainMenu::new()))
    });
}
