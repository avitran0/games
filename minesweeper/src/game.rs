use api::{
    Button, SpriteOptions,
    glam::{UVec2, Vec2, ivec2},
};

use crate::{
    board::{Board, UncoverResult},
    camera::CameraRig,
    cursor::Cursor,
    difficulty::Difficulty,
    leaderboard::{LEADERBOARD, Leaderboard, LeaderboardEntry},
    pause::PauseScreen,
    review::{Outcome, ReviewScreen},
    sprites::Sprites,
    state::State,
};

pub(crate) const TILE_SIZE: u32 = 16;

pub struct GameScreen {
    sprites: Sprites,
    cursor: Cursor,
    board: Board,
    elapsed_ticks: usize,
}

impl GameScreen {
    pub fn new(
        ctx: &mut api::ScreenContext<State>,
        size: UVec2,
        difficulty: Difficulty,
    ) -> Result<Self, api::ScreenAction<State>> {
        let board = Board::new(size, difficulty);
        let cursor = Cursor::new(board.size());
        let camera_position = Self::cell_center(cursor.pos());
        ctx.state.camera = CameraRig::new(camera_position);
        let sprites = Sprites::load(ctx)?;

        Ok(Self {
            sprites,
            cursor,
            board,
            elapsed_ticks: 0,
        })
    }

    fn cell_center(cell: UVec2) -> Vec2 {
        cell.as_vec2() * TILE_SIZE as f32 + Vec2::splat(TILE_SIZE as f32 * 0.5)
    }
}

impl api::Screen<State> for GameScreen {
    fn update(&mut self, ctx: &mut api::ScreenContext<'_, State>) -> api::ScreenAction<State> {
        if ctx.input.just_pressed(Button::Start) {
            return api::ScreenAction::Push(Box::new(PauseScreen::new()));
        }

        ctx.state.camera.set_resolution(ctx.resolution);
        self.tick_cursor_and_camera(ctx);
        if let Some(outcome) = self.update_gameplay(ctx) {
            if let Err(action) = self.finish(outcome, ctx.state, ctx.resolution) {
                return action;
            }

            return api::ScreenAction::Push(Box::new(ReviewScreen::new(
                self.board.clone(),
                self.cursor.clone(),
                outcome,
                self.elapsed_ticks,
                self.sprites,
            )));
        }

        api::ScreenAction::None
    }

    fn draw(&mut self, state: &State, frame: &mut api::Frame) {
        let camera = state.camera.camera();
        let mut world = frame.world(camera);
        self.board.draw(camera, &mut world, &self.sprites);
        self.draw_cursor(frame);
    }
}

impl GameScreen {
    fn draw_cursor(&self, frame: &mut api::Frame) {
        let cursor_screen_pos = ivec2(
            (frame.width() as i32 - TILE_SIZE as i32) / 2,
            (frame.height() as i32 - TILE_SIZE as i32) / 2,
        );
        let opacity = (frame.tick() as f32 * 0.02).sin().abs() * 0.5 + 0.5;
        frame.sprite(
            &self.sprites.cursor,
            cursor_screen_pos,
            SpriteOptions::default().opacity(opacity),
        );
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

    fn update_gameplay(&mut self, ctx: &mut api::ScreenContext<'_, State>) -> Option<Outcome> {
        let outcome = if ctx.input.just_pressed(Button::A) {
            match self.board.uncover(self.cursor.pos()) {
                UncoverResult::HitBomb => Some(Outcome::GameOver),
                UncoverResult::Safe if self.board.has_won() => Some(Outcome::Win),
                UncoverResult::Safe => None,
            }
        } else {
            None
        };

        if ctx.input.just_pressed(Button::B) {
            self.board.flag(self.cursor.pos());
        }
        if self.board.is_generated() {
            self.elapsed_ticks += 1;
        }

        outcome
    }

    fn finish(
        &mut self,
        outcome: Outcome,
        state: &mut State,
        resolution: api::Resolution,
    ) -> Result<(), api::ScreenAction<State>> {
        match outcome {
            Outcome::Win => self.board.flag_bombs(),
            Outcome::GameOver => self.board.reveal_bombs(),
        }

        if outcome == Outcome::Win {
            let board_pixels = self.board.size().as_vec2() * TILE_SIZE as f32;
            state.camera.frame_board(resolution, board_pixels);

            state.leaderboard.insert(
                self.board.difficulty(),
                LeaderboardEntry {
                    size: self.board.size(),
                    time: self.elapsed_ticks,
                },
            );

            api::save::<State, Leaderboard>(LEADERBOARD, &state.leaderboard)?;
        }
        Ok(())
    }
}
