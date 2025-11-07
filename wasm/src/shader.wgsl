// Black Hole Geodesic Compute Shader (WGSL)

struct Camera {
    pos: vec3<f32>,
    _pad0: f32,
    right: vec3<f32>,
    _pad1: f32,
    up: vec3<f32>,
    _pad2: f32,
    forward: vec3<f32>,
    _pad3: f32,
    tan_half_fov: f32,
    aspect: f32,
    moving: u32,
    _pad4: u32,
}

struct Disk {
    inner_radius: f32,
    outer_radius: f32,
    _pad: f32,
    thickness: f32,
}

struct Planet {
    position: vec3<f32>,
    radius: f32,
}

@group(0) @binding(0) var output_texture: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(1) var<uniform> camera: Camera;
@group(0) @binding(2) var<uniform> disk: Disk;
@group(0) @binding(3) var<uniform> planet: Planet;
@group(0) @binding(4) var background_texture: texture_2d<f32>;

const WIDTH: u32 = 800u;
const HEIGHT: u32 = 600u;
const NSTEPS: u32 = 2000u;
const MAX_REVOLUTIONS: f32 = 2.0;
const PI: f32 = 3.14159265359;
const SAG_A_RS: f32 = 1.269e10;

fn crosses_equatorial_plane(old_pos: vec3<f32>, new_pos: vec3<f32>) -> bool {
    let crossed = (old_pos.y * new_pos.y) < 0.0;
    let r = length(vec2<f32>(new_pos.x, new_pos.z));
    return crossed && (r >= disk.inner_radius && r <= disk.outer_radius);
}

fn direction_to_uv(dir: vec3<f32>) -> vec2<f32> {
    let normalized = normalize(dir);
    let u = 0.5 + atan2(normalized.z, normalized.x) / (2.0 * PI);
    let v = 0.5 - asin(normalized.y) / PI;
    return vec2<f32>(u, v);
}

fn intersect_sphere(ray_origin: vec3<f32>, ray_dir: vec3<f32>, sphere_center: vec3<f32>, sphere_radius: f32) -> f32 {
    let oc = ray_origin - sphere_center;
    let a = dot(ray_dir, ray_dir);
    let b = 2.0 * dot(oc, ray_dir);
    let c = dot(oc, oc) - sphere_radius * sphere_radius;
    let discriminant = b * b - 4.0 * a * c;

    if (discriminant < 0.0) {
        return -1.0;
    }

    let t = (-b - sqrt(discriminant)) / (2.0 * a);
    if (t < 0.0) {
        return -1.0;
    }

    return t;
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let pix = vec2<u32>(global_id.xy);
    if (pix.x >= WIDTH || pix.y >= HEIGHT) {
        return;
    }

    // Initialize ray from camera
    let screen_u = (2.0 * (f32(pix.x) + 0.5) / f32(WIDTH) - 1.0) * camera.aspect * camera.tan_half_fov;
    let screen_v = (1.0 - 2.0 * (f32(pix.y) + 0.5) / f32(HEIGHT)) * camera.tan_half_fov;
    let ray_dir = normalize(screen_u * camera.right - screen_v * camera.up + camera.forward);

    var color = vec4<f32>(0.0, 0.0, 0.0, 1.0);

    // Normalize to geometric units where r_s = 2.0
    let unit_scale = SAG_A_RS / 2.0;

    // Leapfrog integration using u = 1/r (in geometric units)
    var pos = camera.pos / unit_scale;
    var u = 1.0 / length(pos);
    let u0 = u;

    let normal_vec = normalize(pos);
    let tangent_vec_unnorm = cross(cross(normal_vec, ray_dir), normal_vec);
    let tangent_len = length(tangent_vec_unnorm);

    // Safeguard against degenerate cases
    var tangent_vec = vec3<f32>(0.0);
    var du = 0.0;
    if (tangent_len > 1e-6) {
        tangent_vec = tangent_vec_unnorm / tangent_len;
        let denominator = dot(ray_dir, tangent_vec);
        if (abs(denominator) > 1e-6) {
            du = -dot(ray_dir, normal_vec) / denominator * u;
        }
    } else {
        // Fallback: use a perpendicular vector
        tangent_vec = normalize(cross(normal_vec, vec3<f32>(0.0, 1.0, 0.0)));
        if (length(cross(normal_vec, vec3<f32>(0.0, 1.0, 0.0))) < 1e-6) {
            tangent_vec = normalize(cross(normal_vec, vec3<f32>(1.0, 0.0, 0.0)));
        }
    }
    let du0 = du;

    var phi = 0.0;
    var old_pos = pos;

    var hit_black_hole = false;
    var hit_disk = false;
    var hit_planet = false;
    var planet_normal = vec3<f32>(0.0);

    for (var j = 0u; j < NSTEPS; j++) {
        let step = MAX_REVOLUTIONS * 2.0 * PI / f32(NSTEPS);

        // Leapfrog integration (in geometric units where r_s = 2.0)
        u += du * step;
        let ddu = -u * (1.0 - 1.5 * u * u);
        du += ddu * step;

        if (u < 0.0) {
            break;
        }

        phi += step;

        old_pos = pos;
        pos = (cos(phi) * normal_vec + sin(phi) * tangent_vec) / u;

        let r = 1.0 / u;

        // Check for event horizon (u > 0.5 means r < 2.0 in geometric units)
        if (u > 0.5) {
            hit_black_hole = true;
            break;
        }

        // Check for disk crossing (need to convert back to physical units)
        let pos_physical = pos * unit_scale;
        let old_pos_physical = old_pos * unit_scale;
        if (crosses_equatorial_plane(old_pos_physical, pos_physical)) {
            hit_disk = true;
            break;
        }

        // Check for planet intersection
        let ray_segment = pos_physical - old_pos_physical;
        let ray_length = length(ray_segment);
        if (ray_length > 0.0) {
            let ray_dir_norm = ray_segment / ray_length;
            let t = intersect_sphere(old_pos_physical, ray_dir_norm, planet.position, planet.radius);
            if (t >= 0.0 && t <= ray_length) {
                let hit_point = old_pos_physical + ray_dir_norm * t;
                planet_normal = normalize(hit_point - planet.position);
                hit_planet = true;
                break;
            }
        }

        // Escape condition
        if (r > 1000.0) {
            break;
        }
    }

    if (hit_black_hole) {
        color = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } else if (hit_planet) {
        // Simple Lambertian shading
        let light_dir = normalize(-planet.position);
        let diffuse = max(0.0, dot(planet_normal, light_dir));
        let ambient = 0.2;
        let brightness = ambient + (1.0 - ambient) * diffuse;
        let planet_color = vec3<f32>(0.4, 0.6, 0.9) * brightness;
        color = vec4<f32>(planet_color, 1.0);
    } else if (hit_disk) {
        let pos_physical = pos * unit_scale;
        let r = length(pos_physical) / disk.outer_radius;
        let disk_color = vec3<f32>(1.0, r, 0.2);
        color = vec4<f32>(disk_color, 1.0);
    } else {
        // Ray escaped - sample background using final ray direction (gravitationally bent!)
        let final_ray_dir = normalize(pos);
        let uv = direction_to_uv(final_ray_dir);

        // Convert UV to texture coordinates (textureLoad requires integer coordinates in compute shaders)
        let tex_dims = textureDimensions(background_texture);
        let tex_x = u32(uv.x * f32(tex_dims.x)) % tex_dims.x;
        let tex_y = u32(uv.y * f32(tex_dims.y)) % tex_dims.y;
        let bg_color = textureLoad(background_texture, vec2<u32>(tex_x, tex_y), 0);
        color = vec4<f32>(bg_color.rgb, 1.0);
    }

    textureStore(output_texture, vec2<i32>(pix), color);
}
