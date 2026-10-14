use api::glam::UVec2;
use ui::{Alignment, Rect, Style, Ui};

use crate::{game::GameScreen, menu::MainMenu, state::State};

#[derive(Clone, Copy)]
pub enum Outcome {
    Win,
    GameOver,
}

pub struct EndScreen {
    ui: Ui,
    outcome: Outcome,
    elapsed_ticks: usize,
    size: UVec2,
}

impl EndScreen {
    pub fn new(outcome: Outcome, elapsed_ticks: usize, size: UVec2) -> Self {
        let mut ui = Ui::new();
        ui.style = Style::light();
        ui.set_alignment(Alignment::Center);
        ui.set_bounds(Rect::new(0, 50, api::WIDTH, api::HEIGHT - 50));

        Self {
            ui,
            outcome,
            elapsed_ticks,
            size,
        }
    }

    fn formatted_time(&self) -> String {
        let total_seconds = self.elapsed_ticks / api::FPS as usize;
        format!("{:02}:{:02}", total_seconds / 60, total_seconds % 60)
    }
}

impl api::Screen<State> for EndScreen {
    fn update(&mut self, ctx: &mut api::ScreenContext<'_, State>) -> api::ScreenAction<State> {
        ctx.state.camera.tick();

        self.ui.begin_frame(ctx);
        self.ui.label(match self.outcome {
            Outcome::Win => "You win!",
            Outcome::GameOver => "Game over",
        });
        self.ui.label(format!("Time: {}", self.formatted_time()));

        if self.ui.button("Play again").clicked {
            return match GameScreen::new(ctx, self.size) {
                Ok(game) => api::ScreenAction::ClearAndPush(Box::new(game)),
                Err(action) => action,
            };
        }

        if self.ui.button("Main menu").clicked {
            return api::ScreenAction::ClearAndPush(Box::new(MainMenu::new()));
        }

        api::ScreenAction::None
    }

    fn draw(&mut self, _state: &State, frame: &mut api::Frame) {
        self.ui.paint(frame);
    }

    fn is_modal(&self) -> bool {
        true
    }
}
