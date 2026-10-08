#pragma description Variable-width bitmap font. FONT version 1. Extension .pxf.
#pragma endian little
#pragma array_limit 0x1000000

import std.sys;

struct Glyph<auto Height> {
    u32 codepoint;
    std::assert(codepoint <= 0x10FFFF && (codepoint < 0xD800 || codepoint > 0xDFFF), "Use a Unicode scalar value.");
    u16 advance;
    u16 width;
    std::assert(width > 0, "Use a nonzero glyph width.");
    // store pixels in row-major order. zero means clear.
    u8 bitmap[width * Height];
};

struct Font {
    char magic[4];
    u16 version;
    std::assert(magic == "FONT" && version == 1, "Use FONT version 1.");
    u16 height;
    u16 glyph_count;
    std::assert(height >= 1 && height <= 64, "Use a font height from 1 to 64.");
    std::assert(glyph_count >= 1, "Add at least one glyph.");
    Glyph<height> glyphs[glyph_count];
};

Font font @ 0;
