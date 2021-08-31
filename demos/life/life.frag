#version 140

in vec2 coords;
out vec4 color;

uniform sampler2D u_previous;
uniform vec2 u_resolution;
uniform float u_time;

#define PIXEL (1.0 / u_resolution)

int is_alive(vec2 st) {
    st = mod(st, 1.0);
    return int(texture(u_previous, st, 0.).r);
}

int count_neighbors(vec2 st) {
    int alive = 0 - is_alive(st);

    for (int i = -1; i <= 1; i++) {
        for (int j = -1; j <= 1; j++) {
            vec2 stn = vec2(st.x + PIXEL.x * i, st.y + PIXEL.y * j);
            alive += is_alive(stn);
        }
    }

    return alive;
}

int step_gol(vec2 st) {
    int neighbors = count_neighbors(st);

    if ((neighbors == 3) || (is_alive(st) == 1 && neighbors == 2)) {
        return 1;
    }

    return 0;
}

float random (vec2 st) {
    return fract(sin(dot(st.xy,
        vec2(12.9898,78.233)))*43758.5453123);
}

void main() {
    if (u_time < 1.0) {
        color = vec4(vec3(0.), 1.);
        if (coords.x < 0.5 && coords.y < 0.5) {
            color = vec4(1.);
        }
        return;
    }

    color = vec4(vec3(step_gol(coords)), 1.);
}
