use crate::{GpuRenderer, Layout};

pub struct TextureGroup {
    pub bind_group: wgpu::BindGroup,
}

impl TextureGroup {
    pub fn from_view<K: Layout>(
        renderer: &mut GpuRenderer,
        texture_view: &wgpu::TextureView,
        layout: K,
    ) -> Self {
        let diffuse_sampler =
            renderer.device().create_sampler(&wgpu::SamplerDescriptor {
                label: Some("Texture_sampler"),
                lod_max_clamp: 0.0,
                ..Default::default()
            });

        let entries = vec![
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
            },
        ];

        let layout = renderer.create_layout(layout);
        let bind_group =
            renderer
                .device()
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Texture Bind Group"),
                    layout: &layout,
                    entries: &entries,
                });

        Self { bind_group }
    }
}
