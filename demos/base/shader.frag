#version 140

in  vec2 coords;
out vec4 color;

uniform float u_time;

void main() {
    color = vec4(coords.xy, pow(sin(u_time * 3.14), 2.), 1.);
}
