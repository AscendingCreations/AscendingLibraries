use crate::{BufferData, BufferLayout};
#[cfg(feature = "rayon")]
use rayon::{iter::repeatn, prelude::*};

/// Vertex Details for [`crate::Map`] that matches the Shaders Vertex Layout.
///
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MapVertex {
    pub pos: [f32; 3],
    pub tilesize: f32,
    pub tile_id: u32,
    pub texture_layer: u32,
    pub color: u32,
    pub camera_type: u32,
}

impl Default for MapVertex {
    fn default() -> Self {
        Self {
            pos: [0.0; 3],
            tilesize: 0.0,
            tile_id: 0,
            texture_layer: 0,
            color: 0,
            camera_type: 0,
        }
    }
}

impl BufferLayout for MapVertex {
    fn attributes() -> Vec<wgpu::VertexAttribute> {
        wgpu::vertex_attr_array![1 => Float32x3, 2 => Float32, 3 => Uint32, 4 => Uint32, 5 => Uint32, 6 => Uint32]
            .to_vec()
    }

    fn default_buffer() -> BufferData {
        Self::with_capacity(9_216, 0)
    }

    fn with_capacity(
        vertex_capacity: usize,
        _index_capacity: usize,
    ) -> BufferData {
        #[cfg(feature = "rayon")]
        let instance_arr: Vec<MapVertex> =
            repeatn(MapVertex::default(), vertex_capacity).collect();

        #[cfg(not(feature = "rayon"))]
        let instance_arr: Vec<MapVertex> =
            std::iter::repeat_n(MapVertex::default(), vertex_capacity)
                .collect();

        BufferData {
            vertexs: bytemuck::cast_slice(&instance_arr).to_vec(),
            ..Default::default()
        }
    }

    fn stride() -> usize {
        std::mem::size_of::<[f32; 8]>()
    }
}
