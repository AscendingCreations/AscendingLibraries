use crate::{
    AsBufferPass, Buffer, BufferData, BufferLayout, BufferPass, GpuDevice,
    GpuRenderer, OrderedIndex,
};
use std::ops::Range;
//This Holds onto all the Vertexs Compressed into a byte array.
//This is Used for objects that need more advanced VBO/IBO other wise use the Instance buffers.

#[derive(Copy, Clone)]
pub struct IndexDetails {
    pub indices_start: u32,
    pub indices_end: u32,
    pub vertex_base: i32,
}

pub struct GpuBuffer<K: BufferLayout> {
    unprocessed: Vec<Vec<OrderedIndex>>,
    pub buffers: Vec<Vec<IndexDetails>>,
    pub vertex_buffer: Buffer<K>,
    vertex_needed: usize,
    pub index_buffer: Buffer<K>,
    index_needed: usize,
    pub layer_size: usize,
}

impl<'a, K: BufferLayout> AsBufferPass<'a> for GpuBuffer<K> {
    fn as_buffer_pass(&'a self) -> BufferPass<'a> {
        BufferPass {
            vertex_buffer: &self.vertex_buffer.buffer,
            index_buffer: &self.index_buffer.buffer,
        }
    }
}

impl<K: BufferLayout> GpuBuffer<K> {
    /// Used to create GpuBuffer from a (Vertex:Vec<u8>, Indices:Vec<u8>).
    pub fn create_buffer(
        gpu_device: &GpuDevice,
        buffers: &BufferData,
        layer_size: usize,
    ) -> Self {
        GpuBuffer {
            unprocessed: Vec::new(),
            buffers: Vec::new(),
            vertex_buffer: Buffer::new(
                gpu_device,
                &buffers.vertexs,
                wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                Some("Vertex Buffer"),
            ),
            vertex_needed: 0,
            index_buffer: Buffer::new(
                gpu_device,
                &buffers.indexs,
                wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                Some("Index Buffer"),
            ),
            index_needed: 0,
            layer_size: layer_size.max(32),
        }
    }

    pub fn add_buffer_store(
        &mut self,
        renderer: &GpuRenderer,
        mut index: OrderedIndex,
        layer: usize,
    ) {
        if let Some(store) = renderer.get_buffer(&index.index) {
            let offset = layer.saturating_add(1);
            // add in the missing layers this is better than keeping a hash since
            // if at anytime a process adds new data to a older layer it will already Exist.
            if self.unprocessed.len() < offset {
                for i in self.unprocessed.len()..offset {
                    //Push the layer buffer. if this is a layer we are adding data too lets
                    //give it a starting size. this cna be adjusted later for better performance
                    //versus ram usage.
                    self.unprocessed.push(if i == layer {
                        Vec::with_capacity(32)
                    } else {
                        Vec::new()
                    });
                }
            }

            self.vertex_needed += store.store.len();
            self.index_needed += store.indexs.len();

            index.index_count = store.indexs.len() as u32 / 4;

            if let Some(unprocessed) = self.unprocessed.get_mut(layer) {
                unprocessed.push(index);
            }
        }
    }

    pub fn finalize(&mut self, renderer: &mut GpuRenderer) {
        let (
            mut changed,
            mut vertex_pos,
            mut index_pos,
            mut pos,
            mut base_vertex,
            mut layer,
        ) = (false, 0, 0, 0, 0, 1);

        if self.vertex_needed > self.vertex_buffer.max
            || self.index_needed > self.index_buffer.max
        {
            self.resize(
                renderer.gpu_device(),
                self.vertex_needed / K::stride(),
                self.index_needed,
            );
            changed = true;
        }

        self.vertex_buffer.count = self.vertex_needed / K::stride();
        self.vertex_buffer.len = self.vertex_needed;

        //shouldnt need if renderer does all the sorting and layering first.
        for processing in &mut self.unprocessed {
            processing.sort();
        }

        if self.buffers.len() < self.unprocessed.len() {
            for _ in self.buffers.len()..self.unprocessed.len() {
                self.buffers.push(Vec::new());
            }
        }

        for buffer in &mut self.buffers {
            buffer.clear()
        }

        for processing in &self.unprocessed {
            for buf in processing {
                let mut write_vertex = false;
                let mut write_index = false;
                let old_vertex_pos = vertex_pos as u64;
                let old_index_pos = index_pos as u64;

                if let Some(store) = renderer.get_buffer_mut(&buf.index) {
                    let vertex_range =
                        vertex_pos..vertex_pos + store.store.len();
                    let index_range = index_pos..index_pos + store.indexs.len();

                    if store.store_pos != vertex_range
                        || changed
                        || store.changed
                    {
                        store.store_pos = vertex_range;
                        write_vertex = true
                    }

                    if store.index_pos != index_range
                        || changed
                        || store.changed
                    {
                        store.index_pos = index_range;
                        write_index = true
                    }

                    if write_index || write_vertex {
                        store.changed = false;
                    }

                    vertex_pos += store.store.len();
                    index_pos += store.indexs.len();
                }

                if write_vertex {
                    if let Some(store) = renderer.get_buffer(&buf.index) {
                        self.vertex_buffer.write(
                            &renderer.device,
                            &store.store,
                            old_vertex_pos,
                        );
                    }
                }

                if write_index {
                    if let Some(store) = renderer.get_buffer(&buf.index) {
                        self.index_buffer.write(
                            &renderer.device,
                            &store.indexs,
                            old_index_pos,
                        );
                    }
                }

                let indices_start = pos;
                let indices_end = pos + buf.index_count;
                let vertex_base = base_vertex;

                base_vertex += buf.index_max as i32 + 1;
                pos += buf.index_count;

                if let Some(buffer) = self.buffers.get_mut(layer - 1) {
                    buffer.push(IndexDetails {
                        indices_start,
                        indices_end,
                        vertex_base,
                    });
                }
            }

            layer += 1;
        }

        for buffer in &mut self.unprocessed {
            buffer.clear()
        }

        self.vertex_needed = 0;
        self.index_needed = 0;
    }

    //private but resizes the buffer on the GPU when needed.
    fn resize(
        &mut self,
        gpu_device: &GpuDevice,
        vertex_capacity: usize,
        index_capacity: usize,
    ) {
        let buffers = K::with_capacity(vertex_capacity, index_capacity);

        self.vertex_buffer = Buffer::new(
            gpu_device,
            &buffers.vertexs,
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            Some("Vertex Buffer"),
        );

        self.index_buffer = Buffer::new(
            gpu_device,
            &buffers.indexs,
            wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            Some("Index Buffer"),
        )
    }

    /// Returns the index_count.
    pub fn index_count(&self) -> usize {
        self.index_buffer.count
    }

    /// Returns the index maximum size.
    pub fn index_max(&self) -> usize {
        self.index_buffer.max
    }

    /// Returns wgpu::BufferSlice of indices.
    /// bounds is used to set a specific Range if needed.
    /// If bounds is None then range is 0..index_count.
    pub fn indices(&self, bounds: Option<Range<u64>>) -> wgpu::BufferSlice {
        let range = if let Some(bounds) = bounds {
            bounds
        } else {
            0..(self.index_buffer.count) as u64
        };

        self.index_buffer.buffer_slice(range)
    }

    /// creates a new pre initlized VertexBuffer with a default size.
    /// default size is based on the initial BufferPass::vertices length.
    pub fn new(device: &GpuDevice, layer_size: usize) -> Self {
        Self::create_buffer(device, &K::default_buffer(), layer_size)
    }

    /// Set the Index based on how many Vertex's Exist
    pub fn set_index_count(&mut self, count: usize) {
        self.index_buffer.count = count;
    }

    /// Returns the Vertex elements count.
    pub fn vertex_count(&self) -> usize {
        self.vertex_buffer.count
    }

    pub fn is_empty(&self) -> bool {
        self.vertex_buffer.count == 0
    }

    /// Returns vertex_buffer's max size in bytes.
    pub fn vertex_max(&self) -> usize {
        self.vertex_buffer.max
    }

    /// Returns vertex_buffer's vertex_stride.
    pub fn vertex_stride(&self) -> usize {
        K::stride()
    }

    /// Returns wgpu::BufferSlice of vertices.
    /// bounds is used to set a specific Range if needed.
    /// If bounds is None then range is 0..vertex_count.
    pub fn vertices(&self, bounds: Option<Range<u64>>) -> wgpu::BufferSlice {
        let range = if let Some(bounds) = bounds {
            bounds
        } else {
            0..self.vertex_buffer.count as u64
        };

        self.vertex_buffer.buffer_slice(range)
    }

    /// Creates a GpuBuffer based on capacity.
    /// Capacity is the amount of objects to initialize for.
    /// Capacity * 2 == the reserved space for the indices.
    pub fn with_capacity(
        gpu_device: &GpuDevice,
        capacity: usize,
        layer_size: usize,
    ) -> Self {
        Self::create_buffer(
            gpu_device,
            &K::with_capacity(capacity, capacity * 2),
            layer_size,
        )
    }
}
