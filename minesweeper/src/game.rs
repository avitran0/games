use api::{
    Anchor, Button, Color, SpriteOptions,
    glam::{UVec2, ivec2, uvec2},
};

use crate::{
    board::{Board, UncoverResult},
    camera::CameraRig,
    cursor::Cursor,
    end_screen::{EndScreen, Outcome},
    pause::PauseScreen,
    sprites::Sprites,
    state::State,
    tile::TileKind,
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
    ) -> Result<Self, api::ScreenAction<State>> {
        let board = Board::new(size);
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

    fn cell_center(cell: UVec2) -> api::glam::Vec2 {
        cell.as_vec2() * TILE_SIZE as f32 + api::glam::Vec2::splat(TILE_SIZE as f32 * 0.5)
    }
}

impl api::Screen<State> for GameScreen {
    fn update(&mut self, ctx: &mut api::ScreenContext<'_, State>) -> api::ScreenAction<State> {
        if ctx.input.just_pressed(Button::Start) {
            return api::ScreenAction::Push(Box::new(PauseScreen::new()));
        }

        self.tick_cursor_and_camera(ctx);
        if let Some(outcome) = self.update_gameplay(ctx) {
            self.finish(outcome, ctx.state);
            return api::ScreenAction::Push(Box::new(EndScreen::new(
                outcome,
                self.elapsed_ticks,
                self.board.size(),
            )));
        }

        api::ScreenAction::None
    }

    fn draw(&mut self, state: &State, frame: &mut api::Frame) {
        let camera = state.camera.camera();
        self.draw_board(frame, camera);
        self.draw_cursor(frame);
    }
}

impl GameScreen {
    fn draw_board(&self, frame: &mut api::Frame, camera: &api::Camera2D) {
        let (x_range, y_range) = self.visible_tile_ranges(camera);
        let mut world = frame.world(camera);
        for y in y_range {
            for x in x_range.clone() {
                self.draw_tile(&mut world, uvec2(x, y));
            }
        }
    }

    fn visible_tile_ranges(
        &self,
        camera: &api::Camera2D,
    ) -> (std::ops::RangeInclusive<u32>, std::ops::RangeInclusive<u32>) {
        let top_left = camera.screen_to_world(ivec2(0, 0));
        let bottom_right = camera.screen_to_world(ivec2(api::WIDTH as i32, api::HEIGHT as i32));
        let size = self.board.size();
        let x_range = Self::visible_tile_range(top_left.x, bottom_right.x, size.x);
        let y_range = Self::visible_tile_range(top_left.y, bottom_right.y, size.y);
        (x_range, y_range)
    }

    fn visible_tile_range(start: f32, end: f32, count: u32) -> std::ops::RangeInclusive<u32> {
        let max = count as i64 - 1;
        let first = ((start / TILE_SIZE as f32).floor() as i64 - 1).clamp(0, max) as u32;
        let last = ((end / TILE_SIZE as f32).floor() as i64 + 1).clamp(0, max) as u32;
        first..=last
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

    fn draw_tile(&self, world: &mut api::WorldFrame<'_, '_>, pos: UVec2) {
        let tile = self.board.tile(pos);
        let draw_pos = (pos * TILE_SIZE).as_ivec2();

        if !tile.uncovered {
            world.sprite(&self.sprites.covered, draw_pos, SpriteOptions::default());
            if tile.flagged {
                world.sprite(&self.sprites.flag, draw_pos, SpriteOptions::default());
            }
            return;
        }

        world.sprite(&self.sprites.uncovered, draw_pos, SpriteOptions::default());
        match tile.kind {
            TileKind::Clear => {
                let neighbors = self.board.neighbors(pos);
                if neighbors != 0 {
                    let text_pos = draw_pos + TILE_SIZE as i32 / 2;
                    let color = match neighbors {
                        1 => Color::Indigo,
                        2 => Color::ForestGreen,
                        3 => Color::TallPoppy,
                        4 => Color::PurpleHeart,
                        _ => Color::Bunker,
                    };
                    world.text_color(neighbors.to_string(), text_pos, Anchor::Center, color);
                }
            }
            TileKind::Bomb => {
                world.sprite(&self.sprites.bomb, draw_pos, SpriteOptions::default());
            }
        }
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

    fn finish(&mut self, outcome: Outcome, state: &mut State) {
        match outcome {
            Outcome::Win => self.board.flag_bombs(),
            Outcome::GameOver => self.board.reveal_bombs(),
        }
        let board_pixels = self.board.size().as_vec2() * TILE_SIZE as f32;
        state.camera.frame_board(board_pixels);
    }
}
