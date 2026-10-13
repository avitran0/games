mod cursor;

use std::collections::HashMap;

use api::{
    Anchor, AnimatedSprite, Assets, Camera2D, Flip, Frame, HEIGHT, ScreenAction, SpriteOptions,
    Tilemap, Tileset, WIDTH,
    glam::{IVec2, UVec2, uvec2},
};

use crate::simulation::{
    block::Rotation,
    block_kind::BlockKind,
    world::{BlockId, World, terrain::OreKind},
};
use crate::sprite_store::SpriteStore;

use self::cursor::CursorRenderer;

const ITEM_RENDER_SCALE: f32 = 0.5;

#[derive(Clone, Copy)]
pub struct CursorView {
    pub position: api::glam::UVec2,
    pub selected_block: Option<BlockKind>,
    pub rotation: Rotation,
}

fn sprite_angle(kind: BlockKind, rotation: Rotation) -> f32 {
    // Mindustry conveyor textures point right in their zero-rotation frame.
    rotation.angle()
        + if kind == BlockKind::Conveyor {
            -90.0
        } else {
            0.0
        }
}

fn intersects_view(
    position: IVec2,
    size: IVec2,
    min: api::glam::Vec2,
    max: api::glam::Vec2,
) -> bool {
    let position = position.as_vec2();
    let end = position + size.as_vec2();
    end.x >= min.x && position.x <= max.x && end.y >= min.y && position.y <= max.y
}

fn conveyor_appearance(connections: [bool; 4], rotation: Rotation) -> (usize, bool) {
    let rotation = match rotation {
        Rotation::Right => 0,
        Rotation::Up => 1,
        Rotation::Left => 2,
        Rotation::Down => 3,
    };
    let left = connections[(rotation + 3) % 4];
    let back = connections[(rotation + 2) % 4];
    let right = connections[(rotation + 1) % 4];
    let blend_case = if back && left && right {
        0
    } else if left && right {
        1
    } else if left && back {
        2
    } else if right && back {
        3
    } else if left {
        4
    } else if right {
        5
    } else {
        -1
    };
    match blend_case {
        0 => (3, false),
        1 => (4, false),
        2 => (2, true),
        3 => (2, false),
        4 => (1, true),
        5 => (1, false),
        _ => (0, false),
    }
}

pub struct WorldRenderer {
    terrain_tileset: Tileset,
    terrain_map: Tilemap,
    sprites: SpriteStore,
    ore_cells: Vec<(UVec2, OreKind, usize)>,
    block_sprites: HashMap<BlockId, AnimatedSprite>,
    conveyor_flips: HashMap<BlockId, bool>,
    preview_sprites: HashMap<BlockKind, AnimatedSprite>,
    draw_order: Vec<BlockId>,
    layout_revision: u64,
    cursor: CursorRenderer,
}

impl WorldRenderer {
    pub fn new<State>(
        assets: &mut Assets,
        world: &World,
        sprites: SpriteStore,
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

        Ok(Self {
            terrain_tileset,
            terrain_map,
            sprites,
            ore_cells,
            block_sprites: HashMap::new(),
            conveyor_flips: HashMap::new(),
            preview_sprites: HashMap::new(),
            draw_order: Vec::new(),
            layout_revision: u64::MAX,
            cursor: CursorRenderer::new(),
        })
    }

    pub fn tile_size(&self) -> api::glam::UVec2 {
        self.terrain_tileset.tile_size().as_uvec2()
    }

    pub fn update(&mut self, world: &World, selected_block: Option<BlockKind>, tick: usize) {
        if let Some(kind) = selected_block {
            self.preview_sprites
                .entry(kind)
                .or_insert_with(|| self.sprites.block(kind))
                .set_tick(tick as u16);
        }

        if self.layout_revision != world.layout_revision() {
            self.layout_revision = world.layout_revision();
            self.draw_order = world.blocks().map(|(id, _)| id).collect();
            self.draw_order.sort_by_key(|id| {
                world
                    .block(*id)
                    .map(|block| (block.origin().y, block.origin().x))
            });
            self.block_sprites.retain(|id, _| world.contains_block(*id));
            self.conveyor_flips
                .retain(|id, _| world.contains_block(*id));

            for (id, block) in world.blocks() {
                if block.kind() == BlockKind::Conveyor {
                    let (variant, flip) =
                        conveyor_appearance(world.conveyor_connections(id), block.rotation());
                    self.conveyor_flips.insert(id, flip);
                    self.block_sprites
                        .insert(id, self.sprites.conveyor_variant(variant));
                } else {
                    self.conveyor_flips.remove(&id);
                    self.block_sprites
                        .entry(id)
                        .or_insert_with(|| self.sprites.block(block.kind()));
                }
            }
        }

        for sprite in self.block_sprites.values_mut() {
            sprite.set_tick(tick as u16);
        }
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
        let screen_a = camera.screen_to_world(IVec2::ZERO);
        let screen_b = camera.screen_to_world(IVec2::new(WIDTH as i32, HEIGHT as i32));
        let visible_min = screen_a.min(screen_b);
        let visible_max = screen_a.max(screen_b);
        for (position, ore, variant) in &self.ore_cells {
            let center = position.as_ivec2() * tile_size + tile_size / 2;
            let center_f = center.as_vec2();
            if center_f.cmplt(visible_min).any() || center_f.cmpgt(visible_max).any() {
                continue;
            }
            let texture = self.sprites.ore_texture(*ore, *variant);
            frame.world(camera).sprite(
                texture,
                center,
                SpriteOptions::default()
                    .anchor(Anchor::Center)
                    .opacity(0.95),
            );
        }
        for id in &self.draw_order {
            let Some(block) = world.block(*id) else {
                continue;
            };
            let Some(sprite) = self.block_sprites.get(id) else {
                continue;
            };
            let block_size = tile_size * block.footprint_cells() as i32;
            let origin = block.origin().as_ivec2() * tile_size;
            if !intersects_view(origin, block_size, visible_min, visible_max) {
                continue;
            }
            let center = origin + block_size / 2;
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
        for id in &self.draw_order {
            let Some(block) = world.block(*id) else {
                continue;
            };
            if let Some(stack) = block.items() {
                let block_size = tile_size * block.footprint_cells() as i32;
                let origin = block.origin().as_ivec2() * tile_size;
                if !intersects_view(origin, block_size, visible_min, visible_max) {
                    continue;
                }
                let center = origin + block_size / 2;
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
                    self.sprites.item(stack.item()),
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
            CursorView {
                position: cursor_position,
                selected_block,
                rotation,
            },
            &self.sprites,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{conveyor_appearance, sprite_angle};
    use crate::simulation::{block::Rotation, block_kind::BlockKind};

    #[test]
    fn conveyor_blend_atlas_transform_matches_world_connections() {
        assert_eq!(
            conveyor_appearance([true, false, false, true], Rotation::Right),
            (1, true)
        );
    }

    #[test]
    fn mindustry_conveyor_texture_rotates_to_the_simulated_output() {
        assert_eq!(sprite_angle(BlockKind::Conveyor, Rotation::Right), 0.0);
        assert_eq!(sprite_angle(BlockKind::Conveyor, Rotation::Down), 90.0);
        assert_eq!(sprite_angle(BlockKind::Conveyor, Rotation::Left), 180.0);
        assert_eq!(sprite_angle(BlockKind::Conveyor, Rotation::Up), -90.0);
    }
}
