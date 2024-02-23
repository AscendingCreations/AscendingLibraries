use crate::{FxHashMap, GpuDevice, LayoutStorage};
use bytemuck::{Pod, Zeroable};
use std::any::{Any, TypeId};

pub trait PipeLineLayout: Pod + Zeroable {
    fn create_layout(
        &self,
        gpu_device: &mut GpuDevice,
        layouts: &mut LayoutStorage,
        surface_format: wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline;

    fn layout_key(&self) -> (TypeId, Vec<u8>) {
        let type_id = self.type_id();
        let bytes: Vec<u8> =
            bytemuck::try_cast_slice(&[*self]).unwrap_or(&[]).to_vec();

        (type_id, bytes)
    }
}

pub struct PipelineStorage {
    pub(crate) map: FxHashMap<(TypeId, Vec<u8>), wgpu::RenderPipeline>,
}

impl PipelineStorage {
    pub fn new() -> Self {
        Self {
            map: FxHashMap::default(),
        }
    }

    pub fn create_pipeline<K: PipeLineLayout>(
        &mut self,
        device: &mut GpuDevice,
        layout_storage: &mut LayoutStorage,
        surface_format: wgpu::TextureFormat,
        pipeline: K,
    ) {
        let key = pipeline.layout_key();

        self.map.insert(
            key,
            pipeline.create_layout(device, layout_storage, surface_format),
        );
    }

    pub fn get_pipeline<K: PipeLineLayout>(
        &self,
        pipeline: K,
    ) -> Option<&wgpu::RenderPipeline> {
        let key = pipeline.layout_key();

        self.map.get(&key)
    }
}

impl Default for PipelineStorage {
    fn default() -> Self {
        Self::new()
    }
}
