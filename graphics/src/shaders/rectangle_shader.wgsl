struct Global {
    view: mat4x4<f32>, //64
    proj: mat4x4<f32>,
    inverse_proj: mat4x4<f32>,
    eye: vec3<f32>,
    scale: f32,
    size: vec2<f32>,
    seconds: f32,
};

@group(0)
@binding(0)
var<uniform> global: Global;


struct VertexInput {
    @builtin(vertex_index) vertex_idx: u32,
    @location(0) v_pos: vec2<f32>,
    @location(1) position: vec3<f32>,
    @location(2) size: vec2<f32>,
    @location(3) uv: vec4<f32>,
    @location(4) color: u32,
    @location(5) border_width: f32,
    @location(6) border_color: u32,
    @location(7) layer: u32,
    @location(8) radius: f32,
    @location(9) use_camera: u32,
};

struct VertexOutput {
    @invariant @builtin(position) clip_position: vec4<f32>,
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(3) container_data: vec4<f32>,
    @location(4) color: vec4<f32>,
    @location(5) border_color: vec4<f32>,
    @location(6) size: vec2<f32>,
    @location(7) border_width: f32,
    @location(8) radius: f32,
    @location(9) layer: i32,
    @location(10) tex_size: vec2<f32>,
};

@group(1)
@binding(0)
var tex: texture_2d_array<f32>;
@group(1)
@binding(1)
var tex_sample: sampler;

fn unpack_tex_data(data: vec2<u32>) -> vec4<u32> {
    return vec4<u32>(
        u32(data[0] & 0xffffu), 
        u32((data[0] & 0xffff0000u) >> 16u),
        u32(data[1] & 0xffffu),
        u32((data[1] & 0xffff0000u) >> 16u)
    );
}

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
    let tex_data = vertex.uv;
    let size = textureDimensions(tex);
    let fsize = vec2<f32> (f32(size.x), f32(size.y));
    var pos = vertex.position;

     switch v {
        case 1u: {
            result.uv = vec2<f32>(tex_data[2], tex_data[3]);
            pos.x += vertex.size.x;
        }
        case 2u: {
            result.uv = vec2<f32>(tex_data[2], 0.0);
            pos.x += vertex.size.x;
            pos.y += vertex.size.y;
        }
        case 3u: {
            result.uv = vec2<f32>(0.0, 0.0);
            pos.y += vertex.size.y;
        }
        default: {
            result.uv = vec2<f32>(0.0, tex_data[3]);
        }
    }

    if (vertex.use_camera == 1u) {
        result.clip_position = (global.proj * global.view) * vec4<f32>(pos, 1.0);
    } else {
        result.clip_position = global.proj * vec4<f32>(pos, 1.0);
    }

    result.container_data = tex_data;
    result.border_width = vertex.border_width;
    result.size = vertex.size * global.scale;
    result.position = vertex.position.xy * global.scale;
    result.radius = vertex.radius;
    result.tex_size = fsize;
    result.layer = i32(vertex.layer);
    result.color = unpack_color(vertex.color);
    result.border_color = unpack_color(vertex.border_color);
    return result;
}

fn distance_alg(
    frag_coord: vec2<f32>,
    position: vec2<f32>,
    size: vec2<f32>,
    radius: f32
) -> f32 {
    var inner_size: vec2<f32> = size - vec2<f32>(radius, radius) * 2.0;
    var top_left: vec2<f32> = position + vec2<f32>(radius, radius);
    var bottom_right: vec2<f32> = top_left + inner_size;

    var top_left_distance: vec2<f32> =  top_left - frag_coord;
    var bottom_right_distance: vec2<f32> = frag_coord - bottom_right;

    var dist: vec2<f32> = vec2<f32>(
        max(max(top_left_distance.x, bottom_right_distance.x), 0.0),
        max(max(top_left_distance.y, bottom_right_distance.y), 0.0)
    );

    return sqrt(dist.x * dist.x + dist.y * dist.y);
}

@fragment
fn fragment(vertex: VertexOutput,) -> @location(0) vec4<f32> {
    var container_color = vertex.color;

    if (vertex.container_data[2] > 0.0 || vertex.container_data[3] > 0.0 ) {
        let coords = vec2<f32>(
            (vertex.container_data[0] + vertex.uv.x) / vertex.tex_size.x,
            (vertex.container_data[1] + vertex.uv.y) / vertex.tex_size.y
        );

        var step = vec2<f32>(0.5, 0.5);
        var tex_pixel = vertex.tex_size * coords - step.xy / 2.0;

        let corner = floor(tex_pixel) + 1.0;
        let frac = min((corner - tex_pixel) * vec2<f32>(2.0, 2.0), vec2<f32>(1.0, 1.0));

        var c1 = textureSampleLevel(tex, tex_sample, (floor(tex_pixel + vec2<f32>(0.0, 0.0)) + 0.5) / vertex.tex_size, vertex.layer, 1.0);
        var c2 = textureSampleLevel(tex, tex_sample, (floor(tex_pixel + vec2<f32>(step.x, 0.0)) + 0.5) / vertex.tex_size, vertex.layer, 1.0);
        var c3 = textureSampleLevel(tex, tex_sample, (floor(tex_pixel + vec2<f32>(0.0, step.y)) + 0.5) / vertex.tex_size, vertex.layer, 1.0);
        var c4 = textureSampleLevel(tex, tex_sample, (floor(tex_pixel + step.xy) + 0.5) / vertex.tex_size, vertex.layer, 1.0);

        c1 = c1 * (frac.x * frac.y);
        c2 = c2 *((1.0 - frac.x) * frac.y);
        c3 = c3 * (frac.x * (1.0 - frac.y));
        c4 = c4 *((1.0 - frac.x) * (1.0 - frac.y));

        container_color = (c1 + c2 + c3 + c4) * container_color;
    }

    var mixed_color: vec4<f32> = container_color;
    let radius = vertex.radius;
    let clippy = vec2<f32>(vertex.clip_position.x, global.size.y - vertex.clip_position.y);

    if (vertex.border_width > 0.0) {
        var border: f32 = max(radius - vertex.border_width, 0.0);

        let distance = distance_alg( 
            clippy, 
            vertex.position.xy + vec2<f32>(vertex.border_width), 
            vertex.size - vec2<f32>(vertex.border_width * 2.0), 
            border 
        );

        let border_mix: f32 = smoothstep(
            max(border - 0.5, 0.0),
            border + 0.5,
            distance
        );

        mixed_color = mix(container_color, vertex.border_color, vec4<f32>(border_mix));
    }

    let dist: f32 = distance_alg(
        clippy,
        vertex.position.xy,
        vertex.size,
        radius
    );

    let radius_alpha: f32 = 1.0 - smoothstep(
        max(radius - 0.5, 0.0),
        radius + 0.5,
        dist);

    let alpha = mixed_color.a * radius_alpha;

    if (alpha <= 0.0) {
        discard;
    }

    return vec4<f32>(mixed_color.r, mixed_color.g, mixed_color.b, alpha);
}