#pragma description Tileset. TSET version 1. Extension .pxt.
#pragma endian little
#pragma array_limit 0x1000000

import std.sys;

struct PalettePixel {
    u8 index;
    std::assert(index <= 255, "Use a color index from 0 to 255.");
};

struct TileImage<auto Width, auto Height> {
    // store pixels in row-major order. zero means clear.
    PalettePixel pixels[Width * Height];
};

struct Tileset {
    char magic[4];
    u16 version;
    std::assert(magic == "TSET" && version == 1, "Use TSET version 1.");
    // enter the matching tilemap uuid here.
    u8 uuid[16];
    u16 tile_width;
    u16 tile_height;
    u16 tile_count;
    std::assert(tile_width >= 8 && tile_width <= 96 && tile_width % 8 == 0, "Use a valid tile width.");
    std::assert(tile_height >= 8 && tile_height <= 96 && tile_height % 8 == 0, "Use a valid tile height.");
    std::assert(tile_count >= 1, "Add at least one tile.");
    // tile ids start at one. array index zero has id one.
    TileImage<tile_width, tile_height> tiles[tile_count];
};

Tileset tileset @ 0;
