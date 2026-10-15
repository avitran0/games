use crate::{main_menu::MainMenu, state::State};

mod config;
mod end_screen;
mod foods;
mod game;
mod main_menu;
mod pause;
mod snakes;
mod state;

fn main() {
    api::run("Snake", State::default(), |ctx| {
        Ok(Box::new(MainMenu::new(ctx)?))
    });
}
