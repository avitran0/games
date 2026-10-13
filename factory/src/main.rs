use crate::{screen::main::MainMenu, state::State};

mod core;
mod rendering;
mod screen;
mod sprite_store;
mod state;

fn main() {
    api::run("Factory", State::default(), |ctx| {
        ctx.state.sprites = Some(sprite_store::SpriteStore::load(ctx.assets)?);
        Ok(Box::new(MainMenu::new()))
    });
}
