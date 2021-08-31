#version 140

in vec2 coords;
out vec4 color;

uniform sampler2D u_texture;
uniform sampler2D u_previous;
uniform vec2 u_resolution;
uniform float u_time;

#define PIXEL (1.0 / u_resolution)

void main() {
    vec3 c = texture(u_texture, coords, 0.).rgb;
    vec3 p = texture(u_previous, coords, 0.).rgb;
    p *= vec3(0.98, 0.97, 0.99);
    color = vec4(max(c, p), 1.);
}
