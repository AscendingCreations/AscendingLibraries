use crate::{Bounds, CameraType, GpuDevice, GpuRenderer, Layout};
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
    manual_view: Mat4,
    manual_scale: f32,
    manual_changed: bool,
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
        manual_view: Mat4,
        manual_scale: f32,
    ) -> Self {
        let mut camera = camera::Camera::new(projection, controls);

        camera.update(0.0);

        // Create the camera uniform.
        let proj = camera.projection();
        let view = camera.view();
        let inverse_proj: Mat4 = (proj).inverse();
        let eye = camera.eye();
        let scale = camera.scale();
        let seconds = 0.0;

        let mut raw = [0f32; 52 + 4 + 20];
        raw[..16].copy_from_slice(&AsRef::<[f32; 16]>::as_ref(&view)[..]);
        raw[16..32].copy_from_slice(&AsRef::<[f32; 16]>::as_ref(&proj)[..]);
        raw[32..48]
            .copy_from_slice(&AsRef::<[f32; 16]>::as_ref(&inverse_proj)[..]);
        raw[48..51].copy_from_slice(&eye);
        raw[51] = scale;
        raw[52..54].copy_from_slice(&screen_size);
        raw[54] = seconds;
        raw[56..72]
            .copy_from_slice(&AsRef::<[f32; 16]>::as_ref(&manual_view)[..]);
        raw[72] = manual_scale;

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
            manual_changed: false,
            manual_scale,
            manual_view,
        }
    }

    pub fn projection(&self) -> Mat4 {
        self.camera.projection()
    }

    pub fn set_controls(&mut self, controls: Controls) -> Controls {
        self.camera.set_controls(controls)
    }

    pub fn set_projection(&mut self, projection: Projection) {
        self.camera.set_projection(projection);
    }

    pub fn set_manual_view(&mut self, view: Mat4, scale: f32) {
        self.manual_view = view;
        self.manual_scale = scale;
        self.manual_changed = true;
    }

    pub fn manual_view(&self) -> Mat4 {
        self.manual_view
    }

    pub fn mut_manual_view(&mut self) -> &mut Mat4 {
        self.manual_changed = true;
        &mut self.manual_view
    }

    pub fn manual_scale(&self) -> f32 {
        self.manual_scale
    }

    pub fn mut_manual_scale(&mut self) -> &mut f32 {
        self.manual_changed = true;
        &mut self.manual_scale
    }

    pub fn update(&mut self, renderer: &GpuRenderer, frame_time: &FrameTime) {
        if self.camera.update(frame_time.delta_seconds()) {
            let proj = self.camera.projection();
            let view = self.camera.view();
            let inverse_proj: Mat4 = (proj).inverse();
            let eye = self.camera.eye();
            let scale = self.camera.scale();

            let mut raw = [0f32; 52];
            raw[..16].copy_from_slice(&AsRef::<[f32; 16]>::as_ref(&view)[..]);
            raw[16..32].copy_from_slice(&AsRef::<[f32; 16]>::as_ref(&proj)[..]);
            raw[32..48].copy_from_slice(
                &AsRef::<[f32; 16]>::as_ref(&inverse_proj)[..],
            );
            raw[48..51].copy_from_slice(&eye);
            raw[51] = scale;

            renderer.queue().write_buffer(
                &self.global_buffer,
                0,
                bytemuck::cast_slice(&raw),
            );
        }

        renderer.queue().write_buffer(
            &self.global_buffer,
            216,
            bytemuck::bytes_of(&frame_time.seconds()),
        );

        if self.manual_changed {
            let mut raw = [0f32; 17];
            raw[..16].copy_from_slice(
                &AsRef::<[f32; 16]>::as_ref(&self.manual_view)[..],
            );
            raw[16] = self.manual_scale;

            renderer.queue().write_buffer(
                &self.global_buffer,
                224,
                bytemuck::cast_slice(&raw),
            );
        }
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

    pub fn view(&self) -> Mat4 {
        self.camera.view()
    }

    pub fn projected_world_to_screen(
        &self,
        camera_type: CameraType,
        bounds: &Bounds,
    ) -> Vec4 {
        let height = f32::abs(bounds.top - bounds.bottom);
        let projection = self.camera.projection();
        let model = Mat4::IDENTITY;
        let view = match camera_type {
            CameraType::None => Mat4::IDENTITY,
            CameraType::ManualView | CameraType::ManualViewWithScale => {
                self.manual_view
            }
            CameraType::ControlView | CameraType::ControlViewWithScale => {
                self.camera.view()
            }
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

        let (bw, bh, objh) = match camera_type {
            CameraType::ManualViewWithScale => (
                bounds.right * self.manual_scale,
                bounds.top * self.manual_scale,
                height * self.manual_scale,
            ),
            CameraType::ControlViewWithScale => (
                bounds.right * self.camera.scale(),
                bounds.top * self.camera.scale(),
                height * self.camera.scale(),
            ),
            _ => (bounds.right, bounds.top, height),
        };

        Vec4::new(xy.x, xy.y - objh, bw, bh)
    }

    pub fn world_to_screen(
        &self,
        camera_type: CameraType,
        bounds: &Bounds,
    ) -> Vec4 {
        let height = f32::abs(bounds.top - bounds.bottom);
        let projection = self.camera.projection();
        let model = Mat4::IDENTITY;
        let clip_coords = projection
            * model
            * Vec4::new(bounds.left, bounds.bottom, 1.0, 1.0);
        let coords = Vec3::from_slice(&clip_coords.to_array()) / clip_coords.w;

        let xy = Vec2::new(
            (coords.x + 1.0) * 0.5 * self.screen_size[0],
            (1.0 - coords.y) * 0.5 * self.screen_size[1],
        );

        let (bw, bh, objh) = match camera_type {
            CameraType::ManualViewWithScale => (
                bounds.right * self.manual_scale,
                bounds.top * self.manual_scale,
                height * self.manual_scale,
            ),
            CameraType::ControlViewWithScale => (
                bounds.right * self.camera.scale(),
                bounds.top * self.camera.scale(),
                height * self.camera.scale(),
            ),
            _ => (bounds.right, bounds.top, height),
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
