use api::{
    Button,
    glam::{UVec2, uvec2},
};
use ui::{Alignment, Ui};

use crate::{difficulty::Difficulty, game::GameScreen, state::State};

pub struct SetupScreen {
    ui: Ui,
    size: UVec2,
    difficulty: Difficulty,
}

impl SetupScreen {
    pub fn new() -> Self {
        let mut ui = Ui::new();
        ui.set_alignment(Alignment::Center);

        Self {
            ui,
            size: uvec2(30, 20),
            difficulty: Difficulty::Normal,
        }
    }
}

impl api::Screen<State> for SetupScreen {
    fn update(&mut self, ctx: &mut api::ScreenContext<'_, State>) -> api::ScreenAction<State> {
        if ctx.input.just_pressed(Button::B) {
            return api::ScreenAction::Pop;
        }

        self.ui.begin_frame(ctx);

        if self.ui.button("Start").clicked {
            let game = match GameScreen::new(ctx, self.size, self.difficulty) {
                Ok(game) => game,
                Err(action) => return action,
            };
            return api::ScreenAction::ClearAndPush(Box::new(game));
        }

        self.ui
            .add(ui::Slider::new("Width", &mut self.size.x, 5, 200).with_step(5));
        self.ui
            .add(ui::Slider::new("Height", &mut self.size.y, 5, 200).with_step(5));
        self.ui.combo_box("Difficulty", &mut self.difficulty);

        api::ScreenAction::None
    }

    fn draw(&mut self, _state: &State, frame: &mut api::Frame) {
        self.ui.paint(frame);
    }
}
