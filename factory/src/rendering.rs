use api::{
    Assets, Camera2D, Frame, ScreenAction, Tilemap, Tileset,
    glam::{IVec2, uvec2},
};

use crate::core::world::World;

pub struct WorldRenderer {
    terrain_tileset: Tileset,
    terrain_map: Tilemap,
}

impl WorldRenderer {
    pub fn new<State>(assets: &mut Assets, world: &World) -> Result<Self, ScreenAction<State>> {
        let terrain_tileset = assets.load_tileset(include_bytes!("assets/terrain.pxt"))?;
        let terrain_map = assets
            .create_tilemap(world.size(), &terrain_tileset)
            .expect("world dimensions must fit in a tilemap");

        for y in 0..world.size().y {
            for x in 0..world.size().x {
                let Some(terrain) = world.terrain_at(uvec2(x, y)) else {
                    continue;
                };
                let tile_id = terrain.ground.tile_id(x, y) + 1;
                assets
                    .set_tile(&terrain_map, uvec2(x, y), tile_id)
                    .expect("terrain tile IDs must exist in the terrain tileset");
            }
        }

        Ok(Self {
            terrain_tileset,
            terrain_map,
        })
    }

    pub fn tile_size(&self) -> api::glam::UVec2 {
        self.terrain_tileset.tile_size().as_uvec2()
    }

    pub fn draw(&self, camera: &Camera2D, frame: &mut Frame) {
        frame.world(camera).tilemap(&self.terrain_map, IVec2::ZERO);
    }
}
