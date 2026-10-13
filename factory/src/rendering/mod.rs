mod cursor;

use std::collections::HashMap;

use api::{
    Anchor, AnimatedSprite, Assets, Camera2D, Frame, ScreenAction, SpriteOptions, Tilemap, Tileset,
    glam::{IVec2, uvec2},
};

use crate::core::{
    block::Rotation,
    block_kind::BlockKind,
    world::{BlockId, World},
};
use crate::sprite_store::SpriteStore;

use self::cursor::CursorRenderer;

pub struct WorldRenderer {
    terrain_tileset: Tileset,
    terrain_map: Tilemap,
    block_sprites: HashMap<BlockId, AnimatedSprite>,
    preview_sprites: HashMap<BlockKind, AnimatedSprite>,
    cursor: CursorRenderer,
}

impl WorldRenderer {
    pub fn new<State>(
        assets: &mut Assets,
        world: &World,
        sprites: &SpriteStore,
    ) -> Result<Self, ScreenAction<State>> {
        let terrain_tileset = assets.load_tileset(include_bytes!("../assets/terrain.pxt"))?;
        let terrain_map = assets
            .create_tilemap(world.size(), &terrain_tileset)
            .map_err(|error| ScreenAction::Push(Box::new(api::ErrorScreen::new(error))))?;

        for y in 0..world.size().y {
            for x in 0..world.size().x {
                let Some(terrain) = world.terrain_at(uvec2(x, y)) else {
                    continue;
                };
                let tile_id = terrain.ground.tile_id() + 1;
                assets
                    .set_tile(&terrain_map, uvec2(x, y), tile_id)
                    .map_err(|error| ScreenAction::Push(Box::new(api::ErrorScreen::new(error))))?;
            }
        }

        let cursor = CursorRenderer::new(sprites);
        Ok(Self {
            terrain_tileset,
            terrain_map,
            block_sprites: HashMap::new(),
            preview_sprites: HashMap::new(),
            cursor,
        })
    }

    pub fn tile_size(&self) -> api::glam::UVec2 {
        self.terrain_tileset.tile_size().as_uvec2()
    }

    pub fn update(
        &mut self,
        world: &World,
        sprites: &SpriteStore,
        selected_block: Option<BlockKind>,
    ) {
        if let Some(block) = selected_block {
            self.preview_sprites
                .entry(block)
                .or_insert_with(|| sprites.block(block))
                .tick();
        }

        for (id, block) in world.blocks() {
            let sprite = self
                .block_sprites
                .entry(id)
                .or_insert_with(|| sprites.block(block.kind()));
            sprite.tick();
        }

        self.block_sprites.retain(|id, _| world.contains_block(*id));
    }

    pub fn draw(
        &self,
        camera: &Camera2D,
        frame: &mut Frame,
        world: &World,
        cursor_position: api::glam::UVec2,
        selected_block: Option<BlockKind>,
        rotation: Rotation,
    ) {
        frame.world(camera).tilemap(&self.terrain_map, IVec2::ZERO);

        let tile_size = self.tile_size().as_ivec2();
        for (id, block) in world.blocks() {
            let Some(sprite) = self.block_sprites.get(&id) else {
                continue;
            };
            let block_size = tile_size * block.footprint_cells() as i32;
            let center = block.origin().as_ivec2() * tile_size + block_size / 2;
            let angle = match block.rotation() {
                Rotation::Up => 0.0,
                Rotation::Right => 90.0,
                Rotation::Down => 180.0,
                Rotation::Left => 270.0,
            };
            frame.world(camera).animated_sprite(
                sprite,
                center,
                SpriteOptions::default()
                    .anchor(Anchor::Center)
                    .rotation(angle)
                    .rotation_anchor(Anchor::Center),
            );
        }

        if let Some(selected_block) = selected_block {
            let block_size = tile_size * selected_block.size().cells() as i32;
            let preview_center = cursor_position.as_ivec2() * tile_size + block_size / 2;
            let preview_angle = match rotation {
                Rotation::Up => 0.0,
                Rotation::Right => 90.0,
                Rotation::Down => 180.0,
                Rotation::Left => 270.0,
            };
            if let Some(preview) = self.preview_sprites.get(&selected_block) {
                frame.world(camera).animated_sprite(
                    preview,
                    preview_center,
                    SpriteOptions::default()
                        .anchor(Anchor::Center)
                        .rotation(preview_angle)
                        .rotation_anchor(Anchor::Center)
                        .opacity(0.5),
                );
            }
        }

        self.cursor.draw(
            camera,
            frame,
            self.tile_size(),
            cursor_position,
            selected_block,
            rotation,
        );
    }
}
