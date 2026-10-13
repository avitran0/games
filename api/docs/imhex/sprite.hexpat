#pragma description Static sprite. SPRT version 1. Extension .pxs.
#pragma endian little
#pragma array_limit 0x1000000

import std.sys;

struct PalettePixel {
    u8 index;
    std::assert(index <= 255, "Use a color index from 0 to 255.");
};

struct Sprite {
    char magic[4];
    u16 version;
    std::assert(magic == "SPRT" && version == 1, "Use SPRT version 1.");
    u16 width;
    u16 height;
    std::assert(width >= 8 && width <= 96 && width % 8 == 0, "Use a valid sprite width.");
    std::assert(height >= 8 && height <= 96 && height % 8 == 0, "Use a valid sprite height.");
    // store pixels in row-major order. zero means clear.
    PalettePixel pixels[width * height];
};

Sprite sprite @ 0;
