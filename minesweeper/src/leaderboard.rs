use std::cmp::Ordering;

use api::{
    Button,
    glam::UVec2,
    serde::{Deserialize, Serialize},
};

use crate::state::State;

pub const LEADERBOARD: &str = "leaderboard";

#[derive(Default, Serialize, Deserialize)]
#[serde(crate = "api::serde")]
pub struct Leaderboard {
    entries: Vec<LeaderboardEntry>,
}

impl Leaderboard {
    pub fn insert(&mut self, entry: LeaderboardEntry) {
        self.entries.push(entry);
        self.sort();

        if self.entries.len() > 100 {
            self.entries.remove(self.entries.len() - 1);
        }
    }

    fn sort(&mut self) {
        self.entries.sort_by(|a, b| {
            let a = a.ticks_per_tile();
            let b = b.ticks_per_tile();
            a.partial_cmp(&b).unwrap_or(Ordering::Equal)
        });
    }
}

#[derive(Default, Serialize, Deserialize)]
#[serde(crate = "api::serde")]
pub struct LeaderboardEntry {
    pub size: UVec2,
    pub time: usize,
}

impl LeaderboardEntry {
    fn ticks_per_tile(&self) -> f32 {
        self.time as f32 / (self.size.x * self.size.y) as f32
    }

    fn time_per_tile(&self) -> f32 {
        self.time as f32 / api::FPS as f32 / (self.size.x * self.size.y) as f32
    }
}

pub struct LeaderboardScreen {
    ui: ui::Ui,
    scroll: u32,
}

impl LeaderboardScreen {
    pub fn new() -> Self {
        let ui = ui::Ui::new();

        Self { ui, scroll: 0 }
    }
}

impl api::Screen<State> for LeaderboardScreen {
    fn update(&mut self, ctx: &mut api::ScreenContext<'_, State>) -> api::ScreenAction<State> {
        if ctx.input.just_pressed(Button::B) {
            return api::ScreenAction::Pop;
        }

        self.ui.begin_frame(ctx);

        self.ui.label("Leaderboard");
        self.ui
            .scroll_area(self.ui.remaining(), &mut self.scroll, |ui| {
                for (index, entry) in ctx.state.leaderboard.entries.iter().enumerate() {
                    ui.button(format!(
                        "{}: {:.2}s - {}x{} ({:.2}s/tile)",
                        index + 1,
                        entry.time as f32 / api::FPS as f32,
                        entry.size.x,
                        entry.size.y,
                        entry.time_per_tile()
                    ));
                }
            });

        api::ScreenAction::None
    }

    fn draw(&mut self, _state: &State, frame: &mut api::Frame) {
        self.ui.paint(frame);
    }
}
