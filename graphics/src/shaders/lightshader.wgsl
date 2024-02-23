struct Global {
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
    inverse_proj: mat4x4<f32>,
    eye: vec3<f32>,
    scale: f32,
    size: vec2<f32>,
    seconds: f32,
};

struct AreaLights {
    pos: vec2<f32>,
    color: u32,
    max_distance: f32,
    anim_speed: f32,
    dither: f32,
    animate: u32,
};

struct RangeReturn {
    within: bool,
    angle: f32,
};

struct DirLights {
    pos: vec2<f32>,
    color: u32,
    max_distance: f32,
    max_width: f32,
    anim_speed: f32,
    angle: f32,
    dither: f32,
    fade_distance: f32,
    edge_fade_distance: f32,
    animate: u32,
};

@group(0)
@binding(0)
var<uniform> global: Global;

struct VertexInput {
    @builtin(vertex_index) vertex_idx: u32,
    @location(0) v_pos: vec2<f32>,
    @location(1) world_color: vec4<f32>,
    @location(2) enable_lights: u32,
    @location(3) dir_count: u32,
    @location(4) area_count: u32,
};

struct VertexOutput {
    @invariant @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec4<f32>,
    @location(1) col: vec4<f32>,
    @location(2) enable_lights: u32,
    @location(3) dir_count: u32,
    @location(4) area_count: u32,
};

const c_area_lights: u32 = 2000u;
const c_dir_lights: u32 = 1365u;

@group(1)
@binding(0)
var<uniform> u_areas: array<AreaLights, c_area_lights>;
@group(2)
@binding(0)
var<uniform> u_dirs: array<DirLights, c_dir_lights>;

fn unpack_color(color: u32) -> vec4<f32> {
    return vec4<f32>(
        f32((color & 0xff0000u) >> 16u),
        f32((color & 0xff00u) >> 8u),
        f32((color & 0xffu)),
        f32((color & 0xff000000u) >> 24u),
    ) / 255.0;
}

@vertex
fn vertex(
    vertex: VertexInput,
) -> VertexOutput {
    var result: VertexOutput;
    let v = vertex.vertex_idx % 4u;

    switch v {
        case 1u: {
            result.clip_position = global.proj * vec4<f32>(global.size.x, 0.0, 1.0, 1.0);
        }
        case 2u: {
            result.clip_position = global.proj * vec4<f32>(global.size.x, global.size.y, 1.0, 1.0);
        }
        case 3u: {
            result.clip_position = global.proj * vec4<f32>(0.0, global.size.y, 1.0, 1.0);
        }
        default: {
            result.clip_position = global.proj * vec4<f32>(0.0, 0.0, 1.0, 1.0);
        }
    }

    result.tex_coords = global.inverse_proj * result.clip_position;
    result.tex_coords = result.tex_coords / result.tex_coords.w;
    result.col = vertex.world_color;
    result.enable_lights = vertex.enable_lights;
    result.dir_count = vertex.dir_count;
    result.area_count = vertex.area_count;
    return result;
}

fn fade(d: f32, x0: f32, x1: f32, c: f32, w: f32) -> f32 {
   let w1 = max(0.000001, w);
   let sD = 1.0 / (1.0 + exp(-(c-d)/w1));
   return x1 - (x0 + (x1 - x0)*(1.0 - sD));
}

fn normalize_360(angle: f32) -> f32 {
    return angle % 360.0;
}

fn normalize_180(angle: f32) -> f32 {
    let angle2 = normalize_360(angle);
    if angle2 > 180.0 {
        return angle2 - 360.0;
    } else {
        if angle2 < -180.0 {
           return angle2 + 360.0;
        } else {
            return angle2;
        }
    }
}

fn within_range(testAngle: f32, a: f32, b: f32 ) -> bool {
    let a1 = a - testAngle;
    let b1 = b - testAngle;

    let a2 = normalize_180( a1 );
    let b2 = normalize_180( b1 );

    if ( a2 * b2 >= 0.0 ) {
        return false;
    } else {
        return abs( a2 - b2 ) < 180.0;
    }
}

fn within_range_ret(testAngle: f32, a: f32, b: f32 ) -> RangeReturn {
    let a1 = a - testAngle;
    let b1 = b - testAngle;

    let a2 = normalize_180( a1 );
    let b2 = normalize_180( b1 );

    if ( a2 * b2 >= 0.0 ) {
        return RangeReturn(false, 0.0);
    } else {
        let angle = abs( a2 - b2 );
        return RangeReturn(angle < 180.0, angle);
    }
}

fn flash_light(light_pos: vec2<f32>, pixel_pos: vec2<f32>, dir: f32, w_angle: f32, range: f32, dither: f32, edge_fade_percent: f32, edge_fade_dist: f32) -> f32 {
    let s_angle = dir - (w_angle / 2.0);
    let e_angle = dir + (w_angle / 2.0);
    let deg = normalize_360(atan2(pixel_pos.y - light_pos.y, pixel_pos.x - light_pos.x) * 180.0 / 3.14159265);

    if (within_range(deg, s_angle + edge_fade_dist, e_angle - edge_fade_dist)) {
        let d = distance(light_pos, pixel_pos);

        if (d > range) {
            return 0.0;
        }

        return fade(d, 0.0, 1.0, range - 2.0, dither);
    } 

    if (within_range(deg, s_angle, e_angle)) {
        let d = distance(light_pos, pixel_pos);

        if (d > range) {
            return 0.0;
        }

        return max((1.0 - min(abs(deg - dir) / (w_angle + 4.0 / 2.0), 1.0)) - edge_fade_percent, 0.0) / (1.0 - edge_fade_percent);
    }

    return 0.0;
}

// Fragment shader
@fragment
fn fragment(vertex: VertexOutput,) -> @location(0) vec4<f32> {
    var col = vertex.col;

    if (vertex.enable_lights > 0u) {
        for(var i = 0u; i < min(vertex.area_count, c_area_lights); i += 1u) {
            let light = u_areas[i];
            let light_color = unpack_color(light.color);
            let pos = vec2<f32>(light.pos.x, light.pos.y);
            let max_distance = light.max_distance - (f32(light.animate) *(1.0 * sin(global.seconds * light.anim_speed)));
            let dist = distance(pos.xy, vertex.tex_coords.xy);
            let cutoff = max(0.1, max_distance);
            let value = fade(dist, 0.0, 1.0, cutoff, light.dither);
            var color2 = col; 
            let alpha = mix(color2.a, light_color.a, value);
            color2.a = alpha;
            col = mix(color2, light_color, vec4<f32>(value));
        }

        for(var i = 0u; i < min(vertex.dir_count, c_dir_lights); i += 1u) {
            let light = u_dirs[i];
            let light_color = unpack_color(light.color);
            let max_distance = light.max_distance - (f32(light.animate) *(1.0 * sin(global.seconds * light.anim_speed)));
            let dist_cutoff = max(0.1, max_distance);
            let max_width = light.max_width - (f32(light.animate) *(1.0 * sin(global.seconds * light.anim_speed)));
            let width_cutoff = max(0.1, max_width);
            let value = flash_light(light.pos, vertex.tex_coords.xy, light.angle, width_cutoff, dist_cutoff, light.dither, light.edge_fade_distance, light.fade_distance);
            var color2 = col; 
            let alpha = mix(color2.a, light_color.a, value);
            color2.a = alpha;
            col = mix(color2, light_color, vec4<f32>(value));
        }
    } 

    if (col.a <= 0.0) {
        discard;
    }

    return col;
}