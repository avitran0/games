#pragma description Tilemap referencing a separate tileset by UUID, TMAP version 1. File extension: .pxm.
#pragma endian little
#pragma array_limit 0x1000000

import std.sys;
import std.mem;

struct Cell {
    // Zero means empty. Other IDs refer to the separately stored tileset.
    u16 tile_id;
    // Bit zero flips horizontally, bit one vertically, bit two transposes diagonally.
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
    // Raw UUID bytes for the separately stored TSET file.
    u8 tileset_uuid[16];
    // Cells use row-major order. Each cell is three bytes with no padding.
    Cell cells[width * height];
};

Tilemap tilemap @ 0;
std::assert(sizeof(tilemap) == std::mem::size(), "The file has extra data at the end.");
