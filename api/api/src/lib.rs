mod assets;
mod camera;
mod context;
pub mod formats;
mod glyphs;
mod handle;
mod input;
mod platform;
mod random;
mod render;
mod save;
mod screen;
mod tween;

use std::time::{Duration, Instant};

pub use assets::{Assets, MAX_TILEMAP_SIDE, TilemapCreateError, TilemapSetError};
pub use camera::Camera2D;
use context::Context;
pub use formats::Tileset;
pub use formats::color::Color;
pub use glam;
pub use glyphs::*;
pub use handle::{AnimatedSprite, Font, Sprite, SpriteSheet, Tilemap};
pub use input::{Button, Input};
pub use random::Rng;
pub use render::{
    TextMetrics,
    frame::{
        Anchor, Flip, Frame, ShapeOptions, SpriteOptions, TextOptions, TilemapOptions, WorldFrame,
    },
};
pub use save::{SaveName, load, save, save_name};
pub use screen::{Screen, ScreenAction, ScreenContext, error::ErrorScreen};
pub use tween::{Interpolation, Tween};

pub use serde;
pub use strum;
pub use utils::{debug, error, info, warn};

const TICK_DURATION: Duration = Duration::from_nanos(1_000_000_000 / 120);

pub fn run<State: AppState + 'static, Setup>(title: &str, state: State, setup: Setup)
where
    Setup: FnOnce(
        &mut ScreenContext<'_, State>,
    ) -> Result<Box<dyn Screen<State>>, ScreenAction<State>>,
{
    init_game_logger(State::app_id());

    let mut ctx = match Context::load(title, state) {
        Ok(ctx) => ctx,
        Err(err) => {
            error!("failed to initialize runtime: {err}");
            std::process::exit(1);
        }
    };
    ctx.setup(setup);
    info!("initialized runtime");

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

fn init_game_logger(app_id: &str) {
    let config_dir = save::game_data_dir(app_id);
    std::fs::create_dir_all(&config_dir).unwrap_or_else(|error| {
        panic!(
            "Cannot create game config directory {}: {error}",
            config_dir.display()
        )
    });

    let options = utils::log::LoggerOptions::default()
        .file(config_dir.join("game.log"))
        .truncate(true)
        .stdout(true);
    utils::log::init_ext(options).unwrap_or_else(|error| {
        panic!(
            "Cannot initialize game log in {}: {error}",
            config_dir.display()
        )
    });
}

pub trait AppState {
    fn app_id() -> &'static str;
}

#[macro_export]
macro_rules! impl_app_state {
    ($state:ty) => {
        impl $crate::AppState for $state {
            fn app_id() -> &'static str {
                env!("CARGO_CRATE_NAME")
            }
        }
    };
}

pub const FPS: u32 = 120;
pub const WIDTH: u32 = 240;
pub const HEIGHT: u32 = 160;
