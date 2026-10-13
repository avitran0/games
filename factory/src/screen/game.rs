use api::{
    Camera2D, ScreenAction,
    glam::{uvec2, vec2},
};

use crate::{core::world::World, rendering::WorldRenderer, state::State};

pub struct GameScreen {
    world: World,
    renderer: WorldRenderer,
    camera: Camera2D,
}

impl GameScreen {
    pub fn new(ctx: &mut api::ScreenContext<'_, State>) -> Result<Self, ScreenAction<State>> {
        let world = World::new(uvec2(128, 128));
        let renderer = WorldRenderer::new(&mut ctx.assets, &world)?;
        let world_size = world.size();
        let tile_size = renderer.tile_size();
        let camera = Camera2D::new(vec2(
            world_size.x as f32 * tile_size.x as f32 / 2.0,
            world_size.y as f32 * tile_size.y as f32 / 2.0,
        ));

        Ok(Self {
            world,
            renderer,
            camera,
        })
    }
}

impl api::Screen<State> for GameScreen {
    fn update(&mut self, _ctx: &mut api::ScreenContext<'_, State>) -> ScreenAction<State> {
        ScreenAction::None
    }

    fn draw(&mut self, _state: &State, frame: &mut api::Frame) {
        self.renderer.draw(&self.camera, frame);
    }
}
