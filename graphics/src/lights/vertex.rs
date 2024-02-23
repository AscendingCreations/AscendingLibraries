use crate::{BufferData, BufferLayout};
use std::iter;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightsVertex {
    pub world_color: [f32; 4],
    pub enable_lights: u32,
    pub dir_count: u32,
    pub area_count: u32,
}

impl Default for LightsVertex {
    fn default() -> Self {
        Self {
            world_color: [0.0; 4],
            enable_lights: 0,
            dir_count: 0,
            area_count: 0,
        }
    }
}

impl BufferLayout for LightsVertex {
    fn attributes() -> Vec<wgpu::VertexAttribute> {
        wgpu::vertex_attr_array![1 => Float32x4, 2 => Uint32, 3 => Uint32, 4 => Uint32 ].to_vec()
    }

    ///default set as large enough to contain 10_000 sprites.
    fn default_buffer() -> BufferData {
        Self::with_capacity(10_000, 0)
    }

    fn with_capacity(
        vertex_capacity: usize,
        _index_capacity: usize,
    ) -> BufferData {
        let instance_arr: Vec<LightsVertex> =
            iter::repeat(LightsVertex::default())
                .take(vertex_capacity)
                .collect();

        BufferData {
            vertexs: bytemuck::cast_slice(&instance_arr).to_vec(),
            ..Default::default()
        }
    }

    fn stride() -> usize {
        std::mem::size_of::<[f32; 7]>()
    }
}
