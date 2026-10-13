use api::{
    Anchor, Button, Camera2D, Color, HEIGHT, Interpolation, ScreenAction, Tween, WIDTH,
    glam::{UVec2, Vec2, ivec2, uvec2},
};

use crate::{
    core::{block_kind::BlockKind, world::World},
    rendering::WorldRenderer,
    screen::{
        block_selection::BlockSelectionScreen,
        cursor::{Cursor, CursorEvent},
    },
    state::State,
};

const CAMERA_TWEEN_TICKS: usize = 18;
const FEEDBACK_DURATION_TICKS: usize = 120 * 2;

pub struct GameScreen {
    world: World,
    renderer: WorldRenderer,
    camera: Camera2D,
    camera_tween: Tween<Vec2>,
    cursor: Cursor,
    feedback: Option<(String, usize)>,
}

impl GameScreen {
    pub fn new(ctx: &mut api::ScreenContext<'_, State>) -> Result<Self, ScreenAction<State>> {
        let sprites = ctx.state.sprites.as_ref().ok_or_else(|| {
            ScreenAction::Push(Box::new(api::ErrorScreen::new(std::io::Error::other(
                "sprite store was not initialized before starting the game",
            ))))
        })?;
        let world = World::new(uvec2(128, 128))
            .map_err(|error| ScreenAction::Push(Box::new(api::ErrorScreen::new(error))))?;
        let renderer = WorldRenderer::new(ctx.assets, &world, sprites)?;
        let world_size = world.size();
        let tile_size = renderer.tile_size();
        let cursor = Cursor::new(world_size);
        let initial_camera_position =
            cursor_center(cursor.position(), tile_size, ctx.state.picked_block);
        let camera = Camera2D::new(initial_camera_position);
        let camera_tween = Tween::new(
            initial_camera_position,
            initial_camera_position,
            CAMERA_TWEEN_TICKS,
            Interpolation::EaseInOutCubic,
        );

        Ok(Self {
            cursor,
            world,
            renderer,
            camera,
            camera_tween,
            feedback: None,
        })
    }
}

impl GameScreen {
    fn update_camera(&mut self, picked_block: Option<BlockKind>) {
        let target = cursor_center(
            self.cursor.position(),
            self.renderer.tile_size(),
            picked_block,
        );
        if self.camera_tween.end() != target {
            self.camera_tween.retarget(target);
        }
        self.camera_tween.tick();
        self.camera.set_position(self.camera_tween.current());
    }
}

fn cursor_center(position: UVec2, tile_size: UVec2, picked_block: Option<BlockKind>) -> Vec2 {
    let tile_size = tile_size.as_vec2();
    let footprint = picked_block.map_or(1, |block| block.size().cells()) as f32;
    position.as_vec2() * tile_size + tile_size * (footprint * 0.5)
}

impl api::Screen<State> for GameScreen {
    fn update(&mut self, ctx: &mut api::ScreenContext<'_, State>) -> ScreenAction<State> {
        if ctx.input.just_pressed(Button::B) && ctx.state.picked_block.is_some() {
            ctx.state.picked_block = None;
            self.update_camera(None);
            return ScreenAction::None;
        }

        if ctx.input.just_pressed(Button::Y) {
            return ScreenAction::Push(Box::new(BlockSelectionScreen::new(ctx)));
        }

        let Some(sprites) = ctx.state.sprites.as_ref() else {
            api::error!("sprite store is missing during gameplay");
            return ScreenAction::Quit;
        };

        self.tick_feedback();
        match self
            .cursor
            .update(ctx.input, &mut self.world, ctx.state.picked_block)
        {
            Some(CursorEvent::Placed) => self.feedback = None,
            Some(CursorEvent::PlacementFailed(error)) => {
                let message = match error {
                    crate::core::world::PlaceBlockError::OutOfBounds { .. } => {
                        "Can't place block outside the world"
                    }
                    crate::core::world::PlaceBlockError::Occupied { .. } => {
                        "That space is occupied"
                    }
                };
                self.feedback = Some((message.to_owned(), FEEDBACK_DURATION_TICKS));
            }
            None => {}
        }
        self.world.tick();
        self.update_camera(ctx.state.picked_block);
        self.renderer
            .update(&self.world, sprites, ctx.state.picked_block);
        ScreenAction::None
    }

    fn draw(&mut self, state: &State, frame: &mut api::Frame) {
        self.renderer.draw(
            &self.camera,
            frame,
            &self.world,
            self.cursor.position(),
            state.picked_block,
            self.cursor.rotation(),
        );
        if let Some((message, _)) = &self.feedback {
            frame.text_color(
                message.clone(),
                ivec2(WIDTH as i32 / 2, HEIGHT as i32 - 4),
                Anchor::Bottom,
                Color::Mischka,
            );
        }
    }
}

impl GameScreen {
    fn tick_feedback(&mut self) {
        self.feedback = self.feedback.take().and_then(|(message, ticks)| {
            let remaining = ticks.saturating_sub(1);
            (remaining > 0).then_some((message, remaining))
        });
    }
}
