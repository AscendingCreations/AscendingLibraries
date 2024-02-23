use crate::{
    AscendingError, BufferPass, BufferStore, GpuDevice, GpuWindow, Index,
    Layout, LayoutStorage, OtherError, PipeLineLayout, PipelineStorage,
    StaticBufferObject,
};
use cosmic_text::FontSystem;
use generational_array::{
    GenerationalArray, GenerationalArrayResult, GenerationalArrayResultMut,
};
use std::rc::Rc;

use winit::{dpi::PhysicalSize, event::Event, window::Window};

///Handles the Window, Device and buffer stores.
pub struct GpuRenderer {
    pub(crate) window: GpuWindow,
    pub(crate) device: GpuDevice,
    pub(crate) buffer_stores: GenerationalArray<BufferStore>,
    pub(crate) layout_storage: LayoutStorage,
    pub(crate) pipeline_storage: PipelineStorage,
    pub(crate) depthbuffer: wgpu::TextureView,
    pub(crate) framebuffer: Option<wgpu::TextureView>,
    pub(crate) frame: Option<wgpu::SurfaceTexture>,
    pub font_sys: FontSystem,
    pub buffer_object: StaticBufferObject,
}

pub trait SetBuffers<'a, 'b>
where
    'b: 'a,
{
    fn set_buffers(&mut self, buffer: BufferPass<'b>);
}

impl<'a, 'b> SetBuffers<'a, 'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn set_buffers(&mut self, buffer: BufferPass<'b>) {
        self.set_vertex_buffer(0, buffer.vertex_buffer.slice(..));
        self.set_index_buffer(
            buffer.index_buffer.slice(..),
            wgpu::IndexFormat::Uint32,
        );
    }
}

impl GpuRenderer {
    pub fn new(window: GpuWindow, device: GpuDevice) -> Self {
        let buffer_object = StaticBufferObject::create_buffer(&device);
        let depth_buffer = window.create_depth_texture(&device);

        Self {
            window,
            device,
            buffer_stores: GenerationalArray::new(),
            layout_storage: LayoutStorage::new(),
            pipeline_storage: PipelineStorage::new(),
            depthbuffer: depth_buffer,
            framebuffer: None,
            frame: None,
            font_sys: FontSystem::new(),
            buffer_object,
        }
    }

    pub fn adapter(&self) -> &wgpu::Adapter {
        self.window.adapter()
    }

    pub fn resize(
        &mut self,
        size: PhysicalSize<u32>,
    ) -> Result<(), AscendingError> {
        self.window.resize(&self.device, size)
    }

    pub fn frame_buffer(&self) -> &Option<wgpu::TextureView> {
        &self.framebuffer
    }

    pub fn depth_buffer(&self) -> &wgpu::TextureView {
        &self.depthbuffer
    }

    pub fn size(&self) -> PhysicalSize<f32> {
        self.window.size
    }

    pub fn inner_size(&self) -> PhysicalSize<u32> {
        self.window.inner_size
    }

    pub fn surface(&self) -> &wgpu::Surface {
        &self.window.surface
    }

    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.window.surface_format
    }

    pub fn update(
        &mut self,
        event: &Event<()>,
    ) -> Result<bool, AscendingError> {
        let frame = match self.window.update(&self.device, event)? {
            Some(frame) => frame,
            _ => return Ok(false),
        };

        self.framebuffer = Some(
            frame
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default()),
        );
        self.frame = Some(frame);

        Ok(true)
    }

    pub fn window(&self) -> &Window {
        &self.window.window
    }

    pub fn update_depth_texture(&mut self) {
        self.depthbuffer = self.window.create_depth_texture(&self.device);
    }

    pub fn present(&mut self) -> Result<(), AscendingError> {
        self.framebuffer = None;

        match self.frame.take() {
            Some(frame) => {
                frame.present();
                Ok(())
            }
            None => Err(AscendingError::Other(OtherError::new(
                "Frame does not Exist. Did you forget to update the renderer?",
            ))),
        }
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device.device
    }

    pub fn gpu_device(&self) -> &GpuDevice {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.device.queue
    }

    pub fn font_sys(&self) -> &FontSystem {
        &self.font_sys
    }

    pub fn font_sys_mut(&mut self) -> &mut FontSystem {
        &mut self.font_sys
    }

    pub fn new_buffer(&mut self) -> Index {
        self.buffer_stores.insert(BufferStore::default())
    }

    pub fn remove_buffer(&mut self, index: Index) {
        let _ = self.buffer_stores.remove(index);
    }

    pub fn get_buffer(&self, index: &Index) -> Option<&BufferStore> {
        match self.buffer_stores.get(index) {
            GenerationalArrayResult::None => None,
            GenerationalArrayResult::OutDated => None,
            GenerationalArrayResult::OutOfBounds => None,
            GenerationalArrayResult::Some(v) => Some(v),
        }
    }

    pub fn get_buffer_mut(
        &mut self,
        index: &Index,
    ) -> Option<&mut BufferStore> {
        match self.buffer_stores.get_mut(index) {
            GenerationalArrayResultMut::None => None,
            GenerationalArrayResultMut::OutDated => None,
            GenerationalArrayResultMut::OutOfBounds => None,
            GenerationalArrayResultMut::Some(v) => Some(v),
        }
    }

    pub fn create_layout<K: Layout>(
        &mut self,
        layout: K,
    ) -> Rc<wgpu::BindGroupLayout> {
        self.layout_storage.create_layout(&mut self.device, layout)
    }

    pub fn create_pipelines(&mut self, surface_format: wgpu::TextureFormat) {
        self.pipeline_storage.create_pipeline(
            &mut self.device,
            &mut self.layout_storage,
            surface_format,
            crate::ImageRenderPipeline,
        );

        self.pipeline_storage.create_pipeline(
            &mut self.device,
            &mut self.layout_storage,
            surface_format,
            crate::MapRenderPipeline,
        );

        self.pipeline_storage.create_pipeline(
            &mut self.device,
            &mut self.layout_storage,
            surface_format,
            crate::TextRenderPipeline,
        );

        self.pipeline_storage.create_pipeline(
            &mut self.device,
            &mut self.layout_storage,
            surface_format,
            crate::Mesh2DRenderPipeline,
        );

        self.pipeline_storage.create_pipeline(
            &mut self.device,
            &mut self.layout_storage,
            surface_format,
            crate::LightRenderPipeline,
        );

        self.pipeline_storage.create_pipeline(
            &mut self.device,
            &mut self.layout_storage,
            surface_format,
            crate::RectRenderPipeline,
        );
    }

    pub fn get_pipelines<K: PipeLineLayout>(
        &self,
        pipeline: K,
    ) -> Option<&wgpu::RenderPipeline> {
        self.pipeline_storage.get_pipeline(pipeline)
    }
}
