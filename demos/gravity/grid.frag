#version 140

in vec2 coords;
out vec4 color;

// use this fragment shader as a starting point for new ones
uniform sampler2D u_previous;
uniform sampler2D u_texture_0;
uniform vec2 u_resolution;
uniform float u_time;

#define PIXEL (1.0 / u_resolution)

vec4 lookup(vec2 p, float dx, float dy) {
    vec2 uv = p + vec2(dx, dy) * PIXEL;
    vec4 c = texture(u_previous, uv, 0.).rgba;
    c = c*2.-1.;

    float mass = texture(u_texture_0, uv, 0.).r * 0.99;

    // if (texture(u_texture_0, uv, 0.).r > 0.5) {
    return (1.0 - mass) * c + (mass * vec4(
        vec2(dx, dy),
        -vec2(dx, dy)
    ));
    // }

    // return c;
}

float sim(vec2 a, vec2 b) {
    return dot(normalize(a), normalize(b));
}

vec4 update(vec2 p) {
    vec2 force = vec2(0.0);
    vec2 force_back = vec2(0.0);
    float tot_s = 0.0;
    float tot_sb = 0.0;

    vec4 c = lookup(p, 0, 0);
    vec2 baseline = vec2(c.r, c.g); // + vec2(c.b, c.a);

    for (float i = -1; i <= 1; i++) {
        for (float j = -1; j <= 1; j++) {
            vec4 n = lookup(p, i, j);

            {
                float s = sim(n.xy, baseline);
                vec2 new = vec2(i, j) + n.xy;
                float strength = sqrt(length(new));
                if (s >= 0.0) {
                    force += n.xy * strength * s;
                    tot_s += strength * s;
                } else {
                    force_back += n.xy * strength * -s;
                    tot_sb += strength * -s;
                }
            }

            {
                float s = sim(n.ba, baseline);
                vec2 new = vec2(i, j) + n.ba;
                float strength = sqrt(length(new));
                if (s >= 0.0) {
                    force += n.ba * strength * s;
                    tot_s += strength * s;
                } else {
                    force_back += n.ba * strength * -s;
                    tot_sb += strength * -s;
                }
            }
        }
    }

    return vec4(
        force      / (tot_s  + 0.004),
        force_back / (tot_sb + 0.004));
}

float rand(vec2 co){
    return fract(sin(dot(co, vec2(12.9898, 78.233))) * 43758.5453);
}

void main() {
    if (u_time < 1.0) {
        color = vec4(
            rand(coords),
            rand(coords + 1.0),
            rand(coords + 2.0),
            rand(coords + 3.0)
            );
        // color = vec4(0.5);
        return;
    }

    vec4 new = update(coords);

    color = new*.5+.5;
}
