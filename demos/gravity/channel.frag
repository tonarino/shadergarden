#version 140

in vec2 coords;
out vec4 color;

uniform sampler2D u_texture_0;

float sim(vec2 a, vec2 b) {
    return dot(normalize(a), normalize(b))*.5+.5;
}

void main() {
    vec4 p = texture(u_texture_0, coords).rgba;
    p = p*2.-1.;
    float a = sim(vec2(p.r, p.g), vec2(p.b, p.a));
    vec2 d = vec2(p.r, p.g) + vec2(p.b, p.a);
    // d = normalize(d);
    d = d * 0.5 + 0.5;
    color = vec4(vec3(a), 1.0) * 2.0;
    color *= vec4(d, 0.5, 1.0);
    color = sqrt(color);
}
