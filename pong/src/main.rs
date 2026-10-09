use crate::{main_menu::MainMenu, state::State};

mod game;
mod main_menu;
mod state;

fn main() {
    api::run("Pong", State::default(), |_ctx| Ok(Box::new(MainMenu)));
}
