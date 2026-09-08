#version 300 es
precision highp float;
precision highp usampler2DArray;

uniform usampler2DArray sprite_texture;
uniform sampler2D palette_texture;
uniform int sprite_frame;
uniform vec2 size;

in vec2 uv;

layout(location = 0) out vec4 color;

void main() {
    ivec2 pixel = ivec2(uv * size);

    uint palette_index = texelFetch(
        sprite_texture,
        ivec3(pixel, sprite_frame),
        0
    ).r;
    if (palette_index == 0u) {
        discard;
    }

    vec4 pal_color = texelFetch(
        palette_texture,
        ivec2(int(palette_index - 1u), 0),
        0
    );

    color = pal_color;
}
