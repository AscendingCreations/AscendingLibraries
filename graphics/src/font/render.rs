use crate::{
    AsBufferPass, AtlasSet, GpuRenderer, GraphicsError, InstanceBuffer,
    OrderedIndex, SetBuffers, StaticVertexBuffer, Text, TextRenderPipeline,
    TextVertex, Vec2,
};
use cosmic_text::{CacheKey, SwashCache};
use log::{error, warn};

/// [`Text`] text and Emoji AtlasSet holder.
///
pub struct TextAtlas {
    /// AtlasSet holding data from Text only.
    pub(crate) text: AtlasSet<CacheKey, Vec2>,
    /// AtlasSet holding data from Colored Emoji's only.
    pub(crate) emoji: AtlasSet<CacheKey, Vec2>,
}

impl TextAtlas {
    /// Creates a new [`TextAtlas`].
    ///
    pub fn new(renderer: &mut GpuRenderer) -> Result<Self, GraphicsError> {
        Ok(Self {
            text: AtlasSet::new(renderer, wgpu::TextureFormat::R8Unorm, false),
            emoji: AtlasSet::new(
                renderer,
                wgpu::TextureFormat::Rgba8UnormSrgb,
                false,
            ),
        })
    }

    /// Calles Trim on both internal [`AtlasSet`]'s
    ///
    pub fn trim(&mut self) {
        self.emoji.trim();
        self.text.trim();
    }
}

/// Instance Buffer Setup for [`Text`].
///
pub struct TextRenderer {
    pub(crate) buffer: InstanceBuffer<TextVertex>,
    pub(crate) swash_cache: SwashCache,
}

impl TextRenderer {
    /// Creates a new [`TextRenderer`].
    ///
    pub fn new(renderer: &GpuRenderer) -> Result<Self, GraphicsError> {
        Ok(Self {
            buffer: InstanceBuffer::new(renderer.gpu_device(), 1024),
            swash_cache: SwashCache::new(),
        })
    }

    /// Adds a Buffer [`OrderedIndex`] to the Rendering Store to get processed.
    /// This must be done before [`TextRenderer::finalize`] but after [`Text::update`] in order for it to Render.
    ///
    /// # Arguments
    /// - index: The [`OrderedIndex`] of the Object we want to render.
    /// - layer: The Buffer Layer we want to add this Object too.
    ///
    pub fn add_buffer_store(
        &mut self,
        renderer: &GpuRenderer,
        index: OrderedIndex,
        layer: usize,
    ) {
        self.buffer.add_buffer_store(renderer, index, layer);
    }

    /// Finalizes the Buffer by processing staged [`OrderedIndex`]'s and uploading it to the GPU.
    /// Must be called after all the [`TextRenderer::add_buffer_store`]'s.
    ///
    pub fn finalize(&mut self, renderer: &mut GpuRenderer) {
        self.buffer.finalize(renderer)
    }

    /// Updates a [`Text`] and adds its [`OrderedIndex`] to staging using [`TextRenderer::add_buffer_store`].
    /// This must be done before [`TextRenderer::finalize`] in order for it to Render.
    ///
    /// # Arguments
    /// - text: [`Text`] we want to update and prepare for rendering.
    /// - atlas: [`TextAtlas`] the [`Text`] needs to render with.
    /// - layer: The Buffer Layer we want to add this Object too.
    ///
    pub fn text_update(
        &mut self,
        text: &mut Text,
        atlas: &mut TextAtlas,
        renderer: &mut GpuRenderer,
        layer: usize,
    ) -> Result<(), GraphicsError> {
        let index = text.update(&mut self.swash_cache, atlas, renderer)?;

        self.add_buffer_store(renderer, index, layer);
        Ok(())
    }

    /// [`Text`] does not use Scissor Clipping.
    /// It uses its own Internal Bounds Clipper.
    ///
    pub fn use_clipping(&mut self) {
        warn!("Text uses its own Clipping.");
    }
}

pub trait RenderText<'a, 'b>
where
    'b: 'a,
{
    fn render_text(
        &mut self,
        renderer: &'b GpuRenderer,
        buffer: &'b TextRenderer,
        atlas: &'b TextAtlas,
        layer: usize,
    );
}

impl<'a, 'b> RenderText<'a, 'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn render_text(
        &mut self,
        renderer: &'b GpuRenderer,
        buffer: &'b TextRenderer,
        atlas: &'b TextAtlas,
        layer: usize,
    ) {
        if buffer.buffer.is_clipped() {
            error!("Text uses its own clipping mechanisim it does not need to be clipped by the clipper.");
            return;
        }

        if let Some(Some(details)) = buffer.buffer.buffers.get(layer) {
            if buffer.buffer.count() > 0 {
                self.set_buffers(renderer.buffer_object.as_buffer_pass());
                self.set_bind_group(1, atlas.text.bind_group(), &[]);
                self.set_bind_group(2, atlas.emoji.bind_group(), &[]);
                self.set_vertex_buffer(1, buffer.buffer.instances(None));
                self.set_pipeline(
                    renderer.get_pipelines(TextRenderPipeline).unwrap(),
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
