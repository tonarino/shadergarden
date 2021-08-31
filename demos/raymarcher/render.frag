// METADATA ----------

uniform vec2 u_resolution;
uniform float u_time;

// CONSTANTS ----------

#define START 0.0
#define END 1000.0
#define STEPS 100
#define EPSILON 0.001
#define FOV 60.0
#define PIXEL (1.0 / u_resolution)

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

float random(vec2 st) {
    return fract(sin(dot(st.xy, vec2(12.9898, 78.233))) * 43758.5453123);
}

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

float scene(vec3 point) {
    float c1 = box(point - vec3(-10.0, 0.0, 0.0), vec3(10.0));
    float c2 = box(point - vec3(0.0, 10.0, 0.0), vec3(10.0));
    float c3 = box(point - vec3(0.0,-10.0, 0.0), vec3(10.0));
    float c4 = box(point - vec3(0.0, 0.0, 10.0), vec3(10.0));
    float c5 = box(point - vec3(0.0, 0.0,-10.0), vec3(10.0));
    float box = min(c5, min(min(c1, c2), min(c3, c4)));

    mat3 t = look(vec3(1.0, 0., 0.5), vec3(0., 0., 0.), vec3(0.0, 1.0, 0.0));
    float box1 = box((point + vec3(2., 2., 2.)) * t, vec3(3.0, 6.0, 3.0));
    mat3 t2 = look(vec3(0.5, 0., 1.0), vec3(0., 0., 0.), vec3(0.0, 1.0, 0.0));
    float box2 = box((point + vec3(-1., 3.5, -1.3)) * t2, vec3(3.0, 3.0, 5.0));
    float box_u = min(box1, box2);

    float circle = sphere(point + vec3(-1., 0.0, -1.3), 2.0);
    return min(min(box, circle), box_u);
}

vec4 emission(vec3 point) {
    if (length(point) > 10.0) {
        float intense = dot(normalize(point), vec3(0., 1., 0.))*.5+.5;
        return vec4(vec3(0.05, 0.05, 0.15), 0.) + intense * 0.2;
    }

    if (sphere(point + vec3(-1., 0.0, -1.3), 2.0) < 0.01) {
        return vec4(vec3(0.), 1.);
    }

    if (point.z > 4.99) {
        return vec4(1., 0., 0., 1.);
    }
    if (point.z < -4.99) {
        return vec4(0., 1., 0., 1.);
    }
    if (point.y > 4.99) {
        return vec4(vec3(0.9), 0.);
    }

    return vec4(0.);
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

vec3 sample_sphere(in vec2 seed) {
    vec3 point = vec3(0.0);

    // sample point in unit cube, check if in unit sphere
    for (int i = 0; i < 3; i++) {
        point = vec3(
            random(seed),
            random(seed * 2.0),
            random(seed * 3.0)) * 2. - 1.;
        if (length(point) <= 1.0) {
            break;
        }
    }

    return point;
}

vec3 sample_sphere_surface(in vec2 seed) {
    return normalize(sample_sphere(seed));
}

vec3 diffuse_dir(in vec3 normal, in vec2 seed) {
    return normalize(normal + sample_sphere_surface(seed));
}

vec3 reflect_dir(in vec3 dir, in vec3 normal, in float roughness, in vec2 seed) {
    return reflect(dir, normal + sample_sphere(seed) * roughness);
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
    vec3 point = vec3(17.9, -1.2, 0.0);
    mat3 view = look(point, vec3(0., -1.2, 0.), vec3(0.0, 1.0, 0.0));
    vec3 dir = view * ray;
    vec3 color = vec3(0.0);

    for (int i = 0; i < 5; i++) {
        vec3 marchResults = march(point, dir);
        float depth = marchResults.x;
        float minDistance = marchResults.y;
        float steps = marchResults.z;

        point = point + (depth - EPSILON * 2.0) * dir;
        vec4 e = emission(point);
        vec3 normals = normal(point);
        if (random(st + u_time * 3.0) < e.a) {
            dir = reflect_dir(dir, normals, 0.05, st + u_time * 4.0);
        } else {
            dir = diffuse_dir(normals, st + u_time * 5.0);
        }
        float occ = 1.0 - ((steps + (random(st) * 2.0 - 1.0)) / float(STEPS));
        color += e.rgb;
        color *= occ;
    }

    color /= 3.0;
    color -= 0.2;
    color *= 2.0;
    color += 0.2;

    // float keyLight = float(smoothstep((normals.x + normals.y) / 2.0 + 0.5, 0.4, 0.6) * 0.5 + 0.5);
    // color = vec3(pow(occ, 2.0) * keyLight);

    gl_FragColor = vec4(color, 1.0);

    // gl_FragColor = vec4(color * 1.0 - (steps / float(STEPS)), 1.0);
}
