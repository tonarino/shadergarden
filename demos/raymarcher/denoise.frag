#version 140

// Snaps a disparity map to some edges,
// Using an automata and some fancy curves.

uniform sampler2D u_texture_0; // edge map
uniform sampler2D u_texture_1; // image
uniform vec2 u_resolution;
uniform float u_time;

in vec2 coords;
out vec4 color;

#define PIXEL (1.0 / u_resolution)

float phi(float n) {
    return (1. - sqrt(4 * n + 1)) / 2.0;
}

float warp_space(float x, float w) {
    float pos = phi(w);
    return -(w/(x-pos)+pos+1.);
}

float left_warped_curve(float x, float w) {
    return .5-.5*cos(3.14*warp_space(x, w));
}

vec3 average_out(vec2 uv) {
    vec3 total = vec3(0.0);
    float out_of = 0.0;

    for (float i = -1.0; i <= 1.0; i++) {
        for (float j = -1.0; j <= 1.0; j++) {
            vec2 pos = vec2(uv.x + i * PIXEL.x, uv.y + j * PIXEL.y);
            float edge = texture(u_texture_0, pos, 0.).r;
            float wall = 1.0 - left_warped_curve(edge, 0.0001);
            vec3 color = texture(u_texture_1, pos, 0.).rgb;
            total  += wall * color;
            out_of += wall;
        }
    }

    if (out_of == 0.0) {
        return texture(u_texture_1, coords, 0.).rgb;
    }

    vec3 average = total / out_of;
    return average;
}

void main() {
    color = vec4(average_out(coords), 1.0);
}
