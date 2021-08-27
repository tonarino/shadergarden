// METADATA ----------

uniform vec2 u_resolution;
uniform float u_time;

// CONSTANTS ----------

#define START 0.0
#define END 1000.0
#define STEPS 100
#define EPSILON 0.001
#define FOV 90.0
#define PIXEL (1.0 / u_resolution)

// SIGNED DISTANCE FUNCTIONS ----------

float box(vec3 point, vec3 size) {
    vec3 d = abs(point) - (size / 2.0);
    float insideDistance = min(max(d.x, max(d.y, d.z)), 0.0);
    float outsideDistance = length(max(d, 0.0));
    return insideDistance + outsideDistance;
}

float sphere(vec3 point, float radius) {
    return length(point) - radius;
}

float scene (vec3 point) {
    point = mod(point + 4.0, 8.0) - 4.0;
    float cube = box(point, vec3(2.1));
    float sphere = sphere(point, sqrt(2.0));
    return max(cube, 0.);
}

// RAY MARCHER ----------

// depth, minimum distance, number of steps
vec3 march(vec3 camera, vec3 ray) {
    float depth = START;
    float minDistance = END;
    float steps = 0.0;

    for (steps = 0.0; steps < float(STEPS); steps++) {
        float dist = scene(camera + depth * ray);
        minDistance = min(minDistance, dist);
        depth += dist;

        if (dist < EPSILON) {
            return vec3(depth, minDistance, steps - 1.0 + (dist / EPSILON));
        }

        if (depth > END) {
            return vec3(END, minDistance, steps);
        }
    }

    return vec3(depth, minDistance, STEPS);
}

// SHADING AND COLORS ----------

vec3 normal(vec3 p) {
    return normalize(vec3(
        scene(vec3(p.x + EPSILON, p.y, p.z)) - scene(vec3(p.x - EPSILON, p.y, p.z)),
        scene(vec3(p.x, p.y + EPSILON, p.z)) - scene(vec3(p.x, p.y - EPSILON, p.z)),
        scene(vec3(p.x, p.y, p.z + EPSILON)) - scene(vec3(p.x, p.y, p.z - EPSILON))
    ));
}

// CAMERA ----------

vec3 makeRay(float fov, float ratio, vec2 st) {
    vec2 xy = st - vec2(ratio, 1.0) * 0.5;
    float z = 1.0 / tan(radians(fov) / 2.0);
    return normalize(vec3(xy, -z));
}

mat3 look(vec3 camera, vec3 target, vec3 up) {
    // Based on gluLookAt man page
    vec3 f = normalize(target - camera);
    vec3 s = normalize(cross(f, up));
    vec3 u = cross(s, f);
    return mat3(s, u, -f);
}

float random (vec2 st) {
    return fract(sin(dot(st.xy, vec2(12.9898, 78.233))) * 43758.5453123);
}

// MAIN LOOP ----------

void main() {
    vec2 st = gl_FragCoord.xy/u_resolution.xy;
    st.x *= u_resolution.x/u_resolution.y;

    vec2 jitter = vec2(
        random(st + u_time),
        random(st + u_time * 2.0));
    st += jitter * PIXEL;

    vec3 ray = makeRay(FOV, u_resolution.x/u_resolution.y, st);
    vec3 point = vec3(1.0, 0.2, 1.5) * 5.0;
    mat3 view = look(point, vec3(0.), vec3(0.0, 1.0, 0.0));
    vec3 dir = view * ray;
    vec3 color = vec3(1.0);

    for (int i = 0; i < 2; i++) {
        vec3 marchResults = march(point, dir);
        float depth = marchResults.x;
        float minDistance = marchResults.y;
        float steps = marchResults.z;

        point = point + (depth - EPSILON * 2.0) * dir;
        vec3 normals = normal(point);
        dir = reflect(dir, normals);
        float occ = 1.0 - ((steps + (random(st) * 2.0 - 1.0)) / float(STEPS));
        color *= occ * (dir*.5+.5);
    }

    // float keyLight = float(smoothstep((normals.x + normals.y) / 2.0 + 0.5, 0.4, 0.6) * 0.5 + 0.5);
    // color = vec3(pow(occ, 2.0) * keyLight);

    gl_FragColor = vec4(color, 1.0);

    // gl_FragColor = vec4(color * 1.0 - (steps / float(STEPS)), 1.0);
}
