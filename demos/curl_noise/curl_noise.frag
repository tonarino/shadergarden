#version 140

// Reference Paper:
// https://www.cs.ubc.ca/~rbridson/docs/bridson-siggraph2007-curlnoise.pdf

in vec2 coords; // Between 0-1
out vec4 color;

uniform sampler2D u_previous;
uniform vec2 u_resolution; // Size of the screen in pixels
uniform float u_time;

#define PIXEL (1.0 / u_resolution)

vec3 permute(vec3 x) {
    return mod(((x * 34.0) + 1.0) * x, 289.0);
}

vec3 taylorInvSqrt(vec3 r) {
    return 1.79284291400159 - 0.85373472095314 * r;
}

// Given 2D coordinates, returns a float noise value
// in the [-1.0, 1.0] range.
float simplex_noise(vec2 p) {
    const vec2 C = vec2(0.211324865405187134, 0.366025403784438597);

    // First corner
    vec2 i = floor(p + dot(p, C.yy));
    vec2 x0 = p - i + dot(i, C.xx);

    // Other corners
    vec2 i1;
    i1.x = step(x0.y, x0.x);
    i1.y = 1.0 - i1.x;

    vec4 x12 = x0.xyxy + vec4(C.xx, C.xx * 2.0 - 1.0);
    x12.xy -= i1;

    // Permutations
    i = mod(i, 289.0); // Avoid truncation in polynomial evaluation.
    vec3 permuted = permute(permute(i.y + vec3(0.0, i1.y, 1.0)) + i.x + vec3(0.0, i1.x, 1.0));

    // Circularly symmetric blending kernel.
    vec3 m = max(0.5 - vec3(dot(x0, x0), dot(x12.xy, x12.xy), dot(x12.zw, x12.zw)), 0.0);

    m = m * m;
    m = m * m;

    // Gradients from 41 points on a line, mapped onto a diamond.
    vec3 x = fract(permuted * (1.0 / 41.0)) * 2.0 - 1.0;
    vec3 gy = abs(x) - 0.5;
    vec3 ox = floor(x + 0.5); // Could use round here?
    vec3 gx = x - ox;

    // Normalize gradients implicitly by scaling m.
    m *= taylorInvSqrt(gx * gx + gy * gy);

    // Compute final noise value at p.
    vec3 g;
    g.x = gx.x * x0.x + gy.x * x0.y;
    g.yz = gx.yz * x12.xz + gy.yz * x12.yw;

    // Scale output to span range [-1, 1].
    // (Scaling factor determined by experiments)
    return 130.0 * dot(m, g);
}

// P is a point from 0-1
// Output is in the range X: [-1.0, 1.0]
//                        Y: [-1.0, 1.0]
vec2 curl_noise(vec2 p) {
    vec2 dx = vec2(PIXEL.x, 0.0);
    vec2 dy = vec2(0.0, PIXEL.y);

    // How the field changes with respect to X
    float dfield_dx = simplex_noise(p + dx) - simplex_noise(p - dx);

    // How the field changes with respect to Y
    float dfield_dy = simplex_noise(p + dy) - simplex_noise(p - dy);

    vec2 vel = vec2(dfield_dy, -dfield_dx);

    return normalize(vel);
}

void main() {
    // float noise = perlin(coords * 100.0);
    vec2 curl = curl_noise(coords * 3.0) + curl_noise(coords * 10.0) + curl_noise(coords * 100.0);

    curl = (curl + 1.0) / 2.0;

    color = vec4(curl.x, curl.y, 0.5, 1.0);
}
