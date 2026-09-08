#pragma description Tileset. TSET version 1. Extension .pxt.
#pragma endian little
#pragma array_limit 0x1000000

import std.sys;
import std.mem;

struct PalettePixel {
    u8 index;
    std::assert(index <= 64, "Use a color index from 0 to 64.");
};

struct TileImage<auto Width, auto Height> {
    // Pixels use row-major order. Zero is clear.
    PalettePixel pixels[Width * Height];
};

struct Tileset {
    char magic[4];
    u16 version;
    std::assert(magic == "TSET" && version == 1, "Use TSET version 1.");
    // Store the 16 UUID bytes from the matching tilemap here.
    u8 uuid[16];
    u16 tile_width;
    u16 tile_height;
    u16 tile_count;
    std::assert(tile_width >= 8 && tile_width <= 64 && tile_width % 8 == 0, "Use a valid tile width.");
    std::assert(tile_height >= 8 && tile_height <= 64 && tile_height % 8 == 0, "Use a valid tile height.");
    std::assert(tile_count >= 1, "Add at least one tile.");
    // Tile IDs start at one. Array index zero has ID one.
    TileImage<tile_width, tile_height> tiles[tile_count];
};

Tileset tileset @ 0;
std::assert(sizeof(tileset) == std::mem::size(), "The file has extra data.");
