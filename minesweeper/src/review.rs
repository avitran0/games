use api::{
    Button, Color, SpriteOptions,
    glam::{IVec2, UVec2, ivec2, uvec2},
};
use ui::{Alignment, Ui, VerticalAlignment};

use crate::{
    board::Board,
    cursor::Cursor,
    game::{GameScreen, TILE_SIZE},
    menu::MainMenu,
    sprites::Sprites,
    state::State,
};

#[derive(Clone, Copy, PartialEq)]
pub enum Outcome {
    Win,
    GameOver,
}

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Menu,
    Review,
}

impl Mode {
    fn toggle(self) -> Self {
        match self {
            Self::Menu => Self::Review,
            Self::Review => Self::Menu,
        }
    }
}

pub struct ReviewScreen {
    board: Board,
    cursor: Cursor,
    sprites: Sprites,
    mode: Mode,
    ui: Ui,
    outcome: Outcome,
    elapsed_ticks: usize,
}

impl ReviewScreen {
    pub fn new(
        board: Board,
        cursor: Cursor,
        outcome: Outcome,
        elapsed_ticks: usize,
        sprites: Sprites,
    ) -> Self {
        let mut ui = Ui::new();
        ui.set_alignment(Alignment::Center);
        ui.set_vertical_alignment(VerticalAlignment::Center);

        Self {
            board,
            cursor,
            sprites,
            mode: Mode::Menu,
            ui,
            outcome,
            elapsed_ticks,
        }
    }

    fn formatted_time(&self) -> String {
        let total_seconds = self.elapsed_ticks / api::FPS as usize;
        format!("{:02}:{:02}", total_seconds / 60, total_seconds % 60)
    }

    fn cell_center(cell: UVec2) -> api::glam::Vec2 {
        cell.as_vec2() * TILE_SIZE as f32 + api::glam::Vec2::splat(TILE_SIZE as f32 * 0.5)
    }
}

impl api::Screen<State> for ReviewScreen {
    fn update(&mut self, ctx: &mut api::ScreenContext<'_, State>) -> api::ScreenAction<State> {
        if ctx.input.just_pressed(Button::Start) {
            self.mode = self.mode.toggle();
        }

        match self.mode {
            Mode::Menu => self.menu_update(ctx),
            Mode::Review => self.review_update(ctx),
        }
    }

    fn draw(&mut self, state: &State, frame: &mut api::Frame) {
        match self.mode {
            Mode::Menu => self.menu_draw(state, frame),
            Mode::Review => self.review_draw(state, frame),
        }
    }
}

impl ReviewScreen {
    fn review_update(
        &mut self,
        ctx: &mut api::ScreenContext<'_, State>,
    ) -> api::ScreenAction<State> {
        self.tick_cursor_and_camera(ctx);
        api::ScreenAction::None
    }

    fn review_draw(&mut self, state: &State, frame: &mut api::Frame) {
        self.draw_board(state, frame);
        self.draw_cursor(frame);
    }

    fn menu_update(&mut self, ctx: &mut api::ScreenContext<'_, State>) -> api::ScreenAction<State> {
        ctx.state.camera.tick();

        self.ui.begin_frame(ctx);
        self.ui.label(match self.outcome {
            Outcome::Win => "You win!",
            Outcome::GameOver => "Game over",
        });
        self.ui.label(format!("Time: {}", self.formatted_time()));

        if self.ui.button("Play again").clicked {
            return match GameScreen::new(ctx, self.board.size(), self.board.difficulty()) {
                Ok(game) => api::ScreenAction::ClearAndPush(Box::new(game)),
                Err(action) => action,
            };
        }

        if self.ui.button("Review").clicked {
            self.mode = self.mode.toggle();
        }

        if self.ui.button("Main menu").clicked {
            return api::ScreenAction::ClearAndPush(Box::new(MainMenu::new()));
        }

        api::ScreenAction::None
    }

    fn menu_draw(&mut self, state: &State, frame: &mut api::Frame) {
        self.draw_board(state, frame);
        frame.rectangle_with_opacity(
            IVec2::ZERO,
            uvec2(api::WIDTH, api::HEIGHT),
            Color::Black,
            0.1,
        );
        self.ui.paint(frame);
    }

    fn draw_board(&mut self, state: &State, frame: &mut api::Frame) {
        let camera = state.camera.camera();
        let mut world = frame.world(camera);
        self.board.draw(camera, &mut world, &self.sprites);
    }

    fn tick_cursor_and_camera(&mut self, ctx: &mut api::ScreenContext<'_, State>) {
        let previous_position = self.cursor.pos();
        self.cursor.tick(ctx);

        let camera = &mut ctx.state.camera;
        if self.cursor.pos() != previous_position {
            camera.move_to(Self::cell_center(self.cursor.pos()));
        }
        camera.tick();
    }

    fn draw_cursor(&self, frame: &mut api::Frame) {
        let cursor_screen_pos = ivec2(
            (api::WIDTH as i32 - TILE_SIZE as i32) / 2,
            (api::HEIGHT as i32 - TILE_SIZE as i32) / 2,
        );
        let opacity = (frame.tick() as f32 * 0.02).sin().abs() * 0.5 + 0.5;
        frame.sprite(
            &self.sprites.cursor,
            cursor_screen_pos,
            SpriteOptions::default().opacity(opacity),
        );
    }
}
