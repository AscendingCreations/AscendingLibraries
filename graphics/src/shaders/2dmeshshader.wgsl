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
    @location(0) position: vec3<f32>,
    @location(1) color: u32,
    @location(2) use_camera: u32,
};

struct VertexOutput {
    @invariant @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

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
    var pos = vertex.position;

    if (vertex.use_camera == 1u) {
        result.clip_position = (global.proj * global.view) * vec4<f32>(pos, 1.0);
    } else {
        result.clip_position = global.proj * vec4<f32>(pos, 1.0);
    }

    result.color = unpack_color(vertex.color);
    return result;
}

@fragment
fn fragment(vertex: VertexOutput,) -> @location(0) vec4<f32> {
    return vertex.color;
}