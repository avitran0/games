use api::glam::uvec2;

use crate::{game::GameScreen, state::State};

pub struct MainMenu {
    ui: ui::Ui,
    width: u32,
    height: u32,
}

impl MainMenu {
    pub fn new() -> Self {
        let mut ui = ui::Ui::new();
        ui.set_alignment(ui::Alignment::Center);
        ui.set_bounds(ui::Rect::new(0, 64, api::WIDTH, api::HEIGHT - 64));

        Self {
            ui,
            width: 10,
            height: 8,
        }
    }
}

impl api::Screen<State> for MainMenu {
    fn update(&mut self, ctx: &mut api::ScreenContext<'_, State>) -> api::ScreenAction<State> {
        self.ui.begin_frame(ctx);

        if self.ui.button("Play").clicked {
            let game = match GameScreen::new(ctx, uvec2(self.width, self.height)) {
                Ok(game) => game,
                Err(action) => return action,
            };
            return api::ScreenAction::Replace(Box::new(game));
        }

        self.ui.slider(
            "Width",
            &mut self.width,
            GameScreen::MIN_BOARD_SIZE,
            GameScreen::MAX_BOARD_WIDTH,
        );
        self.ui.slider(
            "Height",
            &mut self.height,
            GameScreen::MIN_BOARD_SIZE,
            GameScreen::MAX_BOARD_HEIGHT,
        );

        if self.ui.button("Quit").clicked {
            return api::ScreenAction::Quit;
        }

        api::ScreenAction::None
    }

    fn draw(&mut self, _state: &State, frame: &mut api::Frame) {
        self.ui.paint(frame);
    }
}
