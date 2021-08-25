#version 140

in vec2 coords;
out vec4 color;

uniform sampler2D u_previous;
uniform sampler2D u_texture_0;
uniform vec2 u_resolution;
uniform float u_time;

#define PIXEL (1.0 / u_resolution)

vec2 lookup(vec2 p, float dx, float dy) {
    vec2 uv = p + vec2(dx, dy) * PIXEL;
    vec4 c = texture(u_texture_0, uv, 0.).rgba;
    c = c*2.-1.;
    return (c.xy + c.ba);
}


vec3 move(vec2 p) {
    float color = 0.0;
    vec3 ret = vec3(0.0, 0.0, 0.0);

    for (float i = -1; i <= 1; i++) {
        for (float j = -1; j <= 1; j++) {
            vec2 uv = p + vec2(i, j) * PIXEL;
            vec3 particle = texture(u_previous, uv, 0.).rgb;
            if (particle.r > 0.5) {
                vec2 velocity = particle.gb*2.-1.;
                vec2 force = lookup(coords, i, j);
                vec2 new_v = velocity + force * 0.01;
                vec2 new_pos = (vec2(i, j) + new_v.xy) + sin(u_time * 37289.321789 + p) * 0.5;

                if (abs(new_pos.x) < 0.5 && abs(new_pos.y) < 0.5) {
                    ret += vec3(particle.r, new_v);
                }
            }
        }
    }

    return vec3(ret.r, ret.gb*.5+.5);
}

void main() {
    // if (sin(u_time * 10.0) < 0.0) {
        // color = vec4(0.);

        vec2 tile = mod(coords * u_resolution, 128.0);
        if (tile.x < 1.0 || tile.y < 1.0) {
            color = vec4(1.0, 0.5, 0.5, 1.0);
            return;
        }
    // }

    color = vec4(move(coords), 1.0);
}
