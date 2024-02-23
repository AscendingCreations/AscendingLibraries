use crate::{
    AscendingError, AtlasSet, GpuRenderer, Image, ImageRenderPipeline,
    ImageVertex, InstanceBuffer, OrderedIndex, StaticBufferObject,
};

pub struct ImageRenderer {
    pub buffer: InstanceBuffer<ImageVertex>,
}

impl ImageRenderer {
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

    pub fn image_update(
        &mut self,
        image: &mut Image,
        renderer: &mut GpuRenderer,
        atlas: &mut AtlasSet,
        layer: usize,
    ) {
        let index = image.update(renderer, atlas);

        self.add_buffer_store(renderer, index, layer);
    }
}

pub trait RenderImage<'a, 'b>
where
    'b: 'a,
{
    fn render_image(
        &mut self,
        renderer: &'b GpuRenderer,
        buffer: &'b ImageRenderer,
        atlas: &'b AtlasSet,
        layer: usize,
    );
}

impl<'a, 'b> RenderImage<'a, 'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn render_image(
        &mut self,
        renderer: &'b GpuRenderer,
        buffer: &'b ImageRenderer,
        atlas: &'b AtlasSet,
        layer: usize,
    ) {
        if let Some(Some(details)) = buffer.buffer.buffers.get(layer) {
            if buffer.buffer.count() > 0 {
                self.set_bind_group(1, atlas.bind_group(), &[]);
                self.set_vertex_buffer(1, buffer.buffer.instances(None));
                self.set_pipeline(
                    renderer.get_pipelines(ImageRenderPipeline).unwrap(),
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
