mod cursor;

use std::collections::HashMap;

use api::{
    Anchor, AnimatedSprite, Assets, Camera2D, Frame, ScreenAction, Sprite, SpriteOptions, Tilemap,
    Tileset,
    glam::{IVec2, uvec2},
};

use crate::core::{
    block::Rotation,
    block_kind::BlockKind,
    item::Item,
    world::{BlockId, World, terrain::OreKind},
};
use crate::sprite_store::SpriteStore;

use self::cursor::CursorRenderer;

pub struct WorldRenderer {
    terrain_tileset: Tileset,
    terrain_map: Tilemap,
    iron_ore: Sprite,
    ore_cells: Vec<api::glam::UVec2>,
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

        let mut ore_cells = Vec::new();
        for y in 0..world.size().y {
            for x in 0..world.size().x {
                let Some(terrain) = world.terrain_at(uvec2(x, y)) else {
                    continue;
                };
                let tile_id = terrain.ground.tile_id() + 1;
                if terrain.ore == Some(OreKind::Iron) {
                    ore_cells.push(uvec2(x, y));
                }
                assets
                    .set_tile(&terrain_map, uvec2(x, y), tile_id)
                    .map_err(|error| ScreenAction::Push(Box::new(api::ErrorScreen::new(error))))?;
            }
        }

        let cursor = CursorRenderer::new(sprites);
        Ok(Self {
            terrain_tileset,
            terrain_map,
            iron_ore: sprites.iron_ore,
            ore_cells,
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
        tick: usize,
    ) {
        if let Some(block) = selected_block {
            self.preview_sprites
                .entry(block)
                .or_insert_with(|| sprites.block(block))
                .set_tick(tick as u16);
        }

        for (id, block) in world.blocks() {
            let sprite = self
                .block_sprites
                .entry(id)
                .or_insert_with(|| sprites.block(block.kind()));
            sprite.set_tick(tick as u16);
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
        tick: usize,
    ) {
        frame.world(camera).tilemap(&self.terrain_map, IVec2::ZERO);

        let tile_size = self.tile_size().as_ivec2();
        for ore in &self.ore_cells {
            let center = ore.as_ivec2() * tile_size + tile_size / 2;
            frame.world(camera).sprite(
                &self.iron_ore,
                center,
                SpriteOptions::default()
                    .anchor(Anchor::Center)
                    .uniform_scale(0.65)
                    .opacity(0.85),
            );
        }
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
            if let Some(stack) = block.items()
                && stack.item() == Item::IronOre
            {
                let item_center = block
                    .item_progress(tick)
                    .map(|progress| {
                        let output_direction = block.rotation().direction();
                        let input_direction =
                            block.item_input_direction().unwrap_or(-output_direction);
                        let half_span = if output_direction.x == 0 {
                            tile_size.y / 2
                        } else {
                            tile_size.x / 2
                        };
                        let input = center - input_direction * half_span;
                        let output = center + output_direction * half_span;
                        input
                            .as_vec2()
                            .lerp(output.as_vec2(), progress)
                            .round()
                            .as_ivec2()
                    })
                    .unwrap_or(center);
                frame.world(camera).sprite(
                    &self.iron_ore,
                    item_center,
                    SpriteOptions::default()
                        .anchor(Anchor::Center)
                        .uniform_scale(0.65),
                );
            }
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
