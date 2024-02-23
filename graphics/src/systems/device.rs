use crate::{AscendingError, GpuRenderer};
use async_trait::async_trait;
use std::{path::Path, sync::Arc};
use wgpu::TextureFormat;
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    window::Window,
};

///Handles the Device and Queue returned from WGPU.
pub struct GpuDevice {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl GpuDevice {
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
}

///Handles the Window, Adapter and Surface information.
pub struct GpuWindow {
    pub(crate) adapter: wgpu::Adapter,
    pub(crate) surface: wgpu::Surface<'static>,
    pub(crate) window: Arc<Window>,
    pub(crate) surface_format: wgpu::TextureFormat,
    pub(crate) size: PhysicalSize<f32>,
    pub(crate) inner_size: PhysicalSize<u32>,
    pub(crate) surface_config: wgpu::SurfaceConfiguration,
}

impl GpuWindow {
    pub fn adapter(&self) -> &wgpu::Adapter {
        &self.adapter
    }

    pub fn resize(
        &mut self,
        gpu_device: &GpuDevice,
        size: PhysicalSize<u32>,
    ) -> Result<(), AscendingError> {
        if size.width == 0 || size.height == 0 {
            return Ok(());
        }

        self.surface_config.height = size.height;
        self.surface_config.width = size.width;
        self.surface
            .configure(gpu_device.device(), &self.surface_config);
        self.size = PhysicalSize::new(size.width as f32, size.height as f32);

        Ok(())
    }

    pub fn size(&self) -> PhysicalSize<f32> {
        self.size
    }

    pub fn surface(&self) -> &wgpu::Surface {
        &self.surface
    }

    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.surface_format
    }

    pub fn update(
        &mut self,
        gpu_device: &GpuDevice,
        event: &Event<()>,
    ) -> Result<Option<wgpu::SurfaceTexture>, AscendingError> {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if *window_id == self.window.id() => match event {
                WindowEvent::Resized(physical_size) => {
                    self.resize(gpu_device, *physical_size)?;
                    self.inner_size = self.window.inner_size();

                    if self.size.width == 0.0
                        || self.size.height == 0.0
                        || self.inner_size.width == 0
                        || self.inner_size.height == 0
                    {
                        return Ok(None);
                    }

                    self.window.request_redraw();
                }
                WindowEvent::RedrawRequested => {
                    if self.size.width == 0.0
                        || self.size.height == 0.0
                        || self.inner_size.width == 0
                        || self.inner_size.height == 0
                    {
                        return Ok(None);
                    }

                    match self.surface.get_current_texture() {
                        Ok(frame) => {
                            self.window.request_redraw();
                            return Ok(Some(frame));
                        }
                        Err(wgpu::SurfaceError::Lost) => {
                            let size = PhysicalSize::new(
                                self.size.width as u32,
                                self.size.height as u32,
                            );
                            self.resize(gpu_device, size)?;
                            self.inner_size = self.window.inner_size();

                            if self.size.width == 0.0
                                || self.size.height == 0.0
                                || self.inner_size.width == 0
                                || self.inner_size.height == 0
                            {
                                return Ok(None);
                            }
                        }
                        Err(wgpu::SurfaceError::Outdated) => {
                            return Ok(None);
                        }
                        Err(e) => return Err(AscendingError::from(e)),
                    }

                    self.window.request_redraw();
                }
                WindowEvent::Moved(_)
                | WindowEvent::ScaleFactorChanged {
                    scale_factor: _,
                    inner_size_writer: _,
                }
                | WindowEvent::Focused(true)
                | WindowEvent::Occluded(false) => {
                    self.window.request_redraw();
                }
                _ => (),
            },
            _ => (),
        }

        Ok(None)
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn create_depth_texture(
        &self,
        gpu_device: &GpuDevice,
    ) -> wgpu::TextureView {
        let size = wgpu::Extent3d {
            width: self.size.width as u32,
            height: self.size.height as u32,
            depth_or_array_layers: 1,
        };

        let texture =
            gpu_device
                .device()
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some("depth texture"),
                    size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Depth32Float,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[TextureFormat::Depth32Float],
                });

        texture.create_view(&wgpu::TextureViewDescriptor::default())
    }
}

#[async_trait]
pub trait AdapterExt {
    async fn create_renderer(
        self,
        instance: &wgpu::Instance,
        window: Arc<Window>,
        device_descriptor: &wgpu::DeviceDescriptor,
        trace_path: Option<&Path>,
        present_mode: wgpu::PresentMode,
    ) -> Result<GpuRenderer, AscendingError>;
}

#[async_trait]
impl AdapterExt for wgpu::Adapter {
    async fn create_renderer(
        self,
        instance: &wgpu::Instance,
        window: Arc<Window>,
        device_descriptor: &wgpu::DeviceDescriptor,
        trace_path: Option<&Path>,
        present_mode: wgpu::PresentMode,
    ) -> Result<GpuRenderer, AscendingError> {
        let size = window.inner_size();

        let (device, queue) =
            self.request_device(device_descriptor, trace_path).await?;

        let surface = instance.create_surface(window.clone()).unwrap();
        let caps = surface.get_capabilities(&self);

        println!("{:?}", caps.formats);

        let rgba = caps
            .formats
            .iter()
            .position(|v| *v == TextureFormat::Rgba8UnormSrgb);
        let bgra = caps
            .formats
            .iter()
            .position(|v| *v == TextureFormat::Bgra8UnormSrgb);

        let format = if let Some(pos) = rgba {
            caps.formats[pos]
        } else if let Some(pos) = bgra {
            caps.formats[pos]
        } else {
            panic!("Your Rendering Device does not support Bgra8UnormSrgb or Rgba8UnormSrgb");
        };

        println!("surface format: {:?}", format);
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![format],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &surface_config);
        let inner_size = window.inner_size();
        let mut renderer = GpuRenderer::new(
            GpuWindow {
                adapter: self,
                surface,
                window,
                surface_format: format,
                size: PhysicalSize::new(size.width as f32, size.height as f32),
                surface_config,
                inner_size,
            },
            GpuDevice { device, queue },
        );

        // Creates the shader rendering pipelines for each renderer.
        renderer.create_pipelines(renderer.surface_format());
        Ok(renderer)
    }
}

#[async_trait]
pub trait InstanceExt {
    async fn create_device(
        &self,
        window: Arc<Window>,
        request_adapter_options: &wgpu::RequestAdapterOptions,
        device_descriptor: &wgpu::DeviceDescriptor,
        trace_path: Option<&Path>,
        present_mode: wgpu::PresentMode,
    ) -> Result<GpuRenderer, AscendingError>;
}

#[async_trait]
impl InstanceExt for wgpu::Instance {
    async fn create_device(
        &self,
        window: Arc<Window>,
        request_adapter_options: &wgpu::RequestAdapterOptions,
        device_descriptor: &wgpu::DeviceDescriptor,
        trace_path: Option<&Path>,
        present_mode: wgpu::PresentMode,
    ) -> Result<GpuRenderer, AscendingError> {
        let adapter =
            self.request_adapter(request_adapter_options).await.unwrap();
        adapter
            .create_renderer(
                self,
                window,
                device_descriptor,
                trace_path,
                present_mode,
            )
            .await
    }
}
