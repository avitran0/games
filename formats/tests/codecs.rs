use formats::{
    AnimatedSpriteDocument, FontDocument, SpriteDocument, TilemapDocument, TilesetDocument,
};

#[test]
fn decodes_runtime_assets_and_linked_tilemap() {
    let sprite =
        SpriteDocument::decode(include_bytes!("../../snake/src/assets/head_16.pxs")).unwrap();
    let tileset =
        TilesetDocument::decode(include_bytes!("../../snake/src/assets/tileset_16.pxt")).unwrap();
    let map = TilemapDocument::decode(include_bytes!("../../snake/src/assets/map_16.pxm")).unwrap();
    let font = FontDocument::decode(include_bytes!("../../api/assets/font.pxf")).unwrap();
    let animation = AnimatedSpriteDocument::decode(&animation_bytes()).unwrap();

    assert_eq!(sprite.size.x, 16);
    assert_eq!(map.tileset_id, tileset.id);
    assert!(!font.glyphs.is_empty());
    assert_eq!(animation.frames.len(), 1);
}

#[cfg(feature = "edit")]
#[test]
fn edited_documents_round_trip() {
    use glam::uvec2;

    let mut sprite = SpriteDocument::new(uvec2(8, 8)).unwrap();
    sprite.pixels.pixels_mut()[0] = 42;
    assert!(SpriteDocument::decode(&sprite.encode().unwrap()).unwrap() == sprite);

    let animation = AnimatedSpriteDocument::new(uvec2(8, 8)).unwrap();
    assert!(AnimatedSpriteDocument::decode(&animation.encode().unwrap()).unwrap() == animation);

    let font = FontDocument::new(11).unwrap();
    assert_eq!(FontDocument::decode(&font.encode().unwrap()).unwrap(), font);

    let tileset = TilesetDocument::new(uvec2(8, 8)).unwrap();
    assert!(TilesetDocument::decode(&tileset.encode().unwrap()).unwrap() == tileset);

    let mut map = TilemapDocument::new(uvec2(2, 2), tileset.id).unwrap();
    map.cells[0] = formats::Tile {
        id: 1,
        flip_x: true,
        flip_y: true,
        flip_diagonal: true,
    };
    assert!(TilemapDocument::decode(&map.encode().unwrap()).unwrap() == map);
}

#[cfg(feature = "edit")]
#[test]
fn decoders_reject_invalid_fields_and_truncated_files() {
    use glam::uvec2;

    let mut sprite = SpriteDocument::new(uvec2(8, 8)).unwrap().encode().unwrap();
    let mut animation = AnimatedSpriteDocument::new(uvec2(8, 8))
        .unwrap()
        .encode()
        .unwrap();
    let font = FontDocument::new(8).unwrap().encode().unwrap();
    let tileset = TilesetDocument::new(uvec2(8, 8)).unwrap().encode().unwrap();
    let map = TilemapDocument::new(uvec2(1, 1), TilesetDocument::new(uvec2(8, 8)).unwrap().id)
        .unwrap()
        .encode()
        .unwrap();

    for end in 0..sprite.len() {
        assert!(SpriteDocument::decode(&sprite[..end]).is_err());
    }
    for end in 0..animation.len() {
        assert!(AnimatedSpriteDocument::decode(&animation[..end]).is_err());
    }
    for end in 0..font.len() {
        assert!(FontDocument::decode(&font[..end]).is_err());
    }
    for end in 0..tileset.len() {
        assert!(TilesetDocument::decode(&tileset[..end]).is_err());
    }
    for end in 0..map.len() {
        assert!(TilemapDocument::decode(&map[..end]).is_err());
    }

    sprite[4] = 2;
    animation[89] = 1;
    let mut font = font;
    font[10..14].copy_from_slice(&0xD800_u32.to_le_bytes());
    let mut tileset = tileset;
    tileset[26..28].copy_from_slice(&0_u16.to_le_bytes());
    let mut map = map;
    map[28] = 0x80;
    assert!(SpriteDocument::decode(&sprite).is_err());
    assert!(AnimatedSpriteDocument::decode(&animation).is_err());
    assert!(FontDocument::decode(&font).is_err());
    assert!(TilesetDocument::decode(&tileset).is_err());
    assert!(TilemapDocument::decode(&map).is_err());
}

#[cfg(feature = "edit")]
#[test]
fn encoders_reject_invalid_documents() {
    use formats::{GlyphDocument, Tile};
    use glam::uvec2;

    let mut animation = AnimatedSpriteDocument::new(uvec2(8, 8)).unwrap();
    animation.tags[0].end = 1;
    assert!(animation.encode().is_err());

    let mut font = FontDocument::new(8).unwrap();
    font.glyphs.push(font.glyphs[0].clone());
    assert!(font.encode().is_err());
    let invalid = GlyphDocument {
        codepoint: 'A',
        width: 0,
        advance: 1,
        bitmap: Vec::new(),
    };
    font.glyphs = vec![invalid];
    assert!(font.encode().is_err());

    let mut tileset = TilesetDocument::new(uvec2(8, 8)).unwrap();
    tileset.tiles.clear();
    assert!(tileset.encode().is_err());

    let tileset = TilesetDocument::new(uvec2(8, 8)).unwrap();
    let mut map = TilemapDocument::new(uvec2(1, 1), tileset.id).unwrap();
    map.cells[0] = Tile {
        id: u16::MAX as u32 + 1,
        ..Tile::default()
    };
    assert!(map.encode().is_err());
}

fn animation_bytes() -> Vec<u8> {
    [
        b"ANIM".as_slice(),
        &[1, 0, 8, 0, 8, 0, 1, 0, 0, 0],
        &[0; 64],
    ]
    .concat()
}
