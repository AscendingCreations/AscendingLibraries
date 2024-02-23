struct Global {
    view: mat4x4<f32>,
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
    @location(2) hw: vec2<f32>,
    @location(3) tex_data: vec4<f32>,
    @location(4) color: u32,
    @location(5) frames: vec2<f32>,
    @location(6) animate: u32,
    @location(7) use_camera: u32,
    @location(8) time: u32,
    @location(9) layer: i32,
};

struct VertexOutput {
    @invariant @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) tex_data: vec4<f32>,
    @location(2) col: vec4<f32>,
    @location(3) frames: vec2<u32>,
    @location(4) size: vec2<f32>,
    @location(5) layer: i32,
    @location(6) time: u32,
    @location(7) animate: u32,
};

@group(1)
@binding(0)
var tex: texture_2d_array<f32>;
@group(1)
@binding(1)
var tex_sample: sampler;

fn unpack_color(color: u32) -> vec4<f32> {
    return vec4<f32>(
        f32((color & 0xff0000u) >> 16u),
        f32((color & 0xff00u) >> 8u),
        f32((color & 0xffu)),
        f32((color & 0xff000000u) >> 24u),
    ) / 255.0;
}

fn unpack_tex_data(data: vec2<u32>) -> vec4<u32> {
    return vec4<u32>(
        u32(data[0] & 0xffffu), 
        u32((data[0] & 0xffff0000u) >> 16u),
        u32(data[1] & 0xffffu),
        u32((data[1] & 0xffff0000u) >> 16u)
    );
}

@vertex
fn vertex(
    vertex: VertexInput,
) -> VertexOutput {
    var result: VertexOutput;
    let v = vertex.vertex_idx % 4u;
    let size = textureDimensions(tex);
    let fsize = vec2<f32> (f32(size.x), f32(size.y));
    let tex_data = vertex.tex_data;
    var pos = vertex.position;

    switch v {
        case 1u: {
            result.tex_coords = vec2<f32>(tex_data[2], tex_data[3]);
            pos.x += vertex.hw.x;
        }
        case 2u: {
            result.tex_coords = vec2<f32>(tex_data[2], 0.0);
            pos.x += vertex.hw.x;
            pos.y += vertex.hw.y;
        }
        case 3u: {
            result.tex_coords = vec2<f32>(0.0, 0.0);
            pos.y += vertex.hw.y;
        }
        default: {
            result.tex_coords = vec2<f32>(0.0, tex_data[3]);
        }
    }

    if (vertex.use_camera == 1u) {
        result.clip_position = (global.proj * global.view) * vec4<f32>(pos, 1.0);
    } else {
        result.clip_position = global.proj * vec4<f32>(pos, 1.0);
    }

    result.tex_data = tex_data;
    result.layer = vertex.layer;
    result.col = unpack_color(vertex.color);
    result.frames = vec2<u32>(u32(vertex.frames[0]), u32(vertex.frames[1]));
    result.size = fsize;
    result.animate = vertex.animate;
    result.time = vertex.time;
    return result;
}

// Fragment shader
@fragment
fn fragment(vertex: VertexOutput,) -> @location(0) vec4<f32> {
    var coords = vec2<f32>(0.0, 0.0);
    let xframes = vertex.frames[0];
    var yframes = vertex.frames[0];

    if (vertex.animate > 0u) {
        let id = global.seconds / (f32(vertex.time) / 1000.0);
        let frame = u32(floor(id % f32(xframes)));

        if (vertex.frames[1] > 0u) {
            yframes = vertex.frames[1];
        }

        coords = vec2<f32>(
            (f32((f32(frame % yframes) * vertex.tex_data[2]) + vertex.tex_data[0]) + vertex.tex_coords.x) / vertex.size.x,
            (f32((f32(frame / yframes) * vertex.tex_data[3]) + vertex.tex_data[1]) + vertex.tex_coords.y) / vertex.size.y
        );
    } else {
        coords = vec2<f32>(
            (vertex.tex_data[0] + vertex.tex_coords.x) / vertex.size.x,
            (vertex.tex_data[1] + vertex.tex_coords.y) / vertex.size.y
        );
    }

    var step = vec2<f32>(0.5, 0.5);
    var tex_pixel = vertex.size * coords - step.xy / 2.0;

    let corner = floor(tex_pixel) + 1.0;
    let frac = min((corner - tex_pixel) * vec2<f32>(2.0, 2.0), vec2<f32>(1.0, 1.0));

    var c1 = textureSampleLevel(tex, tex_sample, (floor(tex_pixel + vec2<f32>(0.0, 0.0)) + 0.5) / vertex.size, vertex.layer, 1.0);
    var c2 = textureSampleLevel(tex, tex_sample, (floor(tex_pixel + vec2<f32>(step.x, 0.0)) + 0.5) / vertex.size, vertex.layer, 1.0);
    var c3 = textureSampleLevel(tex, tex_sample, (floor(tex_pixel + vec2<f32>(0.0, step.y)) + 0.5) / vertex.size, vertex.layer, 1.0);
    var c4 = textureSampleLevel(tex, tex_sample, (floor(tex_pixel + step.xy) + 0.5) / vertex.size, vertex.layer, 1.0);

    c1 = c1 * (frac.x * frac.y);
    c2 = c2 *((1.0 - frac.x) * frac.y);
    c3 = c3 * (frac.x * (1.0 - frac.y));
    c4 = c4 *((1.0 - frac.x) * (1.0 - frac.y));

    let object_color = (c1 + c2 + c3 + c4) * vertex.col;

    if (object_color.a <= 0.0) {
        discard;
    }

    return object_color;
}