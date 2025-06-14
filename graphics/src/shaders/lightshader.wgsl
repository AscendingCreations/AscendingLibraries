struct Global {
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
    inverse_proj: mat4x4<f32>,
    eye: vec3<f32>,
    scale: f32,
    size: vec2<f32>,
    seconds: f32,
    manual_view: mat4x4<f32>,
    manual_scale: f32,
};

struct AreaLights {
    pos: vec2<f32>,
    color: u32,
    max_distance: f32,
    anim_speed: f32,
    dither: f32,
    animate: u32,
    camera_type: u32,
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
    camera_type: u32,
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
    @location(5) pos: vec3<f32>,
    @location(6) size: vec2<f32>,
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
const c_dir_lights: u32 = 1333u;

@group(1)
@binding(0)
var<uniform> u_areas: array<AreaLights, c_area_lights>;
@group(2)
@binding(0)
var<uniform> u_dirs: array<DirLights, c_dir_lights>;

fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        return c / 12.92;
    } else {
        return pow((c + 0.055) / 1.055, 2.4);
    }
}

fn unpack_color(color: u32) -> vec4<f32> {
    return vec4<f32>(
        srgb_to_linear(f32((color & 0xff0000u) >> 16u) / 255.0),
        srgb_to_linear(f32((color & 0xff00u) >> 8u) / 255.0),
        srgb_to_linear(f32((color & 0xffu)) / 255.0),
        f32((color & 0xff000000u) >> 24u) / 255.0,
    );
}

@vertex
fn vertex(
    vertex: VertexInput,
) -> VertexOutput {
    var result: VertexOutput;
    let v = vertex.vertex_idx % 4u;

    switch v {
        case 1u: {
            result.clip_position = global.proj * vec4<f32>(vertex.pos.x + vertex.size.x, vertex.pos.y, vertex.pos.z, 1.0);
        }
        case 2u: {
            result.clip_position = global.proj * vec4<f32>(vertex.pos.x + vertex.size.x, vertex.pos.y + vertex.size.y, vertex.pos.z, 1.0);
        }
        case 3u: {
            result.clip_position = global.proj * vec4<f32>(vertex.pos.x, vertex.pos.y + vertex.size.y, vertex.pos.z, 1.0);
        }
        default: {
            result.clip_position = global.proj * vec4<f32>(vertex.pos.x, vertex.pos.y, vertex.pos.z, 1.0);
        }
    }

    result.tex_coords = global.inverse_proj * result.clip_position;
    result.tex_coords = result.tex_coords / result.tex_coords.w;
    result.col = vec4<f32>(srgb_to_linear(vertex.world_color.r), srgb_to_linear(vertex.world_color.g), srgb_to_linear(vertex.world_color.b), vertex.world_color.a);
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

    let angle3 = select(angle2 , angle2 - 360.0, angle2 > 180.0);
    return select(angle3, angle2 + 360.0, angle2 < -180.0);
}

fn within_range(testAngle: f32, a: f32, b: f32 ) -> bool {
    let a1 = a - testAngle;
    let b1 = b - testAngle;

    let a2 = normalize_180( a1 );
    let b2 = normalize_180( b1 );

    return select(abs( a2 - b2 ) < 180.0,false,a2 * b2 >= 0.0);
}

fn within_range_ret(testAngle: f32, a: f32, b: f32 ) -> RangeReturn {
    let a1 = a - testAngle;
    let b1 = b - testAngle;

    let a2 = normalize_180( a1 );
    let b2 = normalize_180( b1 );
    let angle = abs( a2 - b2 );
    return RangeReturn(select(false, angle < 180.0 ,a2 * b2 >= 0.0), select(0.0, angle, a2 * b2 >= 0.0));
}

fn flash_light(light_pos: vec2<f32>, pixel_pos: vec2<f32>, dir: f32, w_angle: f32, range: f32, dither: f32, edge_fade_percent: f32, edge_fade_dist: f32) -> f32 {
    let s_angle = dir - (w_angle / 2.0);
    let e_angle = dir + (w_angle / 2.0);
    let deg = normalize_360(atan2(pixel_pos.y - light_pos.y, pixel_pos.x - light_pos.x) * 180.0 / 3.14159265);
    let d = distance(light_pos, pixel_pos);

    if (d > range) {
        return 0.0;
    }

    let flash = select(0.0, fade(d, 0.0, 1.0, range - 2.0, dither), within_range(deg, s_angle + edge_fade_dist, e_angle - edge_fade_dist));
    return select(flash, max((1.0 - min(abs(deg - dir) / (w_angle + 4.0 / 2.0), 1.0)) - edge_fade_percent, 0.0) / (1.0 - edge_fade_percent), within_range(deg, s_angle, e_angle));
}

// Fragment shader
@fragment
fn fragment(vertex: VertexOutput,) -> @location(0) vec4<f32> {
    var col = vertex.col;

    if (vertex.enable_lights > 0u) {
        for(var i = 0u; i < min(vertex.area_count, c_area_lights); i += 1u) {
            let light = u_areas[i];
            let light_pos = vec3<f32>(light.pos.x, light.pos.y, 1.0);
            var pos = vec4<f32>(light.pos.x, light.pos.y, 1.0, 1.0);
            var max_distance = light.max_distance;

            switch light.camera_type {
                case 1u: {
                    pos = (global.view) * vec4<f32>(light_pos, 1.0);
                }
                case 2u: {
                    let scale_mat = mat4x4<f32> (
                        vec4<f32>(global.scale, 0.0, 0.0, 0.0),
                        vec4<f32>(0.0, global.scale, 0.0, 0.0),
                        vec4<f32>(0.0, 0.0, 1.0, 0.0),
                        vec4<f32>(0.0, 0.0, 0.0, 1.0),
                    );

                    pos = (global.view * scale_mat) * vec4<f32>(light_pos, 1.0);
                    max_distance = max_distance * global.scale;
                }
                case 3u: {
                    pos = (global.manual_view) * vec4<f32>(light_pos, 1.0);
                }
                case 4u: {
                    let scale_mat = mat4x4<f32> (
                        vec4<f32>(global.manual_scale, 0.0, 0.0, 0.0),
                        vec4<f32>(0.0, global.manual_scale, 0.0, 0.0),
                        vec4<f32>(0.0, 0.0, 1.0, 0.0),
                        vec4<f32>(0.0, 0.0, 0.0, 1.0),
                    );

                    pos = (global.manual_view * scale_mat) * vec4<f32>(light_pos, 1.0);
                    max_distance = max_distance * global.scale;
                }
                default: {}
            }

            let light_color = unpack_color(light.color);
            max_distance = max_distance - (f32(light.animate) *(1.0 * sin(global.seconds * light.anim_speed)));
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
            let light_pos = vec3<f32>(light.pos.x, light.pos.y, 1.0);
            var pos = vec4<f32>(light.pos.x, light.pos.y, 1.0, 1.0);
            var max_distance = light.max_distance;
            var max_width = light.max_width;
            var fade_distance = light.fade_distance;
            var edge_fade_distance = light.edge_fade_distance;

            switch light.camera_type {
                case 1u: {
                    pos = (global.view) * vec4<f32>(light_pos, 1.0);
                }
                case 2u: {
                    let scale_mat = mat4x4<f32> (
                        vec4<f32>(global.scale, 0.0, 0.0, 0.0),
                        vec4<f32>(0.0, global.scale, 0.0, 0.0),
                        vec4<f32>(0.0, 0.0, 1.0, 0.0),
                        vec4<f32>(0.0, 0.0, 0.0, 1.0),
                    );

                    pos = (global.view * scale_mat) * vec4<f32>(light_pos, 1.0);
                    max_distance = max_distance * global.scale;
                    max_width = max_width * global.scale;
                    fade_distance = fade_distance * global.scale;
                    edge_fade_distance = edge_fade_distance * global.scale;
                }
                case 3u: {
                    pos = (global.manual_view) * vec4<f32>(light_pos, 1.0);
                }
                case 4u: {
                    let scale_mat = mat4x4<f32> (
                        vec4<f32>(global.manual_scale, 0.0, 0.0, 0.0),
                        vec4<f32>(0.0, global.manual_scale, 0.0, 0.0),
                        vec4<f32>(0.0, 0.0, 1.0, 0.0),
                        vec4<f32>(0.0, 0.0, 0.0, 1.0),
                    );

                    pos = (global.manual_view * scale_mat) * vec4<f32>(light_pos, 1.0);
                    max_distance = max_distance * global.manual_scale;
                    max_width = max_width * global.manual_scale;
                    fade_distance = fade_distance * global.manual_scale;
                    edge_fade_distance = edge_fade_distance * global.manual_scale;
                }
                default: {}
            }

            let light_color = unpack_color(light.color);
            max_distance = max_distance - (f32(light.animate) *(1.0 * sin(global.seconds * light.anim_speed)));
            let dist_cutoff = max(0.1, max_distance);
            max_width = max_width - (f32(light.animate) *(1.0 * sin(global.seconds * light.anim_speed)));
            let width_cutoff = max(0.1, max_width);
            let value = flash_light(pos.xy, vertex.tex_coords.xy, light.angle, width_cutoff, dist_cutoff, light.dither, edge_fade_distance, fade_distance);
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