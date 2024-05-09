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
    @location(7) camera_type: u32,
    @location(8) time: u32,
    @location(9) layer: i32,
    @location(10) angle: f32,
    @location(11) flip_style: u32,
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

struct Axises {
    x: vec4<f32>,
    y: vec4<f32>,
    z: vec4<f32>,
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

fn quat_to_axes(quat: vec4<f32>) -> Axises {
    var result: Axises;
    let x2 = quat.x + quat.x;
    let y2 = quat.y + quat.y;
    let z2 = quat.z + quat.z;
    let xx = quat.x * x2;
    let xy = quat.x * y2;
    let xz = quat.x * z2;
    let yy = quat.y * y2;
    let yz = quat.y * z2;
    let zz = quat.z * z2;
    let wx = quat.w * x2;
    let wy = quat.w * y2;
    let wz = quat.w * z2;

    let x_axis = vec4<f32>(1.0 - (yy + zz), xy + wz, xz - wy, 0.0);
    let y_axis = vec4<f32>(xy - wz, 1.0 - (xx + zz), yz + wx, 0.0);
    let z_axis = vec4<f32>(xz + wy, yz - wx, 1.0 - (xx + yy), 0.0);

    result.x = x_axis;
    result.y = y_axis;
    result.z = z_axis;

    return result;
}

fn quat_to_rotation_mat4(quat: vec4<f32>) -> mat4x4<f32> {
    let axises = quat_to_axes(quat);

    return mat4x4<f32> (
        axises.x,
        axises.y,
        axises.z,
        vec4<f32>(0.0, 0.0, 0.0, 1.0),
    );
}

fn flip_mat4(flip_style: u32) -> mat4x4<f32> {
    switch flip_style {
        case 1u: {
            return mat4x4<f32> (
                vec4<f32>(-1.0, 0.0, 0.0, 0.0),
                vec4<f32>(0.0, 1.0, 0.0, 0.0),
                vec4<f32>(0.0, 0.0, 1.0, 0.0),
                vec4<f32>(0.0, 0.0, 0.0, 1.0),
            );}
        case 2u: {
            return mat4x4<f32> (
                vec4<f32>(1.0, 0.0, 0.0, 0.0),
                vec4<f32>(0.0, -1.0, 0.0, 0.0),
                vec4<f32>(0.0, 0.0, 1.0, 0.0),
                vec4<f32>(0.0, 0.0, 0.0, 1.0),
            );}
        case 3u: {
            return mat4x4<f32> (
                vec4<f32>(-1.0, 0.0, 0.0, 0.0),
                vec4<f32>(0.0, -1.0, 0.0, 0.0),
                vec4<f32>(0.0, 0.0, 1.0, 0.0),
                vec4<f32>(0.0, 0.0, 0.0, 1.0),
            );}
        default: {
            return mat4x4<f32> (
                vec4<f32>(1.0, 0.0, 0.0, 0.0),
                vec4<f32>(0.0, 1.0, 0.0, 0.0),
                vec4<f32>(0.0, 0.0, 1.0, 0.0),
                vec4<f32>(0.0, 0.0, 0.0, 1.0),
            );
        }
    }
}

fn flip_rotation_mat4(flip_style: u32, angle: f32, pos: vec2<f32>, hw: vec2<f32>, scale: f32) -> mat4x4<f32> {
    let flip = flip_mat4(flip_style);
    let rotation = quat_to_rotation_mat4(quat_from_rotation_z(angle));
    let scale_mat = mat4x4<f32> (
                vec4<f32>(global.scale, 0.0, 0.0, 0.0),
                vec4<f32>(0.0, global.scale, 0.0, 0.0),
                vec4<f32>(0.0, 0.0, 1.0, 0.0),
                vec4<f32>(0.0, 0.0, 0.0, 1.0),
            );
    let inverse_trans = mat4x4<f32> (
                vec4<f32>(1.0, 0.0, 0.0, 0.0),
                vec4<f32>(0.0, 1.0, 0.0, 0.0),
                vec4<f32>(0.0, 0.0, 1.0, 0.0),
                vec4<f32>((-pos.x - hw.x / 2.0) , (-pos.y - hw.y / 2.0), 0.0, 1.0),
            );
    let trans = mat4x4<f32> (
                vec4<f32>(1.0, 0.0, 0.0, 0.0),
                vec4<f32>(0.0, 1.0, 0.0, 0.0),
                vec4<f32>(0.0, 0.0, 1.0, 0.0),
                vec4<f32>(pos.x + hw.x / 2.0 , pos.y + hw.y / 2.0, 0.0, 1.0),
            );

    return trans * scale_mat * flip * rotation * inverse_trans;
}

fn quat_from_rotation_z(angle: f32) -> vec4<f32>
{
    let half_angle = (angle * 0.5) * 3.14159 / 180.0;
    return vec4<f32>(0.0, 0.0, sin(half_angle), cos(half_angle));
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

    switch vertex.camera_type {
        case 1u: {
            let r_f = flip_rotation_mat4(vertex.flip_style, vertex.angle, vertex.v_pos + vertex.position.xy, vertex.hw, 1.0);
            result.clip_position = (global.proj * global.view) * r_f * vec4<f32>(pos, 1.0);
        }
        case 2u: {
            let r_f = flip_rotation_mat4(vertex.flip_style, vertex.angle, vertex.v_pos + vertex.position.xy, vertex.hw, global.scale);
            result.clip_position = (global.proj * global.view) * r_f * vec4<f32>(pos, 1.0);
        }
        case 3u: {
            let r_f = flip_rotation_mat4(vertex.flip_style, vertex.angle, vertex.v_pos + vertex.position.xy, vertex.hw, 1.0);
            result.clip_position = (global.proj * global.manual_view) * r_f * vec4<f32>(pos, 1.0);
        }
        case 4u: {
            let r_f = flip_rotation_mat4(vertex.flip_style, vertex.angle, vertex.v_pos + vertex.position.xy, vertex.hw, global.manual_scale);
            result.clip_position = (global.proj * global.manual_view) * r_f * vec4<f32>(pos, 1.0);
        }
        default: {
            let r_f = flip_rotation_mat4(vertex.flip_style, vertex.angle, vertex.v_pos + vertex.position.xy, vertex.hw, 1.0);
            result.clip_position = global.proj  * r_f * vec4<f32>(pos, 1.0);
        }
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