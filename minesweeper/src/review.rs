use api::{
    Button, Color, Interpolation, SpriteOptions, Tween,
    glam::{IVec2, UVec2, Vec2},
};
use ui::{Alignment, Style, Ui, VerticalAlignment};

use crate::{
    board::Board,
    cursor::Cursor,
    game::{GameScreen, TILE_SIZE},
    menu::MainMenu,
    sprites::Sprites,
    state::State,
};

const GAME_OVER_FLASH_TICKS: usize = 20;
const GAME_OVER_FLASH_OPACITY: f32 = 0.42;

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
    camera_mode: Option<Mode>,
    ui: Ui,
    outcome: Outcome,
    elapsed_ticks: usize,
    flash: Option<Tween<f32>>,
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
        // sets dark text color
        ui.style = Style::light();
        ui.set_alignment(Alignment::Center);
        ui.set_vertical_alignment(VerticalAlignment::Center);

        Self {
            board,
            cursor,
            sprites,
            mode: Mode::Menu,
            camera_mode: None,
            ui,
            outcome,
            elapsed_ticks,
            flash: (outcome == Outcome::GameOver).then(|| {
                Tween::new(
                    GAME_OVER_FLASH_OPACITY,
                    0.0,
                    GAME_OVER_FLASH_TICKS,
                    Interpolation::EaseOutCubic,
                )
            }),
        }
    }

    fn formatted_time(&self) -> String {
        let total_seconds = self.elapsed_ticks / api::FPS as usize;
        format!("{:02}:{:02}", total_seconds / 60, total_seconds % 60)
    }

    fn cell_center(cell: UVec2) -> Vec2 {
        cell.as_vec2() * TILE_SIZE as f32 + Vec2::splat(TILE_SIZE as f32 * 0.5)
    }
}

impl api::Screen<State> for ReviewScreen {
    fn update(&mut self, ctx: &mut api::ScreenContext<'_, State>) -> api::ScreenAction<State> {
        if ctx.input.just_pressed(Button::Start) {
            self.mode = self.mode.toggle();
        }

        self.board.tick();
        if let Some(flash) = &mut self.flash {
            flash.tick();
            if flash.is_finished() {
                self.flash = None;
            }
        }
        self.sync_camera_mode(&mut ctx.state.camera, ctx.resolution);

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
        self.draw_cursor(state.camera.camera(), frame);
        self.draw_flash(frame);
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
        frame.rectangle_with_opacity(IVec2::ZERO, frame.resolution().into(), Color::Black, 0.1);
        self.draw_flash(frame);
        self.ui.paint(frame);
    }

    fn draw_flash(&self, frame: &mut api::Frame) {
        if let Some(flash) = &self.flash {
            frame.rectangle_with_opacity(
                IVec2::ZERO,
                frame.resolution().into(),
                Color::TallPoppy,
                flash.current(),
            );
        }
    }

    fn sync_camera_mode(
        &mut self,
        camera: &mut crate::camera::CameraRig,
        resolution: api::Resolution,
    ) {
        let resolution_changed = camera.camera().resolution() != resolution;
        camera.set_resolution(resolution);
        if self.camera_mode == Some(self.mode) && !resolution_changed {
            return;
        }

        match self.mode {
            Mode::Menu => {
                let board_pixels = self.board.size().as_vec2() * TILE_SIZE as f32;
                camera.frame_board(resolution, board_pixels);
            }
            Mode::Review => camera.focus_cursor(Self::cell_center(self.cursor.pos())),
        }
        self.camera_mode = Some(self.mode);
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

    fn draw_cursor(&self, camera: &api::Camera2D, frame: &mut api::Frame) {
        let cursor_world_pos =
            Self::cell_center(self.cursor.pos()) - Vec2::splat(TILE_SIZE as f32 * 0.5);
        let cursor_screen_pos = camera.world_to_screen(cursor_world_pos);
        let opacity = (frame.tick() as f32 * 0.02).sin().abs() * 0.5 + 0.5;
        frame.sprite(
            &self.sprites.cursor,
            cursor_screen_pos,
            SpriteOptions::default()
                .uniform_scale(camera.zoom())
                .opacity(opacity),
        );
    }
}
