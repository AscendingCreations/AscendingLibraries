use crate::{GpuRenderer, GraphicsError};
use async_trait::async_trait;
use log::debug;
use std::{path::Path, sync::Arc};
use wgpu::{
    core::instance::RequestAdapterError, Adapter, Backend, Backends,
    DeviceType, Surface, TextureFormat,
};
use winit::{dpi::PhysicalSize, event::WindowEvent, window::Window};

/// Handles the [`wgpu::Device`] and [`wgpu::Queue`] returned from WGPU.
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

/// Our own Adapter Power Settings.
/// Not to be confused with [`wgpu::PowerPreference`].
///
#[derive(Default, Clone, Debug, PartialEq, Eq, Hash)]
pub enum AdapterPowerSettings {
    /// Target Lower Powered GPU's First.
    LowPower,
    /// Target High Powers GPU's First.
    #[default]
    HighPower,
}

/// Options used to Find and load a GPU Adapter to render from.
///
#[derive(Debug)]
pub struct AdapterOptions {
    /// Power preference for the adapter.
    pub allowed_backends: Backends,
    pub power: AdapterPowerSettings,
    /// Surface that is required to be presentable with the requested adapter. This does not
    /// create the surface, only guarantees that the adapter can present to said surface.
    /// Recommend checking this always.
    pub compatible_surface: Option<Surface<'static>>,
}

/// Handles the [`wgpu::Adapter`], [`wgpu::Surface`], [`Window`].
/// Also used to Keep track of [`wgpu::TextureFormat`], [`wgpu::SurfaceConfiguration`]
/// and [`Window`] Sizes.
///
pub struct GpuWindow {
    /// GPU Adapter we will render from.
    pub(crate) adapter: wgpu::Adapter,
    /// Window Surface we will Render Too.
    pub(crate) surface: wgpu::Surface<'static>,
    /// Window we are using to Render Too.
    pub(crate) window: Arc<Window>,
    /// Current Allowed surface_format of the GPU and Window.
    pub(crate) surface_format: wgpu::TextureFormat,
    /// Windows Overall Size.
    pub(crate) size: PhysicalSize<f32>,
    /// Windows Inner Size.
    pub(crate) inner_size: PhysicalSize<u32>,
    /// Currently Accepted Surface configurations.
    pub(crate) surface_config: wgpu::SurfaceConfiguration,
}

impl GpuWindow {
    /// Returns a reference to a [`wgpu::Adapter`].
    ///
    pub fn adapter(&self) -> &wgpu::Adapter {
        &self.adapter
    }

    /// Resizes the [`wgpu::Surface`].
    ///
    pub fn resize(
        &mut self,
        gpu_device: &GpuDevice,
        size: PhysicalSize<u32>,
    ) -> Result<(), GraphicsError> {
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

    /// Returns the Size of the [`wgpu::Surface`].
    ///
    pub fn size(&self) -> PhysicalSize<f32> {
        self.size
    }

    /// Returns Reference to the [`wgpu::Surface`].
    ///
    pub fn surface(&self) -> &wgpu::Surface {
        &self.surface
    }

    /// Returns the [`wgpu::TextureFormat`].
    ///
    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.surface_format
    }

    /// Resizes the [`wgpu::Surface`] and/or requests a redraw event for the Window.
    ///
    pub fn update(
        &mut self,
        gpu_device: &GpuDevice,
        event: &WindowEvent,
    ) -> Result<Option<wgpu::SurfaceTexture>, GraphicsError> {
        match event {
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
                    Err(e) => return Err(GraphicsError::from(e)),
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
        }

        Ok(None)
    }

    /// Returns a Reference to [`Window`].
    ///
    pub fn window(&self) -> &Window {
        &self.window
    }

    /// Creates a Depth Texture from the [`GpuDevice`]
    /// Using the size of the current [`wgpu::Surface`].
    ///
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

/// Trait used to Allow the [`wgpu::Adapter`] to Create a [`GpuRenderer`].
///
#[async_trait]
pub trait AdapterExt {
    /// Creates a [`GpuRenderer`].
    ///
    async fn create_renderer(
        self,
        instance: &wgpu::Instance,
        window: &Arc<Window>,
        device_descriptor: &wgpu::DeviceDescriptor,
        trace_path: Option<&Path>,
        present_mode: wgpu::PresentMode,
    ) -> Result<GpuRenderer, GraphicsError>;
}

#[async_trait]
impl AdapterExt for wgpu::Adapter {
    async fn create_renderer(
        self,
        instance: &wgpu::Instance,
        window: &Arc<Window>,
        device_descriptor: &wgpu::DeviceDescriptor,
        trace_path: Option<&Path>,
        present_mode: wgpu::PresentMode,
    ) -> Result<GpuRenderer, GraphicsError> {
        let size = window.inner_size();

        let (device, queue) =
            self.request_device(device_descriptor, trace_path).await?;

        let surface = instance.create_surface(window.clone()).unwrap();
        let caps = surface.get_capabilities(&self);

        debug!("{:?}", caps.formats);

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

        debug!("surface format: {:?}", format);
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
                window: window.clone(),
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

/// Trait used to Allow the [`wgpu::Instance`] to Create a [`GpuRenderer`].
/// And get Adapters.
///
#[async_trait]
pub trait InstanceExt {
    /// Creates a [`GpuRenderer`].
    ///
    async fn create_device(
        &self,
        window: Arc<Window>,
        options: AdapterOptions,
        device_descriptor: &wgpu::DeviceDescriptor,
        trace_path: Option<&Path>,
        present_mode: wgpu::PresentMode,
    ) -> Result<GpuRenderer, GraphicsError>;

    /// Gets a list of Avaliable Adapters based upon the [`AdapterOptions`].
    ///
    fn get_adapters(&self, options: AdapterOptions) -> Vec<(Adapter, u32)>;
}

#[async_trait]
impl InstanceExt for wgpu::Instance {
    fn get_adapters(&self, options: AdapterOptions) -> Vec<(Adapter, u32)> {
        let mut adapters = self.enumerate_adapters(options.allowed_backends);
        let mut compatible_adapters: Vec<(Adapter, u32)> = Vec::new();

        while let Some(adapter) = adapters.pop() {
            let information = adapter.get_info();

            if information.backend == Backend::Empty {
                continue;
            }

            let is_low = options.power == AdapterPowerSettings::LowPower;
            let device_type = match information.device_type {
                DeviceType::IntegratedGpu if is_low => 1,
                DeviceType::IntegratedGpu => 2,
                DeviceType::DiscreteGpu if is_low => 2,
                DeviceType::DiscreteGpu => 1,
                DeviceType::Other => 3,
                DeviceType::VirtualGpu => 4,
                DeviceType::Cpu => 5,
            };

            if let Some(ref surface) = options.compatible_surface {
                if !adapter.is_surface_supported(surface) {
                    continue;
                }
            }

            compatible_adapters.push((adapter, device_type));
        }

        if compatible_adapters.is_empty() {
            debug!("Unable to find compatible adapters.\nEnsure the backends are set and not Empty.")
        }

        compatible_adapters.sort_by(|a, b| b.1.cmp(&a.1));
        compatible_adapters
    }

    async fn create_device(
        &self,
        window: Arc<Window>,
        options: AdapterOptions,
        device_descriptor: &wgpu::DeviceDescriptor,
        trace_path: Option<&Path>,
        present_mode: wgpu::PresentMode,
    ) -> Result<GpuRenderer, GraphicsError> {
        let mut adapters = self.get_adapters(options);

        while let Some(adapter) = adapters.pop() {
            let ret = adapter
                .0
                .create_renderer(
                    self,
                    &window,
                    device_descriptor,
                    trace_path,
                    present_mode,
                )
                .await;

            if ret.is_ok() {
                match adapter.1 {
                    3 => debug!("A Opengl Or Other Adapter was chosen"),
                    4 => debug!("A Virtual Adapter was chosen."),
                    5 => debug!("A Software rendering Adapter was chosen."),
                    _ => {}
                }

                return ret;
            }
        }

        Err(GraphicsError::Adapter(RequestAdapterError::NotFound))
    }
}
