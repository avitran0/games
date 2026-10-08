use crate::{game::GameScreen, state::State};

mod game;
mod state;

fn main() {
    api::run("Pong", State::default(), |ctx| {
        Ok(Box::new(GameScreen::new(ctx)))
    });
}
