#pragma description Animated sprite. ANIM version 1. Extension .pxa.
#pragma endian little
#pragma array_limit 0x1000000

import std.sys;

struct PalettePixel {
    u8 index;
    std::assert(index <= 255, "Use a color index from 0 to 255.");
};

struct Image<auto Width, auto Height> {
    // store pixels in row-major order. zero means clear.
    PalettePixel pixels[Width * Height];
};

enum Direction : u8 {
    Forward = 0,
    Reverse = 1,
    PingPong = 2,
    PingPongReverse = 3
};

struct AnimationTag<auto FrameCount> {
    u16 name_length;
    std::assert(name_length > 0, "Give the tag a name.");
    // store the name as utf-8 bytes with no ending zero byte.
    char name[name_length];
    u16 first_frame;
    u16 last_frame;
    std::assert(first_frame <= last_frame && last_frame < FrameCount, "Use a valid frame range.");
    Direction direction;
    std::assert(u8(direction) <= 3, "Use a valid direction value.");
};

struct AnimatedSprite {
    char magic[4];
    u16 version;
    std::assert(magic == "ANIM" && version == 1, "Use ANIM version 1.");
    u16 width;
    u16 height;
    std::assert(width >= 8 && width <= 96 && width % 8 == 0, "Use a valid frame width.");
    std::assert(height >= 8 && height <= 96 && height % 8 == 0, "Use a valid frame height.");
    // both counts use u16 values.
    u16 frame_count;
    u16 tag_count;
    Image<width, height> frames[frame_count];
    AnimationTag<frame_count> tags[tag_count];
};

AnimatedSprite sprite @ 0;
