use crate::{BufferData, BufferLayout};
use std::iter;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RectVertex {
    pub position: [f32; 3],
    pub size: [f32; 2],
    pub uv: [f32; 4],
    pub color: u32,
    pub border_width: f32,
    pub border_color: u32,
    pub layer: u32,
    pub radius: f32,
    pub use_camera: u32,
}

impl Default for RectVertex {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            size: [0.0; 2],
            uv: [0.0; 4],
            color: 0,
            border_width: 0.0,
            border_color: 0,
            layer: 0,
            radius: 1.0,
            use_camera: 0,
        }
    }
}

impl BufferLayout for RectVertex {
    fn attributes() -> Vec<wgpu::VertexAttribute> {
        wgpu::vertex_attr_array![1 => Float32x3, 2 => Float32x2, 3 => Float32x4, 4 => Uint32, 5 => Float32, 6 => Uint32, 7 => Uint32, 8 => Float32, 9 => Uint32]
            .to_vec()
    }

    ///default set as large enough to contain 1_000 shapes.
    fn default_buffer() -> BufferData {
        Self::with_capacity(1_000, 0)
    }

    fn with_capacity(
        vertex_capacity: usize,
        _index_capacity: usize,
    ) -> BufferData {
        let instance_arr: Vec<RectVertex> = iter::repeat(RectVertex::default())
            .take(vertex_capacity)
            .collect();

        BufferData {
            vertexs: bytemuck::cast_slice(&instance_arr).to_vec(),
            ..Default::default()
        }
    }

    fn stride() -> usize {
        std::mem::size_of::<[f32; 15]>()
    }
}
