use std::collections::HashMap;

use strum::IntoEnumIterator;

use api::{AnimatedSprite, Assets, ScreenAction, Sprite};

use crate::{
    simulation::{block_kind::BlockKind, item::Item, world::terrain::OreKind},
    state::State,
};

const BLOCK_ANIMATION_DIVISOR: u16 = 7;
const CONVEYOR_VARIANT_COUNT: usize = 5;
const ORE_VARIANT_COUNT: usize = 3;

pub struct SpriteStore {
    blocks: HashMap<BlockKind, AnimatedSprite>,
    conveyor_variants: [AnimatedSprite; CONVEYOR_VARIANT_COUNT],
    ore_textures: [[Sprite; ORE_VARIANT_COUNT]; <OreKind as strum::EnumCount>::COUNT],
    items: [Sprite; Item::COUNT],
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

        let ore_textures = OreKind::iter()
            .map(|ore| {
                Ok([
                    assets.load_sprite(ore_asset(ore, 0))?,
                    assets.load_sprite(ore_asset(ore, 1))?,
                    assets.load_sprite(ore_asset(ore, 2))?,
                ])
            })
            .collect::<Result<Vec<_>, ScreenAction<State>>>()?
            .try_into()
            .unwrap_or_else(|_| unreachable!("ore enum count and iterator must agree"));
        let items = Item::iter()
            .map(|item| assets.load_sprite(item_asset(item)))
            .collect::<Result<Vec<_>, _>>()?
            .try_into()
            .unwrap_or_else(|_| unreachable!("item enum count and iterator must agree"));

        Ok(Self {
            blocks,
            conveyor_variants,
            ore_textures,
            items,
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
        &self.ore_textures[ore.index()][variant % ORE_VARIANT_COUNT]
    }

    pub fn item(&self, item: Item) -> &Sprite {
        &self.items[item.index()]
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
        BlockKind::Core => include_bytes!("assets/blocks/core.pxa"),
    }
}

fn item_asset(item: Item) -> &'static [u8] {
    match item {
        Item::IronOre => include_bytes!("assets/item/iron.pxs"),
        Item::IronPlate => include_bytes!("assets/item/iron_plate.pxs"),
        Item::Coal => include_bytes!("assets/item/coal.pxs"),
        Item::TitaniumOre => include_bytes!("assets/item/titanium.pxs"),
    }
}

fn ore_asset(ore: OreKind, variant: usize) -> &'static [u8] {
    match (ore, variant) {
        (OreKind::Iron, 0) => include_bytes!("assets/ores/iron-0.pxs"),
        (OreKind::Iron, 1) => include_bytes!("assets/ores/iron-1.pxs"),
        (OreKind::Iron, _) => include_bytes!("assets/ores/iron-2.pxs"),
        (OreKind::Coal, 0) => include_bytes!("assets/ores/coal-0.pxs"),
        (OreKind::Coal, 1) => include_bytes!("assets/ores/coal-1.pxs"),
        (OreKind::Coal, _) => include_bytes!("assets/ores/coal-2.pxs"),
        (OreKind::Titanium, 0) => include_bytes!("assets/ores/titanium-0.pxs"),
        (OreKind::Titanium, 1) => include_bytes!("assets/ores/titanium-1.pxs"),
        (OreKind::Titanium, _) => include_bytes!("assets/ores/titanium-2.pxs"),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BlockKind, CONVEYOR_VARIANT_COUNT, block_asset, conveyor_asset, item_asset, ore_asset,
    };
    use crate::simulation::{item::Item, world::terrain::OreKind};
    use api::glam::uvec2;
    use formats::{AnimatedSpriteDocument, SpriteDocument, TilesetDocument};
    use strum::IntoEnumIterator;

    #[test]
    fn drill_sprite_contains_a_visibly_rotating_top_layer() {
        let drill = AnimatedSpriteDocument::decode(block_asset(BlockKind::Drill))
            .expect("drill animation should decode");
        assert_eq!(drill.frames.len(), 8);
        assert!(drill.frames.windows(2).all(|pair| pair[0] != pair[1]));
    }

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

        for item in Item::iter() {
            let sprite =
                SpriteDocument::decode(item_asset(item)).expect("item sprite should decode");
            assert_eq!(sprite.size, uvec2(16, 16));
        }
        for ore in OreKind::iter() {
            for variant in 0..3 {
                let sprite = SpriteDocument::decode(ore_asset(ore, variant))
                    .expect("ore sprite should decode");
                assert_eq!(sprite.size, uvec2(16, 16));
            }
        }

        let terrain = TilesetDocument::decode(include_bytes!("assets/terrain.pxt"))
            .expect("terrain tileset should decode");
        assert_eq!(terrain.tile_size, uvec2(16, 16));
        assert_eq!(terrain.tiles.len(), 3);

        for (bytes, size) in [
            (include_bytes!("assets/cursor/single.pxs").as_slice(), 16),
            (include_bytes!("assets/cursor/double.pxs").as_slice(), 32),
            (include_bytes!("assets/cursor/triple.pxs").as_slice(), 48),
            (include_bytes!("assets/cursor/quad.pxs").as_slice(), 64),
        ] {
            let sprite = SpriteDocument::decode(bytes).expect("cursor sprite should decode");
            assert_eq!(sprite.size, uvec2(size, size));
        }
    }
}
