#pragma description Sprite sheet. SPRS version 1. Extension .pxh.
#pragma endian little
#pragma array_limit 0x1000000

import std.sys;

struct PalettePixel {
    u8 index;
};

struct NamedSprite {
    u16 name_length;
    char name[name_length];
    u16 width;
    u16 height;
    std::assert(width >= 8 && width <= 96 && width % 8 == 0, "Use a valid sprite width.");
    std::assert(height >= 8 && height <= 96 && height % 8 == 0, "Use a valid sprite height.");
    PalettePixel pixels[width * height];
};

struct SpriteSheet {
    char magic[4];
    u16 version;
    std::assert(magic == "SPRS" && version == 1, "Use SPRS version 1.");
    u16 sprite_count;
    std::assert(sprite_count > 0, "A sprite sheet must contain at least one sprite.");
    NamedSprite sprites[sprite_count];
};

SpriteSheet sheet @ 0;
