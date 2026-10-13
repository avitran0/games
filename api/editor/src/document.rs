use formats::{
    AnimatedSpriteDocument, FontDocument, SpriteDocument, TilemapDocument, TilesetDocument,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssetKind {
    Sprite,
    AnimatedSprite,
    Font,
    Tileset,
    Tilemap,
}

struct AssetInfo {
    title: &'static str,
    extension: &'static str,
    filename: &'static str,
    magic: [u8; 4],
}

impl AssetKind {
    pub const ALL: [Self; 5] = [
        Self::Sprite,
        Self::AnimatedSprite,
        Self::Font,
        Self::Tileset,
        Self::Tilemap,
    ];

    const fn info(self) -> AssetInfo {
        match self {
            Self::Sprite => AssetInfo {
                title: "Static sprite",
                extension: "pxs",
                filename: "sprite.pxs",
                magic: *b"SPRT",
            },
            Self::AnimatedSprite => AssetInfo {
                title: "Animated sprite",
                extension: "pxa",
                filename: "animation.pxa",
                magic: *b"ANIM",
            },
            Self::Font => AssetInfo {
                title: "Bitmap font",
                extension: "pxf",
                filename: "font.pxf",
                magic: *b"FONT",
            },
            Self::Tileset => AssetInfo {
                title: "Tileset",
                extension: "pxt",
                filename: "tileset.pxt",
                magic: *b"TSET",
            },
            Self::Tilemap => AssetInfo {
                title: "Tilemap",
                extension: "pxm",
                filename: "map.pxm",
                magic: *b"TMAP",
            },
        }
    }

    pub fn title(self) -> &'static str {
        self.info().title
    }

    pub fn extension(self) -> &'static str {
        self.info().extension
    }

    pub fn default_filename(self) -> &'static str {
        self.info().filename
    }

    pub fn from_magic(magic: &[u8]) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|kind| kind.info().magic == magic)
    }
}

#[derive(Clone)]
pub enum AssetDocument {
    Sprite(SpriteDocument),
    AnimatedSprite(AnimatedSpriteDocument),
    Font(FontDocument),
    Tileset(TilesetDocument),
    Tilemap {
        map: TilemapDocument,
        tileset: TilesetDocument,
    },
}

impl AssetDocument {
    pub fn kind(&self) -> AssetKind {
        match self {
            Self::Sprite(_) => AssetKind::Sprite,
            Self::AnimatedSprite(_) => AssetKind::AnimatedSprite,
            Self::Font(_) => AssetKind::Font,
            Self::Tileset(_) => AssetKind::Tileset,
            Self::Tilemap { .. } => AssetKind::Tilemap,
        }
    }

    pub fn encode(&self) -> Result<Vec<u8>, String> {
        match self {
            Self::Sprite(document) => document.encode().map_err(|error| error.to_string()),
            Self::AnimatedSprite(document) => document.encode().map_err(|error| error.to_string()),
            Self::Font(document) => document.encode().map_err(|error| error.to_string()),
            Self::Tileset(document) => document.encode().map_err(|error| error.to_string()),
            Self::Tilemap { map, .. } => map.encode().map_err(|error| error.to_string()),
        }
    }
}
