use crate::{
    AsBufferPass, GpuBuffer, GpuRenderer, GraphicsError, Mesh2D,
    Mesh2DRenderPipeline, Mesh2DVertex, OrderedIndex, SetBuffers, System,
};

pub struct Mesh2DRenderer {
    pub vbos: GpuBuffer<Mesh2DVertex>,
}

//TODO: Update this to take in instance buffer index too.
impl Mesh2DRenderer {
    pub fn new(renderer: &GpuRenderer) -> Result<Self, GraphicsError> {
        Ok(Self {
            vbos: GpuBuffer::new(renderer.gpu_device(), 512),
        })
    }

    pub fn add_buffer_store(
        &mut self,
        renderer: &GpuRenderer,
        index: OrderedIndex,
        layer: usize,
    ) {
        self.vbos.add_buffer_store(renderer, index, layer);
    }

    pub fn finalize(&mut self, renderer: &mut GpuRenderer) {
        self.vbos.finalize(renderer);
    }

    pub fn mesh_update(
        &mut self,
        mesh: &mut Mesh2D,
        renderer: &mut GpuRenderer,
        layer: usize,
    ) {
        let index = mesh.update(renderer);

        self.add_buffer_store(renderer, index, layer);
    }

    pub fn use_clipping(&mut self) {
        self.vbos.set_as_clipped();
    }
}

pub trait RenderMesh2D<'a, 'b, Controls>
where
    'b: 'a,
    Controls: camera::controls::Controls,
{
    fn render_2dmeshs(
        &mut self,
        renderer: &'b GpuRenderer,
        buffer: &'b Mesh2DRenderer,
        system: &'b System<Controls>,
        layer: usize,
    );
}

impl<'a, 'b, Controls> RenderMesh2D<'a, 'b, Controls> for wgpu::RenderPass<'a>
where
    'b: 'a,
    Controls: camera::controls::Controls,
{
    fn render_2dmeshs(
        &mut self,
        renderer: &'b GpuRenderer,
        buffer: &'b Mesh2DRenderer,
        system: &'b System<Controls>,
        layer: usize,
    ) {
        if let Some(vbos) = buffer.vbos.buffers.get(layer) {
            if !vbos.is_empty() {
                self.set_buffers(buffer.vbos.as_buffer_pass());
                self.set_pipeline(
                    renderer.get_pipelines(Mesh2DRenderPipeline).unwrap(),
                );

                if buffer.vbos.is_clipped() {
                    let mut scissor_is_default = true;

                    for (details, bounds) in vbos {
                        if let Some(bounds) = bounds {
                            let bounds = system.world_to_screen(false, bounds);

                            self.set_scissor_rect(
                                bounds.x as u32,
                                bounds.y as u32,
                                bounds.z as u32,
                                bounds.w as u32,
                            );
                            scissor_is_default = false;
                        }
                        // Indexs can always start at 0 per mesh data.
                        // Base vertex is the Addition to the Index
                        self.draw_indexed(
                            details.indices_start..details.indices_end,
                            details.vertex_base, //i as i32 * details.max,
                            0..1,
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
                } else {
                    for (details, _bounds) in vbos {
                        // Indexs can always start at 0 per mesh data.
                        // Base vertex is the Addition to the Index
                        self.draw_indexed(
                            details.indices_start..details.indices_end,
                            details.vertex_base, //i as i32 * details.max,
                            0..1,
                        );
                    }
                }

                //we need to reset this back for anything else that might need it after mesh is drawn.
                self.set_vertex_buffer(0, renderer.buffer_object.vertices());
                self.set_index_buffer(
                    renderer.buffer_object.indices(),
                    wgpu::IndexFormat::Uint32,
                );
            }
        }
    }
}
