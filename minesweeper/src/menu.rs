use api::glam::{UVec2, uvec2};

use crate::{game::GameScreen, leaderboard::LeaderboardScreen, state::State};

pub struct MainMenu {
    ui: ui::Ui,
    size: UVec2,
}

impl MainMenu {
    pub fn new() -> Self {
        let mut ui = ui::Ui::new();
        ui.set_alignment(ui::Alignment::Center);
        ui.set_bounds(ui::Rect::new(0, 48, api::WIDTH, api::HEIGHT - 48));

        Self {
            ui,
            size: uvec2(30, 20),
        }
    }
}

impl api::Screen<State> for MainMenu {
    fn update(&mut self, ctx: &mut api::ScreenContext<'_, State>) -> api::ScreenAction<State> {
        self.ui.begin_frame(ctx);

        if self.ui.button("Play").clicked {
            let game = match GameScreen::new(ctx, self.size) {
                Ok(game) => game,
                Err(action) => return action,
            };
            return api::ScreenAction::Replace(Box::new(game));
        }

        self.ui
            .add(ui::Slider::new("Width", &mut self.size.x, 5, 200).with_step(5));
        self.ui
            .add(ui::Slider::new("Height", &mut self.size.y, 5, 200).with_step(5));

        if self.ui.button("Leaderboard").clicked {
            return api::ScreenAction::Push(Box::new(LeaderboardScreen::new()));
        }

        if self.ui.button("Quit").clicked {
            return api::ScreenAction::Quit;
        }

        api::ScreenAction::None
    }

    fn draw(&mut self, _state: &State, frame: &mut api::Frame) {
        self.ui.paint(frame);
    }
}
