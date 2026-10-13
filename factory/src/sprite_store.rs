use std::collections::HashMap;

use api::{AnimatedSprite, Assets, ScreenAction, Sprite};

use crate::{
    core::{block_kind::BlockKind, world::terrain::OreKind},
    state::State,
};

const BLOCK_ANIMATION_DIVISOR: u16 = 7;
const CONVEYOR_VARIANT_COUNT: usize = 5;

pub struct SpriteStore {
    blocks: HashMap<BlockKind, AnimatedSprite>,
    conveyor_variants: [AnimatedSprite; CONVEYOR_VARIANT_COUNT],
    ore_textures: HashMap<OreKind, [Sprite; 3]>,
    pub iron_ore: Sprite,
    pub iron_plate: Sprite,
    pub coal: Sprite,
    pub titanium: Sprite,
    pub cursor: CursorSprites,
}

pub struct CursorSprites {
    pub single: Sprite,
    pub double: Sprite,
    pub triple: Sprite,
    pub quad: Sprite,
}

impl SpriteStore {
    pub fn load(assets: &mut Assets) -> Result<Self, ScreenAction<State>> {
        let mut blocks = HashMap::new();
        for kind in BlockKind::iter() {
            if kind != BlockKind::Conveyor {
                blocks.insert(kind, assets.load_animated_sprite(block_asset(kind))?);
            }
        }
        let conveyor_variants = [
            assets.load_animated_sprite(conveyor_asset(0))?,
            assets.load_animated_sprite(conveyor_asset(1))?,
            assets.load_animated_sprite(conveyor_asset(2))?,
            assets.load_animated_sprite(conveyor_asset(3))?,
            assets.load_animated_sprite(conveyor_asset(4))?,
        ];

        let ore_textures = HashMap::from([
            (
                OreKind::Iron,
                [
                    assets.load_sprite(include_bytes!("assets/ores/iron-0.pxs"))?,
                    assets.load_sprite(include_bytes!("assets/ores/iron-1.pxs"))?,
                    assets.load_sprite(include_bytes!("assets/ores/iron-2.pxs"))?,
                ],
            ),
            (
                OreKind::Coal,
                [
                    assets.load_sprite(include_bytes!("assets/ores/coal-0.pxs"))?,
                    assets.load_sprite(include_bytes!("assets/ores/coal-1.pxs"))?,
                    assets.load_sprite(include_bytes!("assets/ores/coal-2.pxs"))?,
                ],
            ),
            (
                OreKind::Titanium,
                [
                    assets.load_sprite(include_bytes!("assets/ores/titanium-0.pxs"))?,
                    assets.load_sprite(include_bytes!("assets/ores/titanium-1.pxs"))?,
                    assets.load_sprite(include_bytes!("assets/ores/titanium-2.pxs"))?,
                ],
            ),
        ]);

        Ok(Self {
            blocks,
            conveyor_variants,
            ore_textures,
            iron_ore: assets.load_sprite(include_bytes!("assets/item/iron.pxs"))?,
            iron_plate: assets.load_sprite(include_bytes!("assets/item/iron_plate.pxs"))?,
            coal: assets.load_sprite(include_bytes!("assets/item/coal.pxs"))?,
            titanium: assets.load_sprite(include_bytes!("assets/item/titanium.pxs"))?,
            cursor: CursorSprites {
                single: assets.load_sprite(include_bytes!("assets/cursor/single.pxs"))?,
                double: assets.load_sprite(include_bytes!("assets/cursor/double.pxs"))?,
                triple: assets.load_sprite(include_bytes!("assets/cursor/triple.pxs"))?,
                quad: assets.load_sprite(include_bytes!("assets/cursor/quad.pxs"))?,
            },
        })
    }

    pub fn block(&self, kind: BlockKind) -> AnimatedSprite {
        if kind == BlockKind::Conveyor {
            return self.conveyor_variant(0);
        }
        self.prepare(self.blocks[&kind].clone())
    }

    pub fn conveyor_variant(&self, variant: usize) -> AnimatedSprite {
        let sprite = self
            .conveyor_variants
            .get(variant)
            .unwrap_or(&self.conveyor_variants[0])
            .clone();
        self.prepare(sprite)
    }

    fn prepare(&self, mut sprite: AnimatedSprite) -> AnimatedSprite {
        sprite.set_animation("default");
        sprite.set_divisor(BLOCK_ANIMATION_DIVISOR);
        sprite
    }

    pub fn ore_texture(&self, ore: OreKind, variant: usize) -> &Sprite {
        &self.ore_textures[&ore][variant % 3]
    }

    pub fn item(&self, item: crate::core::item::Item) -> &Sprite {
        use crate::core::item::Item;
        match item {
            Item::IronOre => &self.iron_ore,
            Item::IronPlate => &self.iron_plate,
            Item::Coal => &self.coal,
            Item::TitaniumOre => &self.titanium,
        }
    }
}

fn conveyor_asset(variant: usize) -> &'static [u8] {
    match variant {
        0 => include_bytes!("assets/blocks/conveyor-0.pxa"),
        1 => include_bytes!("assets/blocks/conveyor-1.pxa"),
        2 => include_bytes!("assets/blocks/conveyor-2.pxa"),
        3 => include_bytes!("assets/blocks/conveyor-3.pxa"),
        _ => include_bytes!("assets/blocks/conveyor-4.pxa"),
    }
}

fn block_asset(kind: BlockKind) -> &'static [u8] {
    match kind {
        BlockKind::Conveyor => unreachable!("conveyors are loaded from their blend variants"),
        BlockKind::Drill => include_bytes!("assets/blocks/drill.pxa"),
        BlockKind::Storage => include_bytes!("assets/blocks/storage.pxa"),
        BlockKind::Smelter => include_bytes!("assets/blocks/smelter.pxa"),
        BlockKind::SolarPanel => include_bytes!("assets/blocks/solar_panel.pxa"),
        BlockKind::Core => include_bytes!("assets/blocks/core.pxa"),
    }
}

#[cfg(test)]
mod tests {
    use super::{BlockKind, CONVEYOR_VARIANT_COUNT, block_asset, conveyor_asset};
    use api::glam::uvec2;
    use formats::{AnimatedSpriteDocument, SpriteDocument, TilesetDocument};

    #[test]
    fn mindustry_derived_assets_decode_at_sixteen_pixels_per_tile() {
        for kind in BlockKind::iter().filter(|kind| *kind != BlockKind::Conveyor) {
            let sprite = AnimatedSpriteDocument::decode(block_asset(kind))
                .unwrap_or_else(|error| panic!("{} sprite is invalid: {error}", kind.label()));
            assert!(sprite.frames.len() >= 2, "{} should animate", kind.label());
            let expected = kind.size().cells() * 16;
            assert_eq!(sprite.size, uvec2(expected, expected));
        }

        for variant in 0..CONVEYOR_VARIANT_COUNT {
            let sprite = AnimatedSpriteDocument::decode(conveyor_asset(variant))
                .expect("conveyor blend sprite should decode");
            assert_eq!(sprite.size, uvec2(16, 16));
            assert_eq!(sprite.frames.len(), 4);
        }

        for asset in [
            include_bytes!("assets/item/iron.pxs").as_slice(),
            include_bytes!("assets/item/iron_plate.pxs").as_slice(),
            include_bytes!("assets/item/coal.pxs").as_slice(),
            include_bytes!("assets/item/titanium.pxs").as_slice(),
            include_bytes!("assets/ores/iron-0.pxs").as_slice(),
            include_bytes!("assets/ores/iron-1.pxs").as_slice(),
            include_bytes!("assets/ores/iron-2.pxs").as_slice(),
            include_bytes!("assets/ores/coal-0.pxs").as_slice(),
            include_bytes!("assets/ores/coal-1.pxs").as_slice(),
            include_bytes!("assets/ores/coal-2.pxs").as_slice(),
            include_bytes!("assets/ores/titanium-0.pxs").as_slice(),
            include_bytes!("assets/ores/titanium-1.pxs").as_slice(),
            include_bytes!("assets/ores/titanium-2.pxs").as_slice(),
        ] {
            let sprite = SpriteDocument::decode(asset).expect("item sprite should decode");
            assert_eq!(sprite.size, uvec2(16, 16));
        }

        let terrain = TilesetDocument::decode(include_bytes!("assets/terrain.pxt"))
            .expect("terrain tileset should decode");
        assert_eq!(terrain.tile_size, uvec2(16, 16));
        assert_eq!(terrain.tiles.len(), 3);

        for (name, size) in [("single", 16), ("double", 32), ("triple", 48), ("quad", 64)] {
            let bytes = match name {
                "single" => include_bytes!("assets/cursor/single.pxs").as_slice(),
                "double" => include_bytes!("assets/cursor/double.pxs").as_slice(),
                "triple" => include_bytes!("assets/cursor/triple.pxs").as_slice(),
                _ => include_bytes!("assets/cursor/quad.pxs").as_slice(),
            };
            let sprite = SpriteDocument::decode(bytes).expect("cursor sprite should decode");
            assert_eq!(sprite.size, uvec2(size, size));
        }
    }
}
