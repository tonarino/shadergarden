#version 140

in vec2 coords;
out vec4 color;

uniform sampler2D u_texture_0;
uniform vec2 u_resolution;
uniform float u_time;

float lookup(vec2 p, float dx, float dy) {
    vec2 uv = (p.xy + vec2(dx, dy)) / u_resolution.xy;
    vec4 c = texture(u_texture_0, uv.xy);
    return 0.2126*c.r + 0.7152*c.g + 0.0722*c.b;
}

void main() {
    vec2 p = coords * u_resolution;

    float gx = 0.0;
    gx += -1.0 * lookup(p, -1.0, -1.0);
    gx += -2.0 * lookup(p, -1.0,  0.0);
    gx += -1.0 * lookup(p, -1.0,  1.0);
    gx +=  1.0 * lookup(p,  1.0, -1.0);
    gx +=  2.0 * lookup(p,  1.0,  0.0);
    gx +=  1.0 * lookup(p,  1.0,  1.0);
    gx = gx / 8.0;

    float gy = 0.0;
    gy += -1.0 * lookup(p, -1.0, -1.0);
    gy += -2.0 * lookup(p,  0.0, -1.0);
    gy += -1.0 * lookup(p,  1.0, -1.0);
    gy +=  1.0 * lookup(p, -1.0,  1.0);
    gy +=  2.0 * lookup(p,  0.0,  1.0);
    gy +=  1.0 * lookup(p,  1.0,  1.0);
    gy = gy / 8.0;

    vec4 col = vec4(vec3(abs((gx*gx + gy*gy))), 1.0);
    color = col;
}
