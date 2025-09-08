use crate::{BufferData, BufferLayout};
#[cfg(feature = "rayon")]
use rayon::{iter::repeat_n, prelude::*};

/// Vertex Details for [`crate::Image`] that matches the Shaders Vertex Layout.
///
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ImageVertex {
    pub pos: [f32; 3],
    pub size: [f32; 2],
    pub tex_data: [f32; 4],
    pub color: u32,
    pub frames: [f32; 2],
    pub animate: u32,
    pub camera_type: u32,
    pub time: u32,
    pub layer: i32,
    pub angle: f32,
    pub flip_style: u32,
}

impl Default for ImageVertex {
    fn default() -> Self {
        Self {
            pos: [0.0; 3],
            size: [0.0; 2],
            tex_data: [0.0; 4],
            color: 0,
            frames: [0.0; 2],
            animate: 0,
            camera_type: 1,
            time: 0,
            layer: 0,
            angle: 0.0,
            flip_style: 0,
        }
    }
}

impl BufferLayout for ImageVertex {
    fn attributes() -> Vec<wgpu::VertexAttribute> {
        wgpu::vertex_attr_array![1 => Float32x3, 2 => Float32x2, 3 => Float32x4, 4 => Uint32, 5 => Float32x2, 6 => Uint32, 7 => Uint32, 8 => Uint32, 9 => Sint32, 10 => Float32, 11 => Uint32 ]
            .to_vec()
    }

    fn default_buffer() -> BufferData {
        Self::with_capacity(10_000, 0)
    }

    fn with_capacity(
        vertex_capacity: usize,
        _index_capacity: usize,
    ) -> BufferData {
        #[cfg(feature = "rayon")]
        let instance_arr: Vec<ImageVertex> =
            repeat_n(ImageVertex::default(), vertex_capacity).collect();

        #[cfg(not(feature = "rayon"))]
        let instance_arr: Vec<ImageVertex> =
            std::iter::repeat_n(ImageVertex::default(), vertex_capacity)
                .collect();

        BufferData {
            vertexs: bytemuck::cast_slice(&instance_arr).to_vec(),
            ..Default::default()
        }
    }

    fn stride() -> usize {
        std::mem::size_of::<[f32; 18]>()
    }
}
