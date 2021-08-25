#version 140

in vec2 coords;
out vec4 color;

uniform sampler2D u_texture_0;
uniform float u_time;
uniform vec2 u_resolution;

#define a_time 0.1 * u_time
#define PIXEL (1.0 / u_resolution)

void main() {
    color = vec4(vec3(0.), 1.);

    vec2 c = mod(1.0 * coords, 1.0);

    float l = length(c - 0.5);
    color = vec4(vec3(1.0 - smoothstep(0.1, 0.1 + PIXEL.x, l)), 1.);
    // color += vec4(vec3(smoothstep(0.4, 0.4 + PIXEL.x, l)), 1.);

    // float l2 = length(c - (vec2(sin(a_time), cos(a_time)) * 0.1 + 0.5));
    // color += vec4(vec3(1.0 - smoothstep(0.05, 0.05 + PIXEL.x, l2)), 1.);
    //
    float l3 = length(c - (vec2(sin(a_time+3.14), cos(a_time+3.14)) * 0.3 + 0.5));
    color += vec4(vec3(1.0 - smoothstep(0.05, 0.05 + PIXEL.x, l3)), 1.);

}
