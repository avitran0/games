#version 300 es
precision highp float;
precision highp usampler2DArray;

uniform usampler2DArray sprite_texture;
uniform sampler2D palette_texture;
uniform int color_index;
uniform int sprite_frame;
uniform vec2 size;

in vec2 uv;

layout(location = 0) out vec4 color;

void main() {
    ivec2 pixel = ivec2(uv * size);

    uint glyph_mask = texelFetch(
        sprite_texture,
        ivec3(pixel, sprite_frame),
        0
    ).r;

    if (glyph_mask == 0u || color_index == 0) {
        discard;
    }

    vec4 pal_color = texelFetch(
        palette_texture,
        ivec2(color_index - 1, 0),
        0
    );

    color = pal_color;
}
