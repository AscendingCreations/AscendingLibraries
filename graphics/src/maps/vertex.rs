use crate::{BufferData, BufferLayout};
use std::iter;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
/// 4 of these per each layer.
pub struct MapVertex {
    pub position: [f32; 3],
    pub tilesize: f32,
    pub tile_id: u32,
    pub texture_layer: u32,
    pub color: u32,
}

impl Default for MapVertex {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            tilesize: 0.0,
            tile_id: 0,
            texture_layer: 0,
            color: 0,
        }
    }
}

impl BufferLayout for MapVertex {
    fn attributes() -> Vec<wgpu::VertexAttribute> {
        wgpu::vertex_attr_array![1 => Float32x3, 2 => Float32, 3 => Uint32, 4 => Uint32, 5 => Uint32]
            .to_vec()
    }

    /// default set as 8,192 tile per map.
    fn default_buffer() -> BufferData {
        Self::with_capacity(8_192, 0)
    }

    fn with_capacity(
        vertex_capacity: usize,
        _index_capacity: usize,
    ) -> BufferData {
        let instance_arr: Vec<MapVertex> = iter::repeat(MapVertex::default())
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
