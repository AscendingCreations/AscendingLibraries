use crate::{
    AscendingError, AtlasSet, GpuRenderer, InstanceBuffer, OrderedIndex, Rect,
    RectRenderPipeline, RectVertex, StaticBufferObject,
};

pub struct RectRenderer {
    pub buffer: InstanceBuffer<RectVertex>,
}

impl RectRenderer {
    pub fn new(renderer: &GpuRenderer) -> Result<Self, AscendingError> {
        Ok(Self {
            buffer: InstanceBuffer::new(renderer.gpu_device(), 512),
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
        self.buffer.finalize(renderer)
    }

    pub fn rect_update(
        &mut self,
        rect: &mut Rect,
        renderer: &mut GpuRenderer,
        atlas: &mut AtlasSet,
        layer: usize,
    ) {
        let index = rect.update(renderer, atlas);

        self.add_buffer_store(renderer, index, layer);
    }
}

pub trait RenderRects<'a, 'b>
where
    'b: 'a,
{
    fn render_rects(
        &mut self,
        renderer: &'b GpuRenderer,
        buffer: &'b RectRenderer,
        atlas: &'b AtlasSet,
        layer: usize,
    );
}

impl<'a, 'b> RenderRects<'a, 'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn render_rects(
        &mut self,
        renderer: &'b GpuRenderer,
        buffer: &'b RectRenderer,
        atlas: &'b AtlasSet,
        layer: usize,
    ) {
        if let Some(Some(details)) = buffer.buffer.buffers.get(layer) {
            if buffer.buffer.count() > 0 {
                self.set_bind_group(1, &atlas.texture_group.bind_group, &[]);
                self.set_vertex_buffer(1, buffer.buffer.instances(None));
                self.set_pipeline(
                    renderer.get_pipelines(RectRenderPipeline).unwrap(),
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
