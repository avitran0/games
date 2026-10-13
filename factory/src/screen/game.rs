use api::{
    Button, Camera2D, Interpolation, ScreenAction, Tween,
    glam::{UVec2, Vec2, uvec2},
};

use crate::{
    rendering::{CursorView, WorldRenderer},
    screen::{block_selection::BlockSelectionScreen, cursor::Cursor, pause::PauseScreen},
    simulation::{block::Rotation, block_kind::BlockKind, world::World},
    sprite_store::SpriteStore,
    state::State,
};

const CAMERA_TWEEN_TICKS: usize = 18;

pub struct GameScreen {
    world: World,
    renderer: WorldRenderer,
    camera: Camera2D,
    camera_tween: Tween<Vec2>,
    cursor: Cursor,
    placement_feedback: Option<(String, usize)>,
}

impl GameScreen {
    pub fn new(ctx: &mut api::ScreenContext<'_, State>) -> Result<Self, ScreenAction<State>> {
        let sprites = SpriteStore::load(ctx.assets)?;
        let mut world = World::new(uvec2(128, 128))
            .map_err(|error| ScreenAction::Push(Box::new(api::ErrorScreen::new(error))))?;
        let core_origin = world.size() / 2 - UVec2::splat(2);
        world
            .place_block(BlockKind::Core, core_origin, Rotation::Up)
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
            placement_feedback: None,
        })
    }

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
            self.world.tick();
            self.update_camera(None);
            let tick = self.world.tick_count().saturating_sub(1);
            self.renderer.update(&self.world, None, tick);
            return ScreenAction::None;
        }

        if ctx.input.just_pressed(Button::Y) {
            return ScreenAction::Push(Box::new(BlockSelectionScreen::new(ctx)));
        }

        if let Some((_, ticks)) = &mut self.placement_feedback {
            *ticks = ticks.saturating_sub(1);
            if *ticks == 0 {
                self.placement_feedback = None;
            }
        }
        if let Some(result) = self
            .cursor
            .update(ctx.input, &mut self.world, ctx.state.picked_block)
        {
            self.placement_feedback = result
                .err()
                .map(|error| (error.to_string(), api::FPS as usize));
        }
        self.world.tick();
        self.update_camera(ctx.state.picked_block);
        let tick = self.world.tick_count().saturating_sub(1);
        self.renderer
            .update(&self.world, ctx.state.picked_block, tick);
        ScreenAction::None
    }

    fn draw(&mut self, state: &State, frame: &mut api::Frame) {
        self.renderer.draw(
            &self.camera,
            frame,
            &self.world,
            CursorView {
                position: self.cursor.position(),
                selected_block: state.picked_block,
                rotation: self.cursor.rotation(),
            },
            self.world.tick_count().saturating_sub(1),
        );

        frame.rectangle(
            api::glam::IVec2::ZERO,
            uvec2(api::WIDTH, 12),
            api::Color::Cinder,
        );
        frame.text(
            format!(
                "Iron {}  Plates {}",
                self.world.stockpile(crate::simulation::item::Item::IronOre),
                self.world
                    .stockpile(crate::simulation::item::Item::IronPlate)
            ),
            api::glam::IVec2::new(5, 3),
            api::Anchor::TopLeft,
        );
        if let Some((message, _)) = &self.placement_feedback {
            frame.text(message, api::glam::IVec2::new(5, 14), api::Anchor::TopLeft);
        }
    }
}
