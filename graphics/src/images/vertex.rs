use crate::{BufferData, BufferLayout};
use std::iter;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ImageVertex {
    pub position: [f32; 3],
    pub hw: [f32; 2],
    pub tex_data: [f32; 4],
    pub color: u32,
    pub frames: [f32; 2],
    pub animate: u32,
    pub use_camera: u32,
    pub time: u32,
    pub layer: i32,
}

impl Default for ImageVertex {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            hw: [0.0; 2],
            tex_data: [0.0; 4],
            color: 0,
            frames: [0.0; 2],
            animate: 0,
            use_camera: 1,
            time: 0,
            layer: 0,
        }
    }
}

impl BufferLayout for ImageVertex {
    fn attributes() -> Vec<wgpu::VertexAttribute> {
        wgpu::vertex_attr_array![1 => Float32x3, 2 => Float32x2, 3 => Float32x4, 4 => Uint32, 5 => Float32x2, 6 => Uint32, 7 => Uint32,8 => Uint32, 9 => Sint32 ]
            .to_vec()
    }

    ///default set as large enough to contain 10_000 sprites.
    fn default_buffer() -> BufferData {
        Self::with_capacity(10_000, 0)
    }

    fn with_capacity(
        vertex_capacity: usize,
        _index_capacity: usize,
    ) -> BufferData {
        let instance_arr: Vec<ImageVertex> =
            iter::repeat(ImageVertex::default())
                .take(vertex_capacity)
                .collect();

        BufferData {
            vertexs: bytemuck::cast_slice(&instance_arr).to_vec(),
            ..Default::default()
        }
    }

    fn stride() -> usize {
        std::mem::size_of::<[f32; 16]>()
    }
}
