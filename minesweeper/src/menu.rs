use crate::{leaderboard::LeaderboardScreen, setup::SetupScreen, state::State};

pub struct MainMenu {
    ui: ui::Ui,
}

impl MainMenu {
    pub fn new() -> Self {
        let mut ui = ui::Ui::new();
        ui.set_alignment(ui::Alignment::Center);
        ui.set_bounds(ui::Rect::new(0, 48, api::WIDTH, api::HEIGHT - 48));

        Self { ui }
    }
}

impl api::Screen<State> for MainMenu {
    fn update(&mut self, ctx: &mut api::ScreenContext<'_, State>) -> api::ScreenAction<State> {
        self.ui.begin_frame(ctx);

        if self.ui.button("Play").clicked {
            return api::ScreenAction::Push(Box::new(SetupScreen::new()));
        }

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
