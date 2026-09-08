#version 300 es
precision highp float;

uniform vec2 position;
uniform vec2 size;
uniform vec2 resolution;

layout(location = 0) in vec2 vertex;

out vec2 uv;

void main() {
    vec2 local = vertex * size;
    vec2 pixel_position = position + local;

    vec2 ndc = vec2(
        pixel_position.x / resolution.x * 2.0 - 1.0,
        1.0 - pixel_position.y / resolution.y * 2.0
    );

    gl_Position = vec4(ndc, 0.0, 1.0);
    uv = vertex;
}
