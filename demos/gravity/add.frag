#version 140

in vec2 coords;
out vec4 color;

uniform sampler2D u_texture_0;
uniform sampler2D u_texture_1;

void main() {
    color = texture(u_texture_0, coords);
    color += texture(u_texture_1, coords).r * 0.3;
}
