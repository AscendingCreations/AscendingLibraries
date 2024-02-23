use std::{iter, mem};

use crate::{
    AreaLightLayout, AreaLightRaw, AscendingError, DirLightLayout,
    DirectionalLightRaw, GpuRenderer, InstanceBuffer, LightRenderPipeline,
    Lights, LightsVertex, OrderedIndex, StaticBufferObject, MAX_AREA_LIGHTS,
    MAX_DIR_LIGHTS,
};

use wgpu::util::{align_to, DeviceExt};

pub struct LightRenderer {
    pub buffer: InstanceBuffer<LightsVertex>,
    area_buffer: wgpu::Buffer,
    dir_buffer: wgpu::Buffer,
    area_bind_group: wgpu::BindGroup,
    dir_bind_group: wgpu::BindGroup,
}

impl LightRenderer {
    pub fn new(renderer: &mut GpuRenderer) -> Result<Self, AscendingError> {
        // The size + Padding == 32.
        let area_alignment: usize =
            align_to(mem::size_of::<AreaLightRaw>(), 32) as usize;
        // The size + Padding == 48.
        let dir_alignment: usize =
            align_to(mem::size_of::<DirectionalLightRaw>(), 48) as usize;

        let area: Vec<u8> = iter::repeat(0u8)
            .take(MAX_AREA_LIGHTS * area_alignment)
            .collect();

        let area_buffer = renderer.device().create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Area Light buffer"),
                contents: &area, //2000
                usage: wgpu::BufferUsages::UNIFORM
                    | wgpu::BufferUsages::COPY_DST,
            },
        );

        let dirs: Vec<u8> = iter::repeat(0u8)
            .take(MAX_DIR_LIGHTS * dir_alignment)
            .collect();

        let dir_buffer = renderer.device().create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Directional Light buffer"),
                contents: &dirs, //2000
                usage: wgpu::BufferUsages::UNIFORM
                    | wgpu::BufferUsages::COPY_DST,
            },
        );

        // Create the bind group layout for the area lights.
        let layout = renderer.create_layout(AreaLightLayout);

        // Create the bind group.
        let area_bind_group =
            renderer
                .device()
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: area_buffer.as_entire_binding(),
                    }],
                    label: Some("area_lights_bind_group"),
                });

        // Create the bind group layout for the directional lights.
        let layout = renderer.create_layout(DirLightLayout);

        // Create the bind group.
        let dir_bind_group =
            renderer
                .device()
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: dir_buffer.as_entire_binding(),
                    }],
                    label: Some("dir_lights_bind_group"),
                });

        Ok(Self {
            buffer: InstanceBuffer::new(renderer.gpu_device(), 32),
            dir_buffer,
            area_buffer,
            area_bind_group,
            dir_bind_group,
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

    pub fn lights_update(
        &mut self,
        lights: &mut Lights,
        renderer: &mut GpuRenderer,
        layer: usize,
    ) {
        let index = lights.update(
            renderer,
            &mut self.area_buffer,
            &mut self.dir_buffer,
        );

        self.add_buffer_store(renderer, index, layer);
    }
}

pub trait RenderLights<'a, 'b>
where
    'b: 'a,
{
    fn render_lights(
        &mut self,
        renderer: &'b GpuRenderer,
        buffer: &'b LightRenderer,
        layer: usize,
    );
}

impl<'a, 'b> RenderLights<'a, 'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn render_lights(
        &mut self,
        renderer: &'b GpuRenderer,
        buffer: &'b LightRenderer,
        layer: usize,
    ) {
        if let Some(Some(details)) = buffer.buffer.buffers.get(layer) {
            if buffer.buffer.count() > 0 {
                self.set_bind_group(1, &buffer.area_bind_group, &[]);
                self.set_bind_group(2, &buffer.dir_bind_group, &[]);
                self.set_vertex_buffer(1, buffer.buffer.instances(None));
                self.set_pipeline(
                    renderer.get_pipelines(LightRenderPipeline).unwrap(),
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
