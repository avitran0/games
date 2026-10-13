use api::{
    Button, Camera2D, Interpolation, ScreenAction, Tween,
    glam::{UVec2, Vec2, uvec2},
};

use crate::{
    core::{block_kind::BlockKind, world::World},
    rendering::WorldRenderer,
    screen::{
        block_selection::BlockSelectionScreen,
        cursor::{Cursor, CursorEvent},
        pause::PauseScreen,
    },
    state::State,
};

const CAMERA_TWEEN_TICKS: usize = 18;

pub struct GameScreen {
    world: World,
    renderer: WorldRenderer,
    camera: Camera2D,
    camera_tween: Tween<Vec2>,
    cursor: Cursor,
    simulation_tick: usize,
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
            simulation_tick: 0,
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
        if ctx.input.just_pressed(Button::Start) {
            return ScreenAction::Push(Box::new(PauseScreen::new()));
        }

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

        self.simulation_tick = self.simulation_tick.wrapping_add(1);
        match self
            .cursor
            .update(ctx.input, &mut self.world, ctx.state.picked_block)
        {
            Some(CursorEvent::Placed) => {}
            Some(CursorEvent::PlacementFailed(error)) => {
                api::warn!("block placement failed: {error:?}");
            }
            None => {}
        }
        self.world.tick(self.simulation_tick);
        self.update_camera(ctx.state.picked_block);
        self.renderer.update(
            &self.world,
            sprites,
            ctx.state.picked_block,
            self.simulation_tick,
        );
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
            self.simulation_tick,
        );
    }
}
