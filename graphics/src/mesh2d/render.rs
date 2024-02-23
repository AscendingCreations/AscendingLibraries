use crate::{
    AsBufferPass, AscendingError, GpuBuffer, GpuRenderer, Mesh2D,
    Mesh2DRenderPipeline, Mesh2DVertex, OrderedIndex, SetBuffers,
};

pub struct Mesh2DRenderer {
    pub vbos: GpuBuffer<Mesh2DVertex>,
}

//TODO: Update this to take in instance buffer index too.
impl Mesh2DRenderer {
    pub fn new(renderer: &GpuRenderer) -> Result<Self, AscendingError> {
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
}

pub trait RenderMesh2D<'a, 'b>
where
    'b: 'a,
{
    fn render_2dmeshs(
        &mut self,
        renderer: &'b GpuRenderer,
        buffer: &'b Mesh2DRenderer,
        layer: usize,
    );
}

impl<'a, 'b> RenderMesh2D<'a, 'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn render_2dmeshs(
        &mut self,
        renderer: &'b GpuRenderer,
        buffer: &'b Mesh2DRenderer,
        layer: usize,
    ) {
        if let Some(vbos) = buffer.vbos.buffers.get(layer) {
            if !vbos.is_empty() {
                self.set_buffers(buffer.vbos.as_buffer_pass());
                self.set_pipeline(
                    renderer.get_pipelines(Mesh2DRenderPipeline).unwrap(),
                );

                for details in vbos {
                    // Indexs can always start at 0 per mesh data.
                    // Base vertex is the Addition to the Index
                    self.draw_indexed(
                        details.indices_start..details.indices_end,
                        details.vertex_base, //i as i32 * details.max,
                        0..1,
                    );
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
