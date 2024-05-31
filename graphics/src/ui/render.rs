use crate::{
    AtlasSet, GpuRenderer, GraphicsError, InstanceBuffer, OrderedIndex, Rect,
    RectRenderPipeline, RectVertex, StaticVertexBuffer, System,
};

pub struct RectRenderer {
    pub buffer: InstanceBuffer<RectVertex>,
}

impl RectRenderer {
    pub fn new(renderer: &GpuRenderer) -> Result<Self, GraphicsError> {
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

    pub fn use_clipping(&mut self) {
        self.buffer.set_as_clipped();
    }
}

pub trait RenderRects<'a, 'b, Controls>
where
    'b: 'a,
    Controls: camera::controls::Controls,
{
    fn render_rects(
        &mut self,
        renderer: &'b GpuRenderer,
        buffer: &'b RectRenderer,
        atlas: &'b AtlasSet,
        system: &'b System<Controls>,
        layer: usize,
    );
}

impl<'a, 'b, Controls> RenderRects<'a, 'b, Controls> for wgpu::RenderPass<'a>
where
    'b: 'a,
    Controls: camera::controls::Controls,
{
    fn render_rects(
        &mut self,
        renderer: &'b GpuRenderer,
        buffer: &'b RectRenderer,
        atlas: &'b AtlasSet,
        system: &'b System<Controls>,
        layer: usize,
    ) {
        if buffer.buffer.is_clipped() {
            if let Some(details) = buffer.buffer.clipped_buffers.get(layer) {
                let mut scissor_is_default = true;

                if buffer.buffer.count() > 0 {
                    self.set_bind_group(
                        1,
                        &atlas.texture_group.bind_group,
                        &[],
                    );
                    self.set_vertex_buffer(1, buffer.buffer.instances(None));
                    self.set_pipeline(
                        renderer.get_pipelines(RectRenderPipeline).unwrap(),
                    );

                    for (details, bounds, camera_type) in details {
                        if let Some(bounds) = bounds {
                            let bounds =
                                system.world_to_screen(*camera_type, bounds);

                            self.set_scissor_rect(
                                bounds.x as u32,
                                bounds.y as u32,
                                bounds.z as u32,
                                bounds.w as u32,
                            );
                            scissor_is_default = false;
                        }

                        self.draw_indexed(
                            0..StaticVertexBuffer::index_count(),
                            0,
                            details.start..details.end,
                        );

                        if !scissor_is_default {
                            self.set_scissor_rect(
                                0,
                                0,
                                system.screen_size[0] as u32,
                                system.screen_size[1] as u32,
                            );
                            scissor_is_default = true;
                        };
                    }
                }
            }
        } else if let Some(Some(details)) = buffer.buffer.buffers.get(layer) {
            if buffer.buffer.count() > 0 {
                self.set_bind_group(1, &atlas.texture_group.bind_group, &[]);
                self.set_vertex_buffer(1, buffer.buffer.instances(None));
                self.set_pipeline(
                    renderer.get_pipelines(RectRenderPipeline).unwrap(),
                );

                self.draw_indexed(
                    0..StaticVertexBuffer::index_count(),
                    0,
                    details.start..details.end,
                );
            }
        }
    }
}
