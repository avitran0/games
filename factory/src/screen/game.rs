use api::{
    Camera2D, Color, HEIGHT, WIDTH,
    glam::{IVec2, ivec2, uvec2, vec2},
};

use crate::{
    core::world::{World, terrain::GroundKind},
    state::State,
};

const TILE_SIZE: i32 = 16;

pub struct GameScreen {
    world: World,
    camera: Camera2D,
}

impl GameScreen {
    pub fn new() -> Self {
        let world = World::new(uvec2(128, 128));
        let world_size = world.size();
        let camera = Camera2D::new(vec2(
            world_size.x as f32 * TILE_SIZE as f32 / 2.0,
            world_size.y as f32 * TILE_SIZE as f32 / 2.0,
        ));

        Self { world, camera }
    }
}

impl api::Screen<State> for GameScreen {
    fn update(&mut self, _ctx: &mut api::ScreenContext<'_, State>) -> api::ScreenAction<State> {
        api::ScreenAction::None
    }

    fn draw(&mut self, _state: &State, frame: &mut api::Frame) {
        let world_size = self.world.size();
        let min_world = self.camera.screen_to_world(ivec2(0, 0));
        let max_world = self
            .camera
            .screen_to_world(ivec2(WIDTH as i32, HEIGHT as i32));
        let min_tile = ivec2(
            (min_world.x / TILE_SIZE as f32).floor() as i32,
            (min_world.y / TILE_SIZE as f32).floor() as i32,
        )
        .max(IVec2::ZERO);
        let max_tile = ivec2(
            (max_world.x / TILE_SIZE as f32).ceil() as i32,
            (max_world.y / TILE_SIZE as f32).ceil() as i32,
        )
        .min(ivec2(world_size.x as i32, world_size.y as i32));

        let mut world_frame = frame.world(&self.camera);
        for y in min_tile.y..max_tile.y {
            for x in min_tile.x..max_tile.x {
                let Some(terrain) = self.world.terrain_at(uvec2(x as u32, y as u32)) else {
                    continue;
                };
                let color = match terrain.ground {
                    GroundKind::Sand => Color::Chalky,
                    GroundKind::Water => Color::EasternBlue,
                };
                world_frame.rectangle(
                    ivec2(x * TILE_SIZE, y * TILE_SIZE),
                    uvec2(TILE_SIZE as u32, TILE_SIZE as u32),
                    color,
                );
            }
        }
    }
}
