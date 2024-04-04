use crate::{Bounds, GpuDevice, GpuRenderer, Layout};
use bytemuck::{Pod, Zeroable};
use camera::Projection;
use crevice::std140::AsStd140;
use glam::{Mat4, Vec2, Vec3, Vec4};
use input::FrameTime;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Clone, Copy, Hash, Pod, Zeroable)]
pub struct SystemLayout;

impl Layout for SystemLayout {
    fn create_layout(
        &self,
        gpu_device: &mut GpuDevice,
    ) -> wgpu::BindGroupLayout {
        gpu_device.device().create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("system_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX
                        | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            },
        )
    }
}

#[derive(AsStd140)]
pub struct CameraUniform {
    view: mint::ColumnMatrix4<f32>,
    proj: mint::ColumnMatrix4<f32>,
    inverse_proj: mint::ColumnMatrix4<f32>,
    eye: mint::Vector3<f32>,
    scale: f32,
}

#[derive(AsStd140)]
pub struct ScreenUniform {
    size: mint::Vector2<f32>,
}

#[derive(AsStd140)]
pub struct TimeUniform {
    //seconds since the start of the program. given by the FrameTime
    seconds: f32,
}

pub struct System<Controls: camera::controls::Controls> {
    camera: camera::Camera<Controls>,
    pub screen_size: [f32; 2],
    global_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl<Controls> System<Controls>
where
    Controls: camera::controls::Controls,
{
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn controls(&self) -> &Controls {
        self.camera.controls()
    }

    pub fn controls_mut(&mut self) -> &mut Controls {
        self.camera.controls_mut()
    }

    pub fn eye(&self) -> [f32; 3] {
        self.camera.eye()
    }

    pub fn new(
        renderer: &mut GpuRenderer,
        projection: Projection,
        controls: Controls,
        screen_size: [f32; 2],
    ) -> Self {
        let mut camera = camera::Camera::new(projection, controls);

        camera.update(0.0);

        // Create the camera uniform.
        let proj = camera.projection();
        let view = camera.view();
        let mat_proj: Mat4 = proj.into();
        let mat_view: Mat4 = view.into();
        let inverse_proj: Mat4 = (mat_proj * mat_view).inverse();
        let eye: mint::Vector3<f32> = camera.eye().into();
        let scale = camera.scale();

        let camera_info = CameraUniform {
            view,
            proj,
            inverse_proj: inverse_proj.into(),
            eye,
            scale,
        };
        let time_info = TimeUniform { seconds: 0.0 };
        let screen_info = ScreenUniform {
            size: screen_size.into(),
        };

        let mut camera_bytes = camera_info.as_std140().as_bytes().to_vec();
        let mut time_bytes = time_info.as_std140().as_bytes().to_vec();
        let mut screen_bytes = screen_info.as_std140().as_bytes().to_vec();

        camera_bytes.append(&mut screen_bytes);
        camera_bytes.append(&mut time_bytes);

        // Create the uniform buffers.
        let global_buffer = renderer.device().create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("camera buffer"),
                contents: &camera_bytes,
                usage: wgpu::BufferUsages::UNIFORM
                    | wgpu::BufferUsages::COPY_DST,
            },
        );

        // Create the bind group layout for the camera.
        let layout = renderer.create_layout(SystemLayout);

        // Create the bind group.
        let bind_group =
            renderer
                .device()
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: global_buffer.as_entire_binding(),
                    }],
                    label: Some("system_bind_group"),
                });

        Self {
            camera,
            screen_size,
            global_buffer,
            bind_group,
        }
    }

    pub fn projection(&self) -> mint::ColumnMatrix4<f32> {
        self.camera.projection()
    }

    pub fn set_controls(&mut self, controls: Controls) -> Controls {
        self.camera.set_controls(controls)
    }

    pub fn set_projection(&mut self, projection: Projection) {
        self.camera.set_projection(projection);
    }

    pub fn update(&mut self, renderer: &GpuRenderer, frame_time: &FrameTime) {
        if self.camera.update(frame_time.delta_seconds()) {
            let proj = self.camera.projection();
            let view = self.camera.view();
            let mat_proj: Mat4 = proj.into();
            let mat_view: Mat4 = view.into();
            let inverse_proj: Mat4 = (mat_proj * mat_view).inverse();
            //inverse_proj = inverse_proj.inverse();

            let eye: mint::Vector3<f32> = self.camera.eye().into();
            let scale = self.camera.scale();

            let camera_info = CameraUniform {
                view,
                proj,
                inverse_proj: inverse_proj.into(),
                eye,
                scale,
            };

            renderer.queue().write_buffer(
                &self.global_buffer,
                0,
                camera_info.as_std140().as_bytes(),
            );
        }

        let time_info = TimeUniform {
            seconds: frame_time.seconds(),
        };

        renderer.queue().write_buffer(
            &self.global_buffer,
            216,
            time_info.as_std140().as_bytes(),
        );
    }

    pub fn update_screen(
        &mut self,
        renderer: &GpuRenderer,
        screen_size: [f32; 2],
    ) {
        if self.screen_size != screen_size {
            self.screen_size = screen_size;
            let screen_info = ScreenUniform {
                size: screen_size.into(),
            };

            renderer.queue().write_buffer(
                &self.global_buffer,
                208,
                screen_info.as_std140().as_bytes(),
            );
        }
    }

    pub fn view(&self) -> mint::ColumnMatrix4<f32> {
        self.camera.view()
    }

    pub fn projected_world_to_screen(
        &self,
        scale: bool,
        bounds: &Bounds,
    ) -> Vec4 {
        let height = f32::abs(bounds.top - bounds.bottom);
        let projection = Mat4::from(self.camera.projection());
        let model = Mat4::IDENTITY;
        let view = if scale {
            Mat4::from(self.camera.view())
        } else {
            Mat4::IDENTITY
        };
        let clip_coords = projection
            * view
            * model
            * Vec4::new(bounds.left, bounds.bottom, 1.0, 1.0);
        let coords = Vec3::from_slice(&clip_coords.to_array()) / clip_coords.w;

        let xy = Vec2::new(
            (coords.x + 1.0) * 0.5 * self.screen_size[0],
            (1.0 - coords.y) * 0.5 * self.screen_size[1],
        );

        let (bw, bh, objh) = if scale {
            (
                bounds.right * self.camera.scale(),
                bounds.top * self.camera.scale(),
                height * self.camera.scale(),
            )
        } else {
            (bounds.right, bounds.top, height)
        };

        Vec4::new(xy.x, xy.y - objh, bw, bh)
    }

    pub fn world_to_screen(&self, scale: bool, bounds: &Bounds) -> Vec4 {
        let height = f32::abs(bounds.top - bounds.bottom);
        let projection = Mat4::from(self.camera.projection());
        let model = Mat4::IDENTITY;
        let clip_coords = projection
            * model
            * Vec4::new(bounds.left, bounds.bottom, 1.0, 1.0);
        let coords = Vec3::from_slice(&clip_coords.to_array()) / clip_coords.w;

        let xy = Vec2::new(
            (coords.x + 1.0) * 0.5 * self.screen_size[0],
            (1.0 - coords.y) * 0.5 * self.screen_size[1],
        );

        let (bw, bh, objh) = if scale {
            (
                bounds.right * self.camera.scale(),
                bounds.top * self.camera.scale(),
                height * self.camera.scale(),
            )
        } else {
            (bounds.right, bounds.top, height)
        };

        Vec4::new(xy.x, xy.y - objh, bw, bh)
    }

    pub fn world_to_screen_direct(
        screen_size: [f32; 2],
        scale: f32,
        projection: Mat4,
        left: f32,
        bottom: f32,
        right: f32,
        top: f32,
    ) -> Vec4 {
        let height = f32::abs(top - bottom);
        let model = Mat4::IDENTITY;
        let clip_coords =
            projection * model * Vec4::new(left, bottom, 1.0, 1.0);
        let coords = Vec3::from_slice(&clip_coords.to_array()) / clip_coords.w;

        let xy = Vec2::new(
            (coords.x + 1.0) * 0.5 * screen_size[0],
            (1.0 - coords.y) * 0.5 * screen_size[1],
        );

        let (bw, bh, objh) = if scale != 1.0 {
            (right * scale, top * scale, height * scale)
        } else {
            (right, top, height)
        };
        // We must minus the height to flip the Y location to window coords.
        // You might not need to do this based on how you handle your Y coords.
        Vec4::new(xy.x, xy.y - objh, bw, bh)
    }
}
