use crate::{
    AsBufferPass, AscendingError, AtlasSet, GpuRenderer, InstanceBuffer, Map,
    MapRenderPipeline, MapVertex, OrderedIndex, SetBuffers, StaticBufferObject,
};

pub struct MapRenderer {
    pub buffer: InstanceBuffer<MapVertex>,
}

impl MapRenderer {
    pub fn new(
        renderer: &mut GpuRenderer,
        map_count: u32,
    ) -> Result<Self, AscendingError> {
        Ok(Self {
            buffer: InstanceBuffer::with_capacity(
                renderer.gpu_device(),
                8_192 * map_count as usize,
                128,
            ),
        })
    }

    pub fn add_buffer_store(
        &mut self,
        renderer: &GpuRenderer,
        index: OrderedIndex,
        layer: usize,
    ) {
        self.buffer.add_buffer_store(renderer, index, layer);
    }

    pub fn finalize(&mut self, renderer: &mut GpuRenderer) {
        self.buffer.finalize(renderer);
    }

    pub fn map_update(
        &mut self,
        map: &mut Map,
        renderer: &mut GpuRenderer,
        atlas: &mut AtlasSet,
        layers: [usize; 2],
    ) {
        if let Some(indexs) = map.update(renderer, atlas) {
            for (id, order_index) in indexs.into_iter().enumerate() {
                self.add_buffer_store(renderer, order_index, layers[id]);
            }
        }
    }
}

pub trait RenderMap<'a, 'b>
where
    'b: 'a,
{
    fn render_map(
        &mut self,
        renderer: &'b GpuRenderer,
        buffer: &'b MapRenderer,
        atlas: &'b AtlasSet,
        layer: usize,
    );
}

impl<'a, 'b> RenderMap<'a, 'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn render_map(
        &mut self,
        renderer: &'b GpuRenderer,
        buffer: &'b MapRenderer,
        atlas: &'b AtlasSet,
        layer: usize,
    ) {
        if let Some(Some(details)) = buffer.buffer.buffers.get(layer) {
            if buffer.buffer.count() > 0 {
                self.set_buffers(renderer.buffer_object.as_buffer_pass());
                self.set_bind_group(1, atlas.bind_group(), &[]);
                self.set_vertex_buffer(1, buffer.buffer.instances(None));
                self.set_pipeline(
                    renderer.get_pipelines(MapRenderPipeline).unwrap(),
                );
                self.draw_indexed(
                    0..StaticBufferObject::index_count(),
                    0,
                    details.start..details.end,
                );
            }
        }
    }
}
