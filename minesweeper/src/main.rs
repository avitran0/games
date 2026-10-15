use crate::{leaderboard::LEADERBOARD, menu::MainMenu, state::State};

mod board;
mod camera;
mod cursor;
mod end_screen;
mod game;
mod leaderboard;
mod menu;
mod pause;
mod sprites;
mod state;
mod tile;

fn main() {
    api::run("Minesweeper", State::default(), |ctx| {
        ctx.state.leaderboard = api::load(LEADERBOARD)?;
        Ok(Box::new(MainMenu::new()))
    });
}
