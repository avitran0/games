#pragma description Tilemap referencing a separate tileset by UUID, TMAP version 1. File extension: .pxm.
#pragma endian little
#pragma array_limit 0x1000000

import std.sys;

struct Cell {
    // zero means empty. other ids refer to the separate tileset.
    u16 tile_id;
    // bit 0 flips horizontally, bit 1 flips vertically, and bit 2 transposes diagonally.
    u8 flags;
    std::assert((flags & 0xF8) == 0, "The flip flags are not valid.");
};

struct Tilemap {
    char magic[4];
    u16 version;
    std::assert(magic == "TMAP" && version == 1, "The file must use TMAP version 1.");
    u16 width;
    u16 height;
    std::assert(width >= 1 && width <= 256 && height >= 1 && height <= 256, "The map width and height must be from 1 to 256 cells.");
    // store the uuid of the separate tileset here.
    u8 tileset_uuid[16];
    // store cells in row-major order. each cell uses three bytes.
    Cell cells[width * height];
};

Tilemap tilemap @ 0;
