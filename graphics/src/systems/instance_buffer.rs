use crate::{
    Bounds, Buffer, BufferLayout, CameraType, GpuDevice, GpuRenderer,
    OrderedIndex,
};
use std::ops::Range;

pub struct InstanceDetails {
    pub start: u32,
    pub end: u32,
}

pub type ClippedInstanceDetails = (InstanceDetails, Option<Bounds>, CameraType);

//This Holds onto all the instances Compressed into a byte array.
pub struct InstanceBuffer<K: BufferLayout> {
    pub unprocessed: Vec<Vec<OrderedIndex>>,
    pub buffers: Vec<Option<InstanceDetails>>,
    pub clipped_buffers: Vec<Vec<ClippedInstanceDetails>>,
    pub buffer: Buffer<K>,
    pub layer_size: usize,
    // this is a calculation of the buffers size when being marked as ready to add into the buffer.
    needed_size: usize,
    is_clipped: bool,
}

impl<K: BufferLayout> InstanceBuffer<K> {
    /// Used to create GpuBuffer from a BufferPass.
    /// Only use this for creating a reusable buffer.
    pub fn create_buffer(
        gpu_device: &GpuDevice,
        data: &[u8],
        layer_size: usize,
    ) -> Self {
        InstanceBuffer {
            unprocessed: Vec::new(),
            buffers: Vec::new(),
            clipped_buffers: Vec::new(),
            buffer: Buffer::new(
                gpu_device,
                data,
                wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                Some("Instance Buffer"),
            ),
            layer_size: layer_size.max(32),
            needed_size: 0,
            is_clipped: false,
        }
    }

    pub fn add_buffer_store(
        &mut self,
        renderer: &GpuRenderer,
        index: OrderedIndex,
        layer: usize,
    ) {
        if let Some(store) = renderer.get_buffer(index.index) {
            let offset = layer.saturating_add(1);

            if self.unprocessed.len() < offset {
                for i in self.unprocessed.len()..offset {
                    //Push the layer buffer. if this is a layer we are adding data too lets
                    //give it a starting size. this cna be adjusted later for better performance
                    //versus ram usage.
                    self.unprocessed.push(if i == layer {
                        Vec::with_capacity(self.layer_size)
                    } else {
                        Vec::new()
                    });
                }
            }

            self.needed_size += store.store.len();

            if let Some(unprocessed) = self.unprocessed.get_mut(layer) {
                unprocessed.push(index);
            }
        }
    }

    fn buffer_write(
        &self,
        renderer: &mut GpuRenderer,
        buf: &OrderedIndex,
        pos: &mut usize,
        count: &mut u32,
        changed: bool,
    ) {
        let mut write_buffer = false;
        let old_pos = *pos as u64;

        if let Some(store) = renderer.get_buffer_mut(buf.index) {
            let range = *pos..*pos + store.store.len();

            if store.store_pos != range || changed || store.changed {
                store.store_pos = range;
                store.changed = false;
                write_buffer = true
            }

            *pos += store.store.len();
            *count += (store.store.len() / K::stride()) as u32;
        }

        if write_buffer {
            if let Some(store) = renderer.get_buffer(buf.index) {
                self.buffer.write(&renderer.device, &store.store, old_pos);
            }
        }
    }

    pub fn finalize(&mut self, renderer: &mut GpuRenderer) {
        let (mut changed, mut pos, mut count) = (false, 0, 0);

        if self.needed_size > self.buffer.max {
            self.resize(renderer.gpu_device(), self.needed_size / K::stride());
            changed = true;
        }

        self.buffer.count = self.needed_size / K::stride();
        self.buffer.len = self.needed_size;

        for processing in &mut self.unprocessed {
            processing.sort();
        }

        if self.is_clipped {
            for buffer in &mut self.clipped_buffers {
                buffer.clear();
            }

            if self.clipped_buffers.len() < self.unprocessed.len() {
                for _ in self.clipped_buffers.len()..self.unprocessed.len() {
                    self.clipped_buffers.push(Vec::new());
                }
            }
        } else {
            self.buffers.clear();
        }

        for (layer, processing) in self.unprocessed.iter().enumerate() {
            if processing.is_empty() {
                if !self.is_clipped {
                    self.buffers.push(None);
                }
                continue;
            }

            let mut start_pos = count;

            if !self.is_clipped {
                for buf in processing {
                    self.buffer_write(
                        renderer, buf, &mut pos, &mut count, changed,
                    );
                }

                self.buffers.push(Some(InstanceDetails {
                    start: start_pos,
                    end: count,
                }));
            } else {
                for buf in processing {
                    self.buffer_write(
                        renderer, buf, &mut pos, &mut count, changed,
                    );

                    if let Some(buffer) = self.clipped_buffers.get_mut(layer) {
                        buffer.push((
                            InstanceDetails {
                                start: start_pos,
                                end: count,
                            },
                            buf.bounds,
                            buf.camera_type,
                        ));
                    }

                    start_pos = count;
                }
            }
        }

        self.needed_size = 0;

        for buffer in &mut self.unprocessed {
            buffer.clear()
        }
    }

    //private but resizes the buffer on the GPU when needed.
    fn resize(&mut self, gpu_device: &GpuDevice, capacity: usize) {
        let data = K::with_capacity(capacity, 0);

        self.buffer = Buffer::new(
            gpu_device,
            &data.vertexs,
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            Some("Vertex Buffer"),
        );
    }

    /// creates a new pre initlized InstanceBuffer with a default size.
    /// default size is based on the initial InstanceLayout::default_buffer length.
    pub fn new(gpu_device: &GpuDevice, layer_size: usize) -> Self {
        Self::create_buffer(
            gpu_device,
            &K::default_buffer().vertexs,
            layer_size,
        )
    }

    /// Returns the elements count.
    pub fn count(&self) -> u32 {
        self.buffer.count as u32
    }

    /// Returns the elements byte count.
    pub fn len(&self) -> u64 {
        self.buffer.len as u64
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Returns vertex_buffer's max size in bytes.
    pub fn max(&self) -> usize {
        self.buffer.max
    }

    /// Returns if the buffer is clipped or not to deturmine if you should use
    /// buffers or clipped_buffers.
    pub fn is_clipped(&self) -> bool {
        self.is_clipped
    }

    /// Sets the Buffer into Clipping mode.
    /// This will Produce a clipped_buffers instead of the buffers which
    /// will still be layered but a Vector of individual objects will Exist rather
    /// than a grouped object per layer. Will make it less Efficient but allows Bounds Clipping.
    pub fn set_as_clipped(&mut self) {
        self.is_clipped = true;
    }

    /// Returns buffer's stride.
    pub fn stride(&self) -> usize {
        K::stride()
    }

    /// Returns wgpu::BufferSlice of vertices.
    /// bounds is used to set a specific Range if needed.
    /// If bounds is None then range is 0..vertex_count.
    pub fn instances(&self, bounds: Option<Range<u64>>) -> wgpu::BufferSlice {
        let range = if let Some(bounds) = bounds {
            bounds
        } else {
            0..self.len()
        };

        self.buffer.buffer_slice(range)
    }

    /// Creates a Buffer based on capacity.
    /// Capacity is the amount of objects to initialize for.
    pub fn with_capacity(
        gpu_device: &GpuDevice,
        capacity: usize,
        layer_size: usize,
    ) -> Self {
        Self::create_buffer(
            gpu_device,
            &K::with_capacity(capacity, 0).vertexs,
            layer_size,
        )
    }
}
