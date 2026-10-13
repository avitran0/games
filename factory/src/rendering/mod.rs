mod cursor;

use std::collections::HashMap;

use api::{
    Anchor, AnimatedSprite, Assets, Camera2D, Flip, Frame, ScreenAction, Sprite, SpriteOptions,
    Tilemap, Tileset,
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

const ITEM_RENDER_SCALE: f32 = 0.5;

pub struct CursorView {
    pub position: api::glam::UVec2,
    pub selected_block: Option<BlockKind>,
    pub rotation: Rotation,
}

fn sprite_angle(kind: BlockKind, rotation: Rotation) -> f32 {
    let angle = match rotation {
        Rotation::Up => 0.0,
        Rotation::Right => 90.0,
        Rotation::Down => 180.0,
        Rotation::Left => 270.0,
    };
    // Mindustry conveyor textures point right in their zero-rotation frame.
    angle
        + if kind == BlockKind::Conveyor {
            -90.0
        } else {
            0.0
        }
}

pub struct WorldRenderer {
    terrain_tileset: Tileset,
    terrain_map: Tilemap,
    item_sprites: HashMap<Item, Sprite>,
    ore_textures: HashMap<OreKind, [Sprite; 3]>,
    ore_cells: Vec<(api::glam::UVec2, OreKind, usize)>,
    block_sprites: HashMap<BlockId, AnimatedSprite>,
    conveyor_flips: HashMap<BlockId, bool>,
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
                if let Some(ore) = terrain.ore {
                    let variant = (x.wrapping_mul(17) + y.wrapping_mul(31)) as usize % 3;
                    ore_cells.push((uvec2(x, y), ore, variant));
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
            ore_textures: HashMap::from([
                (
                    OreKind::Iron,
                    std::array::from_fn(|variant| *sprites.ore_texture(OreKind::Iron, variant)),
                ),
                (
                    OreKind::Coal,
                    std::array::from_fn(|variant| *sprites.ore_texture(OreKind::Coal, variant)),
                ),
                (
                    OreKind::Titanium,
                    std::array::from_fn(|variant| *sprites.ore_texture(OreKind::Titanium, variant)),
                ),
            ]),
            item_sprites: [
                Item::IronOre,
                Item::IronPlate,
                Item::Coal,
                Item::TitaniumOre,
            ]
            .into_iter()
            .map(|item| (item, *sprites.item(item)))
            .collect(),
            ore_cells,
            block_sprites: HashMap::new(),
            conveyor_flips: HashMap::new(),
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
            let mut sprite = if block.kind() == BlockKind::Conveyor {
                let (variant, flip_y) = world.conveyor_appearance(id);
                self.conveyor_flips.insert(id, flip_y);
                sprites.conveyor_variant(variant)
            } else {
                self.conveyor_flips.remove(&id);
                sprites.block(block.kind())
            };
            sprite.set_tick(tick as u16);
            self.block_sprites.insert(id, sprite);
        }

        self.block_sprites.retain(|id, _| world.contains_block(*id));
        self.conveyor_flips
            .retain(|id, _| world.contains_block(*id));
    }

    pub fn draw(
        &self,
        camera: &Camera2D,
        frame: &mut Frame,
        world: &World,
        cursor_view: CursorView,
        tick: usize,
    ) {
        frame.world(camera).tilemap(&self.terrain_map, IVec2::ZERO);

        let tile_size = self.tile_size().as_ivec2();
        for (position, ore, variant) in &self.ore_cells {
            let center = position.as_ivec2() * tile_size + tile_size / 2;
            let texture = self.ore_textures[ore][*variant];
            frame.world(camera).sprite(
                &texture,
                center,
                SpriteOptions::default()
                    .anchor(Anchor::Center)
                    .opacity(0.95),
            );
        }
        let mut blocks = world.blocks().collect::<Vec<_>>();
        blocks.sort_by_key(|(_, block)| (block.origin().y, block.origin().x));
        for (id, block) in &blocks {
            let Some(sprite) = self.block_sprites.get(id) else {
                continue;
            };
            let block_size = tile_size * block.footprint_cells() as i32;
            let center = block.origin().as_ivec2() * tile_size + block_size / 2;
            let angle = sprite_angle(block.kind(), block.rotation());
            let options = SpriteOptions::default()
                .anchor(Anchor::Center)
                .rotation(angle)
                .rotation_anchor(Anchor::Center);
            let options = if self.conveyor_flips.get(id).copied().unwrap_or(false) {
                options.flip(Flip::Vertical)
            } else {
                options
            };
            frame.world(camera).animated_sprite(sprite, center, options);
        }

        // Draw moving item icons above every belt so neighboring sprites cannot cover them.
        for (_, block) in blocks {
            if let Some(stack) = block.items() {
                let block_size = tile_size * block.footprint_cells() as i32;
                let center = block.origin().as_ivec2() * tile_size + block_size / 2;
                let item_center = block
                    .item_progress(tick)
                    .map(|progress| {
                        let output_direction = block.output_direction();
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
                    &self.item_sprites[&stack.item()],
                    item_center,
                    SpriteOptions::default()
                        .anchor(Anchor::Center)
                        .uniform_scale(ITEM_RENDER_SCALE),
                );
            }
        }

        let CursorView {
            position: cursor_position,
            selected_block,
            rotation,
        } = cursor_view;
        if let Some(selected_block) = selected_block {
            let block_size = tile_size * selected_block.size().cells() as i32;
            let preview_center = cursor_position.as_ivec2() * tile_size + block_size / 2;
            let preview_angle = sprite_angle(selected_block, rotation);
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

#[cfg(test)]
mod tests {
    use super::sprite_angle;
    use crate::core::{block::Rotation, block_kind::BlockKind};

    #[test]
    fn mindustry_conveyor_texture_rotates_to_the_simulated_output() {
        assert_eq!(sprite_angle(BlockKind::Conveyor, Rotation::Right), 0.0);
        assert_eq!(sprite_angle(BlockKind::Conveyor, Rotation::Down), 90.0);
        assert_eq!(sprite_angle(BlockKind::Conveyor, Rotation::Left), 180.0);
        assert_eq!(sprite_angle(BlockKind::Conveyor, Rotation::Up), -90.0);
    }
}
