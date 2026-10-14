use crate::{menu::MainMenu, state::State};

mod board;
mod camera;
mod cursor;
mod end_screen;
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
