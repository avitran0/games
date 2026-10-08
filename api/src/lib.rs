mod assets;
mod context;
pub mod formats;
mod glyphs;
mod handle;
mod input;
mod platform;
mod random;
mod render;
mod screen;
mod tween;

use std::time::{Duration, Instant};

pub use assets::{Assets, TilemapSetError};
use context::Context;
pub use formats::Tileset;
pub use formats::color::Color;
pub use glam;
pub use glyphs::*;
pub use handle::{AnimatedSprite, Font, Sprite, Tilemap};
pub use input::{Button, Input};
pub use random::Rng;
pub use render::frame::{Anchor, Flip, Frame};
pub use screen::{Screen, ScreenAction, ScreenContext, error::ErrorScreen};
pub use tween::{Interpolation, Tween};

pub use utils::{debug, error, info, warn};

const TICK_DURATION: Duration = Duration::from_nanos(1_000_000_000 / 60);

pub fn run<State: 'static, Setup>(title: &str, state: State, setup: Setup)
where
    Setup: FnOnce(
        &mut ScreenContext<'_, State>,
    ) -> Result<Box<dyn Screen<State>>, ScreenAction<State>>,
{
    utils::log::init();

    let mut ctx = match Context::load(title, state) {
        Ok(ctx) => ctx,
        Err(err) => {
            error!("Cannot start '{title}': {err}");
            std::process::exit(1);
        }
    };
    ctx.setup(setup);
    info!("Runtime is ready.");

    let mut next_tick = Instant::now();
    while !ctx.should_quit() {
        ctx.begin_frame();
        ctx.update_screens();
        ctx.draw_screens();
        ctx.end_frame();

        next_tick += TICK_DURATION;
        let now = Instant::now();
        if next_tick > now {
            std::thread::sleep(next_tick - now);
        } else {
            next_tick = now;
        }
    }
}

pub const WIDTH: u32 = 240;
pub const HEIGHT: u32 = 160;
