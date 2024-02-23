use crate::{AsBufferPass, BufferPass, GpuDevice};
use bytemuck::{Pod, Zeroable};
use std::mem;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
pub struct Vertex {
    _position: [f32; 2],
}

const INDICES: [u32; 6] = [0, 1, 2, 0, 2, 3];

const VERTS: [Vertex; 4] = [
    Vertex {
        _position: [0.0, 0.0],
    },
    Vertex {
        _position: [1.0, 0.0],
    },
    Vertex {
        _position: [1.0, 1.0],
    },
    Vertex {
        _position: [0.0, 1.0],
    },
];

// This is to be used along with Instances.
// This its the reusable VBO and IBO per each instance.
// The GPU will point to the Instance per each Vertex object and then
// Move the index by one to the next instance.
// This only works for things that dont need custom VBO/IBO's
pub struct StaticBufferObject {
    pub vbo: wgpu::Buffer,
    pub ibo: wgpu::Buffer,
}

impl<'a> AsBufferPass<'a> for StaticBufferObject {
    fn as_buffer_pass(&'a self) -> BufferPass<'a> {
        BufferPass {
            vertex_buffer: &self.vbo,
            index_buffer: &self.ibo,
        }
    }
}

impl StaticBufferObject {
    /// Used to create vbo/ibo from the static context.
    pub fn create_buffer(gpu_device: &GpuDevice) -> Self {
        Self {
            vbo: gpu_device.device().create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("static vertex buffer"),
                    contents: bytemuck::cast_slice(&VERTS),
                    usage: wgpu::BufferUsages::VERTEX,
                },
            ),
            ibo: gpu_device.device().create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("static index buffer"),
                    contents: bytemuck::cast_slice(&INDICES),
                    usage: wgpu::BufferUsages::INDEX,
                },
            ),
        }
    }

    /// Returns the index_count.
    pub fn index_count() -> u32 {
        INDICES.len() as u32
    }

    //VertexBufferLayout for StaticBufferObject VBO.
    pub fn vertex_attribute() -> wgpu::VertexAttribute {
        wgpu::VertexAttribute {
            shader_location: 0,
            format: wgpu::VertexFormat::Float32x2,
            offset: 0,
        }
    }

    pub fn stride() -> u64 {
        mem::size_of::<Vertex>() as u64
    }

    /// Returns wgpu::BufferSlice of indices.
    pub fn indices(&self) -> wgpu::BufferSlice {
        self.ibo.slice(..)
    }

    /// creates a the static vbo/ibo.
    pub fn new(gpu_device: &GpuDevice) -> Self {
        Self::create_buffer(gpu_device)
    }

    /// Returns wgpu::BufferSlice of vertices.
    pub fn vertices(&self) -> wgpu::BufferSlice {
        self.vbo.slice(..)
    }
}
