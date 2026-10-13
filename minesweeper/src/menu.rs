use crate::{difficulty::Difficulty, game::GameScreen, state::State};

#[derive(Default)]
pub struct MainMenu {
    ui: ui::Ui,
    difficulty: Difficulty,
}

impl MainMenu {
    pub fn new() -> Self {
        let mut ui = ui::Ui::new();
        ui.set_alignment(ui::Alignment::Center);
        ui.set_bounds(ui::Rect::new(0, 64, api::WIDTH, api::HEIGHT - 64));

        Self {
            ui,
            difficulty: Difficulty::Normal,
        }
    }
}

impl api::Screen<State> for MainMenu {
    fn update(&mut self, ctx: &mut api::ScreenContext<'_, State>) -> api::ScreenAction<State> {
        self.ui.begin_frame(ctx);

        if self.ui.button("Play").clicked {
            let game = match GameScreen::new(ctx, self.difficulty) {
                Ok(game) => game,
                Err(action) => return action,
            };
            return api::ScreenAction::Replace(Box::new(game));
        }

        self.ui.combo_box("Difficulty", &mut self.difficulty);

        if self.ui.button("Quit").clicked {
            return api::ScreenAction::Quit;
        }

        api::ScreenAction::None
    }

    fn draw(&mut self, state: &State, frame: &mut api::Frame) {
        self.ui.paint(frame);
    }
}
