use crate::{Bounds, GpuDevice, GpuRenderer, Layout};
use bytemuck::{Pod, Zeroable};
use camera::Projection;
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
        let proj_inv: mint::ColumnMatrix4<f32> = inverse_proj.into();
        let seconds = 0.0;
        let size: mint::Vector2<f32> = screen_size.into();

        let mut raw = [0f32; 52 + 4];
        raw[..16].copy_from_slice(&AsRef::<[f32; 16]>::as_ref(&view)[..]);
        raw[16..32].copy_from_slice(&AsRef::<[f32; 16]>::as_ref(&proj)[..]);
        raw[32..48].copy_from_slice(&AsRef::<[f32; 16]>::as_ref(&proj_inv)[..]);
        raw[48..51].copy_from_slice(&AsRef::<[f32; 3]>::as_ref(&eye)[..]);
        raw[51] = scale;
        raw[52..54].copy_from_slice(&AsRef::<[f32; 2]>::as_ref(&size)[..]);
        raw[54] = seconds;

        // Create the uniform buffers.
        let global_buffer = renderer.device().create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("camera buffer"),
                contents: bytemuck::cast_slice(&raw),
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
            let proj_inv: mint::ColumnMatrix4<f32> = inverse_proj.into();
            let eye: mint::Vector3<f32> = self.camera.eye().into();
            let scale = self.camera.scale();

            let mut raw = [0f32; 52];
            raw[..16].copy_from_slice(&AsRef::<[f32; 16]>::as_ref(&view)[..]);
            raw[16..32].copy_from_slice(&AsRef::<[f32; 16]>::as_ref(&proj)[..]);
            raw[32..48]
                .copy_from_slice(&AsRef::<[f32; 16]>::as_ref(&proj_inv)[..]);
            raw[48..51].copy_from_slice(&AsRef::<[f32; 3]>::as_ref(&eye)[..]);
            raw[51] = scale;

            renderer.queue().write_buffer(
                &self.global_buffer,
                0,
                bytemuck::cast_slice(&raw),
            );
        }

        let raw = [frame_time.seconds(); 1];
        renderer.queue().write_buffer(
            &self.global_buffer,
            216,
            bytemuck::cast_slice(&raw),
        );
    }

    pub fn update_screen(
        &mut self,
        renderer: &GpuRenderer,
        screen_size: [f32; 2],
    ) {
        if self.screen_size != screen_size {
            self.screen_size = screen_size;

            renderer.queue().write_buffer(
                &self.global_buffer,
                208,
                bytemuck::cast_slice(&screen_size),
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
