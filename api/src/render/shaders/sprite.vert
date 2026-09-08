#version 300 es
precision highp float;

uniform vec2 position;
uniform vec2 size;
uniform float rotation;
uniform vec2 rotation_anchor;
uniform vec2 resolution;
uniform vec2 flip;
uniform int flip_diagonal;

layout(location = 0) in vec2 vertex;

out vec2 uv;

void main() {
    vec2 local = vertex * size;

    mat2 rotation_matrix = mat2(
        cos(rotation), sin(rotation),
        -sin(rotation), cos(rotation)
    );

    vec2 pixel_position = position + rotation_anchor
        + rotation_matrix * (local - rotation_anchor);

    vec2 ndc = vec2(
        pixel_position.x / resolution.x * 2.0 - 1.0,
        1.0 - pixel_position.y / resolution.y * 2.0
    );

    gl_Position = vec4(ndc, 0.0, 1.0);
    vec2 texture_position = flip_diagonal != 0 ? vertex.yx : vertex;
    uv = vec2(
        flip.x < 0.0 ? texture_position.x : 1.0 - texture_position.x,
        flip.y < 0.0 ? texture_position.y : 1.0 - texture_position.y
    );
}
