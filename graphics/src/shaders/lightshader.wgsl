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
    animate: u32,
    camera_type: u32,
};

struct DirLights {
    pos: vec2<f32>,
    color: u32,
    max_distance: f32,
    max_width: f32,
    anim_speed: f32,
    angle: f32,
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

const pi: f32 = 3.14159265;
const two_pi: f32 = 6.2831853;

fn flash_light(light_pos: vec2<f32>, pixel_pos: vec2<f32>, dir: f32, w_angle: f32, range: f32) -> f32 {
    let d = distance(light_pos, pixel_pos);
    let degree_radian = radians(dir);
    let w_radian = clamp(radians(w_angle), 0.0, 2.0 * pi);
    // Calculate the start angle from the direction angle and the angle width of the "cone".
    let s_angle = degree_radian - (w_radian / 2.0);
    // Calculate the start vector.
    let s = vec2<f32>(cos(s_angle), sin(s_angle));
    // Calculate the direction between the pixel position and the light position and normalize the vector.
    let direction = normalize(pixel_pos - light_pos);
    let p_pos = vec3<f32>(pixel_pos.x, pixel_pos.y, 0.0);
    let l_pos = vec3<f32>(light_pos.x, light_pos.y, 0.0);
    let d_dir = vec3<f32>(direction.x, direction.y, 0.0);

    // Only emit light if the direction projected onto the start vector is within the angle width of our cone. 1.0 - (d / range)
    return select(1.0 - (d / range), 0.0, atan2(length(cross(vec3(direction, 0.0), vec3(s, 0.0))), dot(direction, s)) >= w_radian);
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
            let value = 1.0 - (dist / cutoff);
            var color2 = col; 
            let alpha = select(color2.a, mix(color2.a, light_color.a, value), dist <= cutoff);
            color2.a = alpha;
            col = select(color2, mix(color2, light_color, vec4<f32>(value)), dist <= cutoff);
        }

        for(var i = 0u; i < min(vertex.dir_count, c_dir_lights); i += 1u) {
            let light = u_dirs[i];
            let light_pos = vec3<f32>(light.pos.x, light.pos.y, 1.0);
            var pos = vec4<f32>(light.pos.x, light.pos.y, 1.0, 1.0);
            var max_distance = light.max_distance;
            var max_width = light.max_width;

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
                }
                default: {}
            }

            let light_color = unpack_color(light.color);
            max_distance = max_distance - (f32(light.animate) *(1.0 * sin(global.seconds * light.anim_speed)));
            let dist_cutoff = max(0.1, max_distance);
            max_width = max_width - (f32(light.animate) *(1.0 * sin(global.seconds * light.anim_speed)));
            let width_cutoff = max(0.1, max_width);
            let value = flash_light(pos.xy, vertex.tex_coords.xy, light.angle, width_cutoff, dist_cutoff);
            var color2 = col; 
            let d = distance(pos.xy, vertex.tex_coords.xy);
            let alpha = select(color2.a, mix(color2.a, light_color.a, value),  d <= dist_cutoff);
            color2.a = alpha;
            col = select(color2, mix(color2, light_color, vec4<f32>(value)),  d <= dist_cutoff);
        }
    } 

    if (col.a <= 0.0) {
        discard;
    }

    return col;
}