#version 140

in vec2 coords; // Between 0-1
out vec4 color;

uniform sampler2D u_previous;
uniform vec2 u_resolution; // Size of the screen in pixels
uniform float u_time;

#define PIXEL (1.0 / u_resolution)

// P is a point from 0-1
vec2 curl_noise(vec2 p) {
    vec2 dx = vec2(PIXEL.x, 0.0);
    vec2 dy = vec2(0.0, PIXEL.y);

    float p_x0 = perlin(p - dx);
    float p_x1 = perlin(p + dx);

    // How the field changes with respect to X
    float dfield_dx = perlin(p + dx) - perlin(p - dx);

    // How the field changes with respect to Y
    float dfield_dy = perlin(p + dy) - perlin(p - dy);

    vec2 vel = vec2(dfield_dy, -dfield_dx);

    return (normalize(vel));
}

void main() {
    // float noise = perlin(coords * 100.0);
    vec2 curl = curl_noise(coords * 10.0);
    color = vec4(curl.x, 0.0, curl.y, 1.0);
}
