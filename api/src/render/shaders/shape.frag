#version 300 es
precision highp float;

uniform sampler2D palette_texture;
uniform int color_index;
uniform int border_color_index;
uniform float border_width;
uniform float radius;
uniform vec2 size;
in vec2 shape_uv;

layout(location = 0) out vec4 color;

void main() {
    vec2 half_size = size * 0.5;
    vec2 point = (shape_uv - 0.5) * size;
    float corner_radius = clamp(radius, 0.0, min(half_size.x, half_size.y));
    vec2 q = abs(point) - half_size + corner_radius;
    float distance = length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - corner_radius;
    if (distance > 0.0) {
        discard;
    }

    int selected_color = color_index;
    if (border_width > 0.0 && distance >= -border_width) {
        selected_color = border_color_index;
    }
    if (selected_color == 0) {
        discard;
    }
    color = texelFetch(palette_texture, ivec2(selected_color - 1, 0), 0);
}
