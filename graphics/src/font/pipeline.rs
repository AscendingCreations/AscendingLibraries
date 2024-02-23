use crate::{
    BufferLayout, GpuDevice, LayoutStorage, PipeLineLayout, StaticBufferObject,
    SystemLayout, TextVertex, TextureLayout,
};
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Hash, Pod, Zeroable)]
pub struct TextRenderPipeline;

impl PipeLineLayout for TextRenderPipeline {
    fn create_layout(
        &self,
        gpu_device: &mut GpuDevice,
        layouts: &mut LayoutStorage,
        surface_format: wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline {
        let shader = gpu_device.device().create_shader_module(
            wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("../shaders/textshader.wgsl").into(),
                ),
            },
        );

        let system_layout = layouts.create_layout(gpu_device, SystemLayout);
        let texture_layout = layouts.create_layout(gpu_device, TextureLayout);

        // Create the render pipeline.
        gpu_device.device().create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some("Text render pipeline"),
                layout: Some(&gpu_device.device().create_pipeline_layout(
                    &wgpu::PipelineLayoutDescriptor {
                        label: Some("Text_render_pipeline_layout"),
                        bind_group_layouts: &[
                            &system_layout,
                            &texture_layout,
                            &texture_layout,
                        ],
                        push_constant_ranges: &[],
                    },
                )),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vertex",
                    buffers: &[
                        wgpu::VertexBufferLayout {
                            array_stride: StaticBufferObject::stride(),
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &[
                                StaticBufferObject::vertex_attribute(),
                            ],
                        },
                        wgpu::VertexBufferLayout {
                            array_stride: TextVertex::stride() as u64,
                            step_mode: wgpu::VertexStepMode::Instance,
                            attributes: &TextVertex::attributes(),
                        },
                    ],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fragment",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: surface_format,
                        blend: Some(
                            wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING,
                        ),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            },
        )
    }
}
