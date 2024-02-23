use crate::{BufferData, BufferLayout};
use std::iter;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TextVertex {
    pub position: [f32; 3],
    pub hw: [f32; 2],
    pub tex_coord: [f32; 2],
    pub layer: u32,
    pub color: u32,
    pub use_camera: u32,
    pub is_color: u32,
}

impl Default for TextVertex {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 1.0],
            hw: [0.0; 2],
            tex_coord: [0.0; 2],
            layer: 0,
            color: 0,
            use_camera: 0,
            is_color: 0,
        }
    }
}

impl BufferLayout for TextVertex {
    fn attributes() -> Vec<wgpu::VertexAttribute> {
        wgpu::vertex_attr_array![1 => Float32x3, 2 => Float32x2, 3 => Float32x2, 4 => Uint32, 5 => Uint32, 6 => Uint32, 7 => Uint32]
            .to_vec()
    }

    ///default set as large enough to contain 1024 glyphs.
    fn default_buffer() -> BufferData {
        Self::with_capacity(1024, 0)
    }

    fn with_capacity(
        vertex_capacity: usize,
        _index_capacity: usize,
    ) -> BufferData {
        let instance_arr: Vec<TextVertex> = iter::repeat(TextVertex::default())
            .take(vertex_capacity)
            .collect();

        BufferData {
            vertexs: bytemuck::cast_slice(&instance_arr).to_vec(),
            ..Default::default()
        }
    }

    fn stride() -> usize {
        std::mem::size_of::<[f32; 11]>()
    }
}
