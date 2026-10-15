use std::cmp::Ordering;

use api::{
    Button,
    glam::UVec2,
    serde::{Deserialize, Serialize},
};

use crate::{difficulty::Difficulty, state::State};

pub const LEADERBOARD: api::SaveName = api::save_name("leaderboard");

#[derive(Default, Serialize, Deserialize)]
#[serde(crate = "api::serde")]
pub struct Leaderboard {
    easy: Vec<LeaderboardEntry>,
    normal: Vec<LeaderboardEntry>,
    hard: Vec<LeaderboardEntry>,
}

impl Leaderboard {
    pub fn insert(&mut self, difficulty: Difficulty, entry: LeaderboardEntry) {
        self.entries_mut(difficulty).push(entry);
        self.sort();
    }

    pub fn sort(&mut self) {
        Self::sort_entries(&mut self.easy);
        Self::sort_entries(&mut self.normal);
        Self::sort_entries(&mut self.hard);
    }

    pub fn entries(&self, difficulty: Difficulty) -> &[LeaderboardEntry] {
        match difficulty {
            Difficulty::Easy => &self.easy,
            Difficulty::Normal => &self.normal,
            Difficulty::Hard => &self.hard,
        }
    }

    fn entries_mut(&mut self, difficulty: Difficulty) -> &mut Vec<LeaderboardEntry> {
        match difficulty {
            Difficulty::Easy => &mut self.easy,
            Difficulty::Normal => &mut self.normal,
            Difficulty::Hard => &mut self.hard,
        }
    }

    fn sort_entries(entries: &mut Vec<LeaderboardEntry>) {
        entries.sort_by(|a, b| {
            let a = a.ticks_per_tile();
            let b = b.ticks_per_tile();
            a.partial_cmp(&b).unwrap_or(Ordering::Equal)
        });
        entries.truncate(100);
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
    difficulty: Difficulty,
}

impl LeaderboardScreen {
    pub fn new() -> Self {
        let ui = ui::Ui::new();

        Self {
            ui,
            scroll: 0,
            difficulty: Difficulty::Normal,
        }
    }
}

impl api::Screen<State> for LeaderboardScreen {
    fn update(&mut self, ctx: &mut api::ScreenContext<'_, State>) -> api::ScreenAction<State> {
        if ctx.input.just_pressed(Button::B) {
            return api::ScreenAction::Pop;
        }

        self.ui.begin_frame(ctx);

        self.ui.label("Leaderboard");
        let previous_difficulty = self.difficulty;
        let leaderboard = &ctx.state.leaderboard;
        let scroll = &mut self.scroll;

        self.ui.tab_bar(&mut self.difficulty, |ui, difficulty| {
            if difficulty != previous_difficulty {
                *scroll = 0;
            }

            let entries = leaderboard.entries(difficulty);
            if entries.is_empty() {
                ui.label("No records for this difficulty");
                return;
            }

            ui.scroll_area(ui.remaining(), scroll, |ui| {
                for (index, entry) in entries.iter().enumerate() {
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
        });

        api::ScreenAction::None
    }

    fn draw(&mut self, _state: &State, frame: &mut api::Frame) {
        self.ui.paint(frame);
    }
}
