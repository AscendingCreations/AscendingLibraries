use crate::{
    AsBufferPass, AtlasSet, GpuRenderer, GraphicsError, InstanceBuffer, Map,
    MapRenderPipeline, MapVertex, OrderedIndex, SetBuffers, StaticVertexBuffer,
};
use log::warn;

/// Instance Buffer Setup for [`Map`]'s.
///
pub struct MapRenderer {
    /// Instance Buffer holding all Rendering information for [`Map`]'s.
    pub buffer: InstanceBuffer<MapVertex>,
}

impl MapRenderer {
    /// Creates a new [`MapRenderer`].
    ///
    /// # Arguments
    /// - map_count: The number of Maps to presize the instance buffer by.
    ///
    pub fn new(
        renderer: &mut GpuRenderer,
        map_count: u32,
    ) -> Result<Self, GraphicsError> {
        Ok(Self {
            buffer: InstanceBuffer::with_capacity(
                renderer.gpu_device(),
                9_216 * map_count as usize,
                144,
            ),
        })
    }

    /// Adds a Buffer [`OrderedIndex`] to the Rendering Store to get processed.
    /// This must be done before [`MapRenderer::finalize`] but after [`Map::update`] in order for it to Render.
    ///
    /// # Arguments
    /// - index: The [`OrderedIndex`] of the Object we want to render.
    /// - buffer_layer: The Buffer Layer we want to add this Object too.
    ///
    pub fn add_buffer_store(
        &mut self,
        renderer: &GpuRenderer,
        index: OrderedIndex,
        buffer_layer: usize,
    ) {
        self.buffer.add_buffer_store(renderer, index, buffer_layer);
    }

    /// Finalizes the Buffer by processing staged [`OrderedIndex`]'s and uploading it to the GPU.
    /// Must be called after all the [`MapRenderer::add_buffer_store`]'s.
    ///
    pub fn finalize(&mut self, renderer: &mut GpuRenderer) {
        self.buffer.finalize(renderer);
    }

    /// Updates a [`Map`] and adds its [`OrderedIndex`]'s to staging using [`MapRenderer::add_buffer_store`].
    /// This must be done before [`MapRenderer::finalize`] in order for it to Render.
    ///
    /// # Arguments
    /// - map: [`Map`] we want to update and prepare for rendering.
    /// - atlas: [`AtlasSet`] the [`Map`] needs to render with.
    /// - buffer_layers: The Buffer Layer's we want to add this Object too.
    ///
    pub fn map_update(
        &mut self,
        map: &mut Map,
        renderer: &mut GpuRenderer,
        atlas: &mut AtlasSet,
        buffer_layers: [usize; 2],
    ) {
        if let Some(indexs) = map.update(renderer, atlas) {
            for (id, order_index) in indexs.into_iter().enumerate() {
                self.add_buffer_store(renderer, order_index, buffer_layers[id]);
            }
        }
    }

    /// Map does not use Clipping.
    pub fn use_clipping(&mut self) {
        warn!("Map does not use Clipping.");
    }
}

/// Trait used to Grant Direct [`Map`] Rendering to [`wgpu::RenderPass`]
pub trait RenderMap<'a, 'b>
where
    'b: 'a,
{
    /// Renders the all [`Map`]'s within the buffer layer to screen that have been processed and finalized.
    ///
    fn render_map(
        &mut self,
        renderer: &'b GpuRenderer,
        buffer: &'b MapRenderer,
        atlas: &'b AtlasSet,
        buffer_layer: usize,
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
        buffer_layer: usize,
    ) {
        if let Some(Some(details)) = buffer.buffer.buffers.get(buffer_layer) {
            if buffer.buffer.count() > 0 {
                self.set_buffers(renderer.buffer_object.as_buffer_pass());
                self.set_bind_group(1, atlas.bind_group(), &[]);
                self.set_vertex_buffer(1, buffer.buffer.instances(None));
                self.set_pipeline(
                    renderer.get_pipelines(MapRenderPipeline).unwrap(),
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
